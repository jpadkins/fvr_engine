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

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::server::*;

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
    pub passable: bool,
    pub tile: Tile,
}

#[derive(Copy, Clone, Debug)]
pub struct Actor {
    pub entity: Entity,
    pub last_weight: Option<f32>,
    pub stationary: i32,
    pub thing: Thing,
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
pub struct Player {
    pub fov: Fov,
    pub stationary: i32,
    pub xy: ICoord,
}

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
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, ActorMap>,
        ReadExpect<'a, ChaseMap>,
        ReadExpect<'a, Player>,
        ReadStorage<'a, ChasingPlayer>,
        ReadStorage<'a, IsActor>,
        WriteStorage<'a, HasXY>,
    );

    fn run(
        &mut self,
        (mut actor_map, chase_map, player, chasing_player, is_actor, mut xy): Self::SystemData,
    ) {
        for (_, _, p) in (&chasing_player, &is_actor, &mut xy).join() {
            let new_position = chase_map.0.best_neighbor(p.0);

            if new_position.is_none() {
                actor_map.0.get_xy_mut(p.0).as_mut().unwrap().stationary += 1;
                continue;
            }

            let (new_position, _weight) = new_position.unwrap();

            if new_position == player.xy || actor_map.0.get_xy(new_position).is_some() {
                actor_map.0.get_xy_mut(p.0).as_mut().unwrap().stationary += 1;
                continue;
            }

            *actor_map.0.get_xy_mut(new_position) = actor_map.0.get_xy_mut(p.0).take();
            actor_map.0.get_xy_mut(new_position).as_mut().unwrap().stationary = 0;

            p.0 = new_position;
        }
    }
}

struct FleePlayerSystem;

impl<'a> System<'a> for FleePlayerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, ActorMap>,
        ReadExpect<'a, FleeMap>,
        ReadExpect<'a, Player>,
        ReadStorage<'a, FleeingPlayer>,
        ReadStorage<'a, IsActor>,
        WriteStorage<'a, HasXY>,
    );

    fn run(
        &mut self,
        (mut actor_map, flee_map, player, fleeing_player, is_actor, mut xy): Self::SystemData,
    ) {
        for (_, _, p) in (&fleeing_player, &is_actor, &mut xy).join() {
            let new_position = flee_map.0.best_neighbor(p.0);

            if new_position.is_none() {
                actor_map.0.get_xy_mut(p.0).as_mut().unwrap().stationary += 1;
                continue;
            }

            let (new_position, _weight) = new_position.unwrap();

            if new_position == player.xy || actor_map.0.get_xy(new_position).is_some() {
                actor_map.0.get_xy_mut(p.0).as_mut().unwrap().stationary += 1;
                continue;
            }

            *actor_map.0.get_xy_mut(new_position) = actor_map.0.get_xy_mut(p.0).take();
            actor_map.0.get_xy_mut(new_position).as_mut().unwrap().stationary = 0;

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

        cell_map.get_xy_mut((27, 16)).things[0] = GRASS_THING;
    }

    fn populate_mobs(&mut self) -> Result<()> {
        let mut rng = thread_rng();

        // Chasing mobs.
        for _ in 0..20 {
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            if xy == self.player().xy
                || self.actor_map().0.get_xy(xy).is_some()
                || !self.passable_map().0.get_xy(xy).passable()
            {
                continue;
            }

            let entity = self.world.create_entity().with(HasXY(xy)).build();
            let actor =
                Actor { thing: CHASING_MOB_THING, entity, stationary: 0, last_weight: None };

            self.world.write_component::<IsActor>().insert(entity, IsActor(actor))?;
            self.world.write_component::<ChasingPlayer>().insert(entity, ChasingPlayer {})?;
            *self.actor_map_mut().0.get_xy_mut(xy) = Some(actor);
        }

        // Fleeing Mobs.
        for _ in 0..20 {
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            if xy == self.player().xy
                || self.actor_map().0.get_xy(xy).is_some()
                || !self.passable_map().0.get_xy(xy).passable()
            {
                continue;
            }

            let entity = self.world.create_entity().with(HasXY(xy)).build();
            let actor =
                Actor { thing: FLEEING_MOB_THING, entity, stationary: 0, last_weight: None };

            self.world.write_component::<IsActor>().insert(entity, IsActor(actor))?;
            self.world.write_component::<FleeingPlayer>().insert(entity, FleeingPlayer {})?;
            *self.actor_map_mut().0.get_xy_mut(xy) = Some(actor);
        }

        Ok(())
    }

    fn refresh_navigation_maps(&mut self) {
        // Refresh the passability map and states.
        xy_tuple_iter!(x, y, self.dimensions, {
            let passable;

            // Treat actors who have not moved in two rounds as obstacles.
            if let Some(actor) = self.actor_map().0.get_xy((x, y)) {
                passable = !(actor.stationary > 2);
            } else {
                passable = self.cell_map().0.get_xy((x, y)).passable();
            }

            *self.passable_map_mut().0.get_xy_mut((x, y)) = passable.into();
            *self.chase_map_mut().0.states_mut().get_xy_mut((x, y)) = passable.into();
            *self.flee_map_mut().0.states_mut().get_xy_mut((x, y)) = passable.into();
            *self.player_mut().fov.states_mut().get_xy_mut((x, y)) = passable.into();
        });

        let player_xy = self.player().xy;
        *self.passable_map_mut().0.get_xy_mut(player_xy) = Passability::Passable;

        // Caluclate the chase map.
        *self.chase_map_mut().0.states_mut().get_xy_mut(player_xy) = DIJKSTRA_DEFAULT_GOAL;
        self.chase_map_mut().0.calculate();

        // Calculate the flee map using the max xy of the chase map.
        let highest_xy = self.chase_map().0.highest_xy();

        if let Some(xy) = highest_xy {
            // If a path exists to the player, use "intelligent" combined flee pathing.
            *self.flee_map_mut().0.states_mut().get_xy_mut(xy) = DIJKSTRA_DEFAULT_GOAL;
            self.flee_map_mut().0.calculate();

            // Modulate the flee map by some coefficient of the chase map.
            let highest_weight = self.chase_map().0.get_xy(xy).unwrap();

            xy_tuple_iter!(x, y, self.dimensions, {
                let chase_weight = {
                    if let Some(weight) = self.chase_map().0.get_xy((x, y)) {
                        *weight
                    } else {
                        continue;
                    }
                };

                self.flee_map_mut().0.combine_xy((x, y), highest_weight - chase_weight);
            });
        } else {
            self.flee_map_mut().0.calculate();
        }

        self.flee_map_mut().0.refresh_highest();
    }

    fn refresh_player_fov(&mut self) {
        let player_xy = self.player().xy;
        self.player_mut().fov.calculate(player_xy, 33.0);
    }

    pub fn new(dimensions: ICoord) -> Result<Self> {
        let mut cell_map = GridMap::new(dimensions);
        let mut passable_map = GridMap::new(dimensions);
        let actor_map = GridMap::new(dimensions);
        let chase_map = DijkstraMap::new(dimensions, Distance::Euclidean);
        let flee_map = DijkstraMap::new(dimensions, Distance::Euclidean);
        let player_fov = Fov::new(dimensions, Distance::Euclidean);

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
        world.insert(Player { xy: (27, 16), stationary: 0, fov: player_fov });

        let mut zone = Self { dimensions, world };
        zone.populate_mobs()?;
        zone.refresh_navigation_maps();
        zone.refresh_player_fov();

        Ok(zone)
    }

    pub fn width(&self) -> i32 {
        self.dimensions.0
    }

    pub fn height(&self) -> i32 {
        self.dimensions.1
    }

    pub fn dimensions(&self) -> ICoord {
        self.dimensions
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

    pub fn player(&self) -> Fetch<Player> {
        self.world.read_resource::<Player>()
    }

    pub fn player_mut(&mut self) -> FetchMut<Player> {
        self.world.write_resource::<Player>()
    }

    pub fn relative_xy(&self, view: &Rect, xy: ICoord) -> Option<ICoord> {
        if !view.contains(xy) {
            return None;
        }

        Some((xy.0 - view.x, xy.1 - view.y))
    }

    pub fn move_player(&mut self, dir: Direction) -> Response {
        let player_xy = self.player().xy;
        let new_xy = (player_xy.0 + dir.dx(), player_xy.1 + dir.dy());

        if !self.cell_map().0.in_bounds(new_xy) {
            return Response::Fail(Some(String::from("Blocked!")));
        }

        if self.actor_map().0.get_xy(new_xy).is_none()
            && self.cell_map().0.get_xy(new_xy).passable()
        {
            self.player_mut().xy = new_xy;
            self.refresh_player_fov();
            Response::Success(None)
        } else {
            Response::Fail(Some(String::from("Blocked!")))
        }
    }

    pub fn teleport_player(&mut self, xy: ICoord) -> Response {
        if !self.cell_map().0.in_bounds(xy) {
            return Response::Fail(Some(String::from("Blocked!")));
        }

        if self.actor_map().0.get_xy(xy).is_none() && self.cell_map().0.get_xy(xy).passable() {
            self.player_mut().xy = xy;
            self.refresh_player_fov();
            Response::Success(None)
        } else {
            Response::Fail(Some(String::from("Blocked!")))
        }
    }

    pub fn dispatch(&mut self) {
        let mut chase_player_system = ChasePlayerSystem {};
        chase_player_system.run_now(&self.world);

        let mut flee_player_system = FleePlayerSystem {};
        flee_player_system.run_now(&self.world);

        self.world.maintain();
        self.refresh_navigation_maps();
    }
}
