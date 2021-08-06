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
use fvr_engine_core::{map2d_iter_index_mut, prelude::*, xy_tuple_iter};

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

static CHASING_MOB_THING: Thing = Thing {
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

static FLEEING_MOB_THING: Thing = Thing {
    tile: Tile {
        glyph: 'M',
        layout: TileLayout::Center,
        style: TileStyle::Regular,
        size: TileSize::Normal,
        outlined: false,
        background_color: TileColor::TRANSPARENT,
        foreground_color: PaletteColor::BrightBlue.const_into(),
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
    pub stationary: bool,
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
    dimensions: ICoord,
    world: World,
}

pub struct CellMap(pub GridMap<Cell>);
pub struct PassableMap(pub GridMap<Passability>);
pub struct ActorMap(pub GridMap<Option<Actor>>);
pub struct ChaseMap(pub DijkstraMap);
pub struct FleeMap(pub DijkstraMap);
pub struct PlayerXY(pub ICoord);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct HasXY(pub ICoord);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct IsActor(pub Actor);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ChasingPlayer;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct FleeingPlayer;

struct ChasePlayerSystem;

impl<'a> System<'a> for ChasePlayerSystem {
    type SystemData = (
        WriteExpect<'a, ActorMap>,
        ReadExpect<'a, ChaseMap>,
        ReadExpect<'a, PlayerXY>,
        ReadStorage<'a, ChasingPlayer>,
        ReadStorage<'a, IsActor>,
        WriteStorage<'a, HasXY>,
    );

    fn run(
        &mut self,
        (mut actor_map, chase_map, player_xy, chasing_player, is_actor, mut xy): Self::SystemData,
    ) {
        for (_, _, p) in (&chasing_player, &is_actor, &mut xy).join() {
            let dir = chase_map.0.min_direction(p.0);

            if dir == NULL_DIRECTION {
                actor_map.0.get_xy_mut(p.0).unwrap().stationary = true;
                continue;
            }

            let new_position =
                ((p.0 .0 as i32 + dir.dx()) as i32, (p.0 .1 as i32 + dir.dy()) as i32);

            if new_position == player_xy.0 || actor_map.0.get_xy(new_position).is_some() {
                actor_map.0.get_xy_mut(p.0).unwrap().stationary = true;
                continue;
            }

            *actor_map.0.get_xy_mut(new_position) = actor_map.0.get_xy_mut(p.0).take();
            actor_map.0.get_xy_mut(new_position).unwrap().stationary = false;

            p.0 = new_position;
        }
    }
}

struct FleePlayerSystem;

impl<'a> System<'a> for FleePlayerSystem {
    type SystemData = (
        WriteExpect<'a, ActorMap>,
        ReadExpect<'a, FleeMap>,
        ReadExpect<'a, PlayerXY>,
        ReadStorage<'a, FleeingPlayer>,
        ReadStorage<'a, IsActor>,
        WriteStorage<'a, HasXY>,
    );

    fn run(
        &mut self,
        (mut actor_map, flee_map, player_xy, fleeing_player, is_actor, mut xy): Self::SystemData,
    ) {
        for (_, _, p) in (&fleeing_player, &is_actor, &mut xy).join() {
            let dir = flee_map.0.min_direction(p.0);

            if dir == NULL_DIRECTION {
                actor_map.0.get_xy_mut(p.0).unwrap().stationary = true;
                continue;
            }

            let new_position =
                ((p.0 .0 as i32 + dir.dx()) as i32, (p.0 .1 as i32 + dir.dy()) as i32);

            if new_position == player_xy.0 || actor_map.0.get_xy(new_position).is_some() {
                actor_map.0.get_xy_mut(p.0).unwrap().stationary = true;
                continue;
            }

            *actor_map.0.get_xy_mut(new_position) = actor_map.0.get_xy_mut(p.0).take();
            actor_map.0.get_xy_mut(new_position).unwrap().stationary = false;

            p.0 = new_position;
        }
    }
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

        // Chasing mobs.
        for _ in 0..100 {
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            if xy == self.player_xy().0
                || self.actor_map().0.get_xy(xy).is_some()
                || !self.passable_map().0.get_xy(xy).passable()
            {
                continue;
            }

            let entity = self.world.create_entity().with(HasXY(xy)).build();
            let actor = Actor { thing: CHASING_MOB_THING, entity, stationary: false };

            self.world.write_component::<IsActor>().insert(entity, IsActor(actor))?;
            self.world.write_component::<ChasingPlayer>().insert(entity, ChasingPlayer {})?;
            *self.actor_map_mut().0.get_xy_mut(xy) = Some(actor);
        }

        // Fleeing Mobs.
        for _ in 0..100 {
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            if xy == self.player_xy().0
                || self.actor_map().0.get_xy(xy).is_some()
                || !self.passable_map().0.get_xy(xy).passable()
            {
                continue;
            }

            let entity = self.world.create_entity().with(HasXY(xy)).build();
            let actor = Actor { thing: FLEEING_MOB_THING, entity, stationary: false };

            self.world.write_component::<IsActor>().insert(entity, IsActor(actor))?;
            self.world.write_component::<FleeingPlayer>().insert(entity, FleeingPlayer {})?;
            *self.actor_map_mut().0.get_xy_mut(xy) = Some(actor);
        }

        Ok(())
    }

    fn refresh_passable_map(&mut self) {
        xy_tuple_iter!(x, y, self.dimensions, {
            let mut passable = true;

            if let Some(actor) = self.actor_map().0.get_xy((x, y)) {
                // TODO: Is this correct?
                passable = true; //actor.stationary;
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
            *self.flee_map_mut().0.states_mut().get_xy_mut((x, y)) = passability.into();
        });

        let player_xy = self.player_xy().0;

        // Caluclate the chase map.
        *self.chase_map_mut().0.states_mut().get_xy_mut(player_xy) = DIJKSTRA_DEFAULT_GOAL;
        self.chase_map_mut().0.calculate();

        // Calculate the flee map using the max xy of the chase map.
        let farthest_xy = self.chase_map().0.farthest_xy();
        *self.flee_map_mut().0.states_mut().get_xy_mut(farthest_xy) = DIJKSTRA_DEFAULT_GOAL;
        self.flee_map_mut().0.calculate();

        // Modulate the flee map by some coefficient of the chase map.
        xy_tuple_iter!(x, y, self.dimensions, {
            let chase_weight = {
                if let Some(weight) = self.chase_map().0.get_xy((x, y)) {
                    *weight
                } else {
                    continue;
                }
            };

            self.flee_map_mut().0.combine_xy((x, y), chase_weight * -1.2);
        });

        Ok(())
    }

    pub fn new(dimensions: ICoord) -> Result<Self> {
        let mut cell_map = GridMap::new(dimensions);
        let mut passable_map = GridMap::new(dimensions);
        let actor_map = GridMap::new(dimensions);
        let chase_map = DijkstraMap::new(dimensions, Distance::Euclidean);
        let flee_map = DijkstraMap::new(dimensions, Distance::Euclidean);

        Self::generate_dummy_map(&mut cell_map, &mut passable_map);

        let mut world = World::new();

        // Register components.
        world.register::<HasXY>();
        world.register::<IsActor>();
        world.register::<ChasingPlayer>();
        world.register::<FleeingPlayer>();

        // Insert resources.
        world.insert(CellMap(cell_map));
        world.insert(PassableMap(passable_map));
        world.insert(ActorMap(actor_map));
        world.insert(ChaseMap(chase_map));
        world.insert(FleeMap(flee_map));
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
        let new_xy = (player_xy.0 as i32 + dir.dx(), player_xy.1 as i32 + dir.dy());

        if !self.cell_map().0.in_bounds_icoord(new_xy) {
            return Ok(false);
        }

        if self.actor_map().0.get_xy(new_xy).is_none()
            && self.cell_map().0.get_xy(new_xy).passable()
        {
            self.player_xy_mut().0 = new_xy;
            self.refresh_passable_map();
            self.refresh_navigation_maps()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn teleport_player(&mut self, xy: ICoord) -> Result<bool> {
        if !self.cell_map().0.in_bounds(xy) {
            return Ok(false);
        }

        if self.actor_map().0.get_xy(xy).is_none() && self.cell_map().0.get_xy(xy).passable() {
            self.player_xy_mut().0 = xy;
            self.refresh_passable_map();
            self.refresh_navigation_maps()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn dispatch(&mut self) -> Result<()> {
        let mut chase_player_system = ChasePlayerSystem {};
        chase_player_system.run_now(&self.world);

        let mut flee_player_system = FleePlayerSystem {};
        flee_player_system.run_now(&self.world);

        self.world.maintain();
        self.refresh_passable_map();
        self.refresh_navigation_maps()?;

        Ok(())
    }
}
