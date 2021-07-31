//-------------------------------------------------------------------------------------------------
// STD crate includes.
//-------------------------------------------------------------------------------------------------
use std::cell::RefCell;
use std::sync::Mutex;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;
use rand::prelude::*;
use specs::shred::{Fetch, FetchMut};
use specs::{prelude::*, Component};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::{
    map2d_iter_index_mut, prelude::FleeMap as CoreFleeMap, prelude::*, xy_tuple_iter,
};

static TREE_THING: Thing = Thing {
    tile: Tile {
        glyph: 'T',
        layout: TileLayout::Center,
        style: TileStyle::Regular,
        size: TileSize::Normal,
        outlined: false,
        background_color: TileColor::TRANSPARENT,
        foreground_color: PaletteColor::BrightGreen.const_into(),
        outline_color: TileColor::TRANSPARENT,
        background_opacity: 1.0,
        foreground_opacity: 1.0,
        outline_opacity: 1.0,
    },
    passable: false,
};

static GRASS_THING: Thing = Thing {
    tile: Tile {
        glyph: '.',
        layout: TileLayout::Center,
        style: TileStyle::Regular,
        size: TileSize::Normal,
        outlined: false,
        background_color: TileColor::TRANSPARENT,
        foreground_color: PaletteColor::DarkGreen.const_into(),
        outline_color: TileColor::TRANSPARENT,
        background_opacity: 1.0,
        foreground_opacity: 1.0,
        outline_opacity: 1.0,
    },
    passable: true,
};

static MOB_THING: Thing = Thing {
    tile: Tile {
        glyph: 'M',
        layout: TileLayout::Center,
        style: TileStyle::Regular,
        size: TileSize::Normal,
        outlined: false,
        background_color: TileColor::TRANSPARENT,
        foreground_color: PaletteColor::BrightRed.const_into(),
        outline_color: TileColor::TRANSPARENT,
        background_opacity: 1.0,
        foreground_opacity: 1.0,
        outline_opacity: 1.0,
    },
    passable: true,
};

#[derive(Copy, Clone, Debug, Default)]
pub struct Thing {
    pub tile: Tile,
    pub passable: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct Actor {
    pub thing: Thing,
    pub entity: Entity,
}

#[derive(Clone, Debug, Default)]
pub struct Cell {
    pub things: Vec<Thing>,
}

impl Cell {
    pub fn passable(&self) -> bool {
        self.things.iter().all(|thing| thing.passable)
    }
}

pub struct Zone {
    dimensions: UCoord,
    world: World,
}

pub struct CellMap(pub GridMap<Cell>);
pub struct PassableMap(pub GridMap<Passability>);
pub struct ActorMap(pub GridMap<Option<Actor>>);
pub struct ChaseMap(pub DijkstraMap);
pub struct FleeMap(pub Mutex<RefCell<CoreFleeMap>>);
pub struct PlayerXY(pub UCoord);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct HasPosition(pub UCoord);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct IsActor(pub Actor);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ChasingPlayer {
    path: Vec<UCoord>,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct FleeingPlayer {
    path: Vec<UCoord>,
}

impl Zone {
    fn generate_dummy_map(cell_map: &mut GridMap<Cell>, passable_map: &mut GridMap<Passability>) {
        let mut rng = thread_rng();

        map2d_iter_index_mut!(cell_map, x, y, item, {
            if rng.gen::<u8>() % 5 == 0 {
                *item = Cell { things: vec![TREE_THING] };
                *passable_map.get_xy_mut((x, y)) = Passability::Blocked;
            } else {
                *item = Cell { things: vec![GRASS_THING] };
                *passable_map.get_xy_mut((x, y)) = Passability::Passable;
            }
        });

        let dimensions = cell_map.dimensions();
        cell_map.get_xy_mut((dimensions.0 / 2, dimensions.1 / 2)).things[0] = GRASS_THING;
    }

    fn populate_mobs(&mut self) -> Result<()> {
        let mut rng = thread_rng();

        for _ in 0..10 {
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            if xy == self.player_xy().0
                || self.actor_map().0.get_xy(xy).is_some()
                || !self.passable_map().0.get_xy(xy).passable()
            {
                continue;
            }

            let entity = self.world.create_entity().with(HasPosition(xy)).build();
            let actor = Actor { thing: MOB_THING, entity };

            self.world.write_component::<IsActor>().insert(entity, IsActor(actor))?;
            *self.actor_map_mut().0.get_xy_mut(xy) = Some(actor);
        }

        Ok(())
    }

    fn refresh_passable_map(&mut self) {
        xy_tuple_iter!(x, y, self.dimensions, {
            let mut passable = true;

            if self.actor_map().0.get_xy((x, y)).is_some() {
                passable = false;
            } else if !self.cell_map().0.get_xy((x, y)).passable() {
                passable = false;
            }

            *self.passable_map_mut().0.get_xy_mut((x, y)) = passable.into();
        });

        *self.passable_map_mut().0.get_xy_mut(self.player_xy().0) = Passability::Blocked;
    }

    fn refresh_navigation_maps(&mut self) -> Result<()> {
        xy_tuple_iter!(x, y, self.dimensions, {
            let passability = *self.passable_map().0.get_xy((x, y));
            *self.chase_map_mut().0.states_mut().get_xy_mut((x, y)) = passability.into();
        });

        let player_xy = self.player_xy().0;
        *self.chase_map_mut().0.states_mut().get_xy_mut(player_xy) = DIJKSTRA_DEFAULT_GOAL;

        self.chase_map_mut().0.calculate();
        self.flee_map().0.lock().unwrap().borrow_mut().calculate(&self.chase_map().0);

        Ok(())
    }

    pub fn new(dimensions: UCoord) -> Result<Self> {
        let mut cell_map = GridMap::new(dimensions);
        let mut passable_map = GridMap::new(dimensions);
        let actor_map = GridMap::new(dimensions);
        let chase_map = DijkstraMap::new(dimensions, Distance::Euclidean);
        let flee_map = CoreFleeMap::new(dimensions, Distance::Euclidean);

        Self::generate_dummy_map(&mut cell_map, &mut passable_map);

        let mut world = World::new();

        // Register components.
        world.register::<HasPosition>();
        world.register::<IsActor>();
        world.register::<ChasingPlayer>();
        world.register::<FleeingPlayer>();

        // Insert resources.
        world.insert(CellMap(cell_map));
        world.insert(PassableMap(passable_map));
        world.insert(ActorMap(actor_map));
        world.insert(ChaseMap(chase_map));
        world.insert(FleeMap(Mutex::new(RefCell::new(flee_map))));
        world.insert(PlayerXY((dimensions.0 / 2, dimensions.1 / 2)));

        let mut zone = Self { dimensions, world };
        zone.populate_mobs()?;
        zone.refresh_passable_map();
        zone.refresh_navigation_maps()?;

        Ok(zone)
    }

    pub fn cell_map(&self) -> Fetch<CellMap> {
        self.world.read_resource::<CellMap>()
    }

    pub fn cell_map_mut(&mut self) -> FetchMut<CellMap> {
        self.world.write_resource::<CellMap>()
    }

    pub fn passable_map(&self) -> Fetch<PassableMap> {
        self.world.read_resource::<PassableMap>()
    }

    pub fn passable_map_mut(&self) -> FetchMut<PassableMap> {
        self.world.write_resource::<PassableMap>()
    }

    pub fn actor_map(&self) -> Fetch<ActorMap> {
        self.world.read_resource::<ActorMap>()
    }

    pub fn actor_map_mut(&mut self) -> FetchMut<ActorMap> {
        self.world.write_resource::<ActorMap>()
    }

    pub fn chase_map(&self) -> Fetch<ChaseMap> {
        self.world.read_resource::<ChaseMap>()
    }

    pub fn chase_map_mut(&mut self) -> FetchMut<ChaseMap> {
        self.world.write_resource::<ChaseMap>()
    }

    pub fn flee_map(&self) -> Fetch<FleeMap> {
        self.world.read_resource::<FleeMap>()
    }

    pub fn flee_map_mut(&mut self) -> FetchMut<FleeMap> {
        self.world.write_resource::<FleeMap>()
    }

    pub fn player_xy(&self) -> Fetch<PlayerXY> {
        self.world.read_resource::<PlayerXY>()
    }

    pub fn player_xy_mut(&mut self) -> FetchMut<PlayerXY> {
        self.world.write_resource::<PlayerXY>()
    }

    pub fn move_player(&mut self, dir: Direction) -> Result<bool> {
        let player_xy = self.player_xy().0;
        let new_xy = Misc::itou((player_xy.0 as i32 + dir.dx(), player_xy.1 as i32 + dir.dy()));

        if self.cell_map().0.get_xy(new_xy).passable() {
            self.player_xy_mut().0 = new_xy;
            self.refresh_passable_map();
            self.refresh_navigation_maps()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn dispatch(&mut self) {}
}
