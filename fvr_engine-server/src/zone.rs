//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::sync::{Arc, Mutex};

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;
use rand::prelude::*;
use specs::prelude::*;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::{map2d_iter_index_mut, prelude::*, xy_tuple_iter};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::actor::*;
use crate::cell::*;
use crate::components::*;
use crate::server::*;
use crate::thing::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------

// TODO: Remove.
static TREE_THING: Thing = Thing {
    tile: Tile {
        glyph: '♣',
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
    passability: Passability::Blocked,
    transparency: Transparency::Opaque,
};

// TODO: Remove.
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
    passability: Passability::Passable,
    transparency: Transparency::Transparent,
};

// TODO: Remove.
static AVOID_MOB_THING: Thing = Thing {
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
    passability: Passability::Blocked,
    transparency: Transparency::Transparent,
};

// TODO: Remove.
static CHASE_MOB_THING: Thing = Thing {
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
    passability: Passability::Blocked,
    transparency: Transparency::Transparent,
};

// TODO: Remove.
static PLAYER_THING: Thing = Thing {
    tile: Tile {
        glyph: '@',
        layout: TileLayout::Center,
        style: TileStyle::Regular,
        size: TileSize::Normal,
        outlined: false,
        background_color: TileColor::TRANSPARENT,
        foreground_color: PaletteColor::White.const_into(),
        outline_color: TileColor::TRANSPARENT,
        background_opacity: 1.0,
        foreground_opacity: 1.0,
        outline_opacity: 1.0,
    },
    passability: Passability::Blocked,
    transparency: Transparency::Transparent,
};

//-------------------------------------------------------------------------------------------------
// Helper struct to store pathing related state for a cell.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
pub struct PathingProperties {
    pub dijkstra_state: DijkstraState,
    pub transparency: Transparency,
}

impl PathingProperties {
    //---------------------------------------------------------------------------------------------
    // Returns whether the pathing properties are passable.
    //---------------------------------------------------------------------------------------------
    fn passable(&self) -> bool {
        Into::<Passability>::into(self.dijkstra_state) == Passability::Passable
    }
}

// From impls to allow pathing properties to be used by different struct APIs.
impl From<PathingProperties> for DijkstraState {
    fn from(pathing: PathingProperties) -> Self {
        pathing.dijkstra_state
    }
}
impl From<PathingProperties> for Passability {
    fn from(pathing: PathingProperties) -> Self {
        pathing.dijkstra_state.into()
    }
}
impl From<PathingProperties> for Transparency {
    fn from(pathing: PathingProperties) -> Self {
        pathing.transparency
    }
}

//-------------------------------------------------------------------------------------------------
// Zone describes a descrete chunk of the game world.
//-------------------------------------------------------------------------------------------------
pub struct Zone {
    // Dimensions of the zone.
    pub dimensions: ICoord,
    // Position of the player in the zone.
    pub player_xy: ICoord,
    // Entity of the player in the world.
    pub player_entity: Entity,
    // Fov of the player.
    pub player_fov: Fov,
    // Grid of the zone's cells.
    pub cell_map: GridMap<Cell>,
    // Grid of the zone's actors.
    pub actor_map: GridMap<Option<SharedActor>>,
    // Navigation map pointing away from the player.
    pub avoid_map: DijkstraMap,
    // Navigation map pointing towards the player.
    pub chase_map: DijkstraMap,
    // Shared pathing propertie.
    pub pathing: GridMap<PathingProperties>,
}

impl Zone {
    //---------------------------------------------------------------------------------------------
    // TODO: Remove.
    //---------------------------------------------------------------------------------------------
    pub fn generate_dummy_map(&mut self) {
        let mut rng = thread_rng();
        const TREE_CHANCE: u8 = 15;

        // Iterate over the map, setting each cell to either grass or a tree.
        map2d_iter_index_mut!(self.cell_map, x, y, item, {
            if rng.gen::<u8>() % TREE_CHANCE == 0 {
                *item = Cell { things: vec![TREE_THING] };
            } else {
                *item = Cell { things: vec![GRASS_THING] };
            }
        });

        // Ensure the player's cell is passable.
        *self.cell_map.get_xy_mut(self.player_xy) = Cell { things: vec![GRASS_THING] };
    }

    //---------------------------------------------------------------------------------------------
    // TODO: Remove.
    //---------------------------------------------------------------------------------------------
    pub fn generate_dummy_mobs(&mut self, world: &mut World) -> Result<()> {
        let mut rng = thread_rng();
        const AVOID_MOB_COUNT: u8 = 50;
        const CHASE_MOB_COUNT: u8 = 20;

        // Populate map randomly with actors that avoid the player.
        for _ in 0..AVOID_MOB_COUNT {
            // Find a random coord.
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            // Check if it is available.
            if xy == self.player_xy
                || self.actor_map.get_xy(xy).is_some()
                || !self.pathing.get_xy(xy).passable()
            {
                continue;
            }

            // Create the avoid mob and insert it into the world and the actor map.
            let entity = world.create_entity().build();
            let actor = Arc::new(Mutex::new(Actor {
                entity,
                thing: AVOID_MOB_THING,
                xy,
                navigation: ActorNavigation::default(),
                stats: rng.gen(),
                behavior: 0,
                intention: BASIC_AVOID_PLAYER_INDEX,
            }));

            world.write_component::<IsActor>().insert(entity, IsActor(actor.clone()))?;
            world.write_component::<HasGoals>().insert(entity, HasGoals::default())?;
            *self.actor_map.get_xy_mut(xy) = Some(actor);
        }

        // Populate map randomly with actors that chase the player.
        for _ in 0..CHASE_MOB_COUNT {
            // Find a random coord.
            let xy = (rng.gen_range(0..self.dimensions.0), rng.gen_range(0..self.dimensions.1));

            // Check if it is available.
            if xy == self.player_xy
                || self.actor_map.get_xy(xy).is_some()
                || !self.pathing.get_xy(xy).passable()
            {
                continue;
            }

            // Create the chase mob and insert it into the world and the actor map.
            let entity = world.create_entity().build();
            let actor = Arc::new(Mutex::new(Actor {
                entity,
                thing: CHASE_MOB_THING,
                xy,
                navigation: ActorNavigation::default(),
                stats: rng.gen(),
                behavior: 0,
                intention: BASIC_CHASE_PLAYER_INDEX,
            }));

            world.write_component::<IsActor>().insert(entity, IsActor(actor.clone()))?;
            world.write_component::<HasGoals>().insert(entity, HasGoals::default())?;
            *self.actor_map.get_xy_mut(xy) = Some(actor);
        }

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Refreshes the state of the navigation related maps.
    //---------------------------------------------------------------------------------------------
    fn refresh_navigation_maps(&mut self) {
        // Refresh the path properties map.
        xy_tuple_iter!(x, y, self.dimensions, {
            // Each coord starts out as passable and transparent.
            let mut passability = Passability::Passable;
            let mut transparency = Transparency::Transparent;

            // Check properties from any present actors.
            if let Some(actor) = self.actor_map.get_xy((x, y)) {
                // Aquire ref to actor.
                let actor = actor.as_ref().lock().expect("Failed to lock actor mutex.");

                // Treat actors who have not moved in two rounds as obstacles.
                if actor.navigation.stationary >= 2
                    || actor.thing.passability != Passability::Passable
                {
                    passability = Passability::Blocked;
                }
                transparency = actor.thing.transparency;
            }

            // Check properties from the cell.
            let cell = self.cell_map.get_xy((x, y));
            if cell.passability() != Passability::Passable {
                passability = Passability::Blocked;
            }
            if cell.transparency() != Transparency::Transparent {
                transparency = Transparency::Opaque;
            }

            // Update the pathing properties.
            let pathing = self.pathing.get_xy_mut((x, y));
            pathing.dijkstra_state = passability.into();
            pathing.transparency = transparency;
        });

        // Cache the path properties and set player position as the current goal.
        let path_props = *self.pathing.get_xy(self.player_xy);
        self.pathing.get_xy_mut(self.player_xy).dijkstra_state = DIJKSTRA_DEFAULT_GOAL;

        // Caluclate the chase map.
        self.chase_map.calculate_thin(&self.pathing);

        // Calculate the avoid map using the max xy of the chase map.
        let highest_xy = self.chase_map.highest_xy();

        // Reset the path properties.
        *self.pathing.get_xy_mut(self.player_xy) = path_props;

        if let Some(xy) = highest_xy {
            // If a path exists to the player then use combined flee pathing.
            self.pathing.get_xy_mut(xy).dijkstra_state = DIJKSTRA_DEFAULT_GOAL;

            // Calculate the flee map with the highest chase map xy as the goal.
            self.avoid_map.calculate_thin(&self.pathing);

            // Find the highest weight in the chase map.
            let highest_weight = self.chase_map.get_xy(xy).expect("No highest weight!");

            // Modulate the entire flee map by some coefficient of the chase map.
            xy_tuple_iter!(x, y, self.dimensions, {
                let chase_weight = {
                    if let Some(weight) = self.chase_map.get_xy((x, y)) {
                        *weight
                    } else {
                        continue;
                    }
                };

                self.avoid_map.combine_xy((x, y), highest_weight - chase_weight);
            });
        } else {
            // Otherwise, reset the avoid map.
            self.avoid_map.calculate_thin(&self.pathing);
        }

        // Refresh the highest point in the avoid map.
        self.avoid_map.refresh_highest();
    }

    //---------------------------------------------------------------------------------------------
    // Refreshes the player fov.
    //---------------------------------------------------------------------------------------------
    fn refresh_player_fov(&mut self) {
        // TODO: Use a meaningful, dynamic value here.
        const PLAYER_FOV_DISTANCE: f32 = 30.0;
        self.player_fov.calculate_thin(self.player_xy, PLAYER_FOV_DISTANCE, &self.pathing);
    }

    //---------------------------------------------------------------------------------------------
    // TODO: Remove.
    //---------------------------------------------------------------------------------------------
    pub fn dummy(dimensions: ICoord, world: &mut World) -> Result<Self> {
        let mut actor_map = GridMap::new(dimensions);

        // Create and insert the player entity.
        let mut rng = thread_rng();
        let player_xy = (rng.gen_range(0..dimensions.0), rng.gen_range(0..dimensions.1));
        let player_entity = world.create_entity().build();
        let player_actor = Arc::new(Mutex::new(Actor {
            entity: player_entity,
            thing: PLAYER_THING,
            xy: player_xy,
            navigation: ActorNavigation::default(),
            stats: rng.gen(),
            behavior: usize::MAX,
            intention: usize::MAX,
        }));
        world.write_component::<IsActor>().insert(player_entity, IsActor(player_actor.clone()))?;
        *actor_map.get_xy_mut(player_xy) = Some(player_actor);

        // Generate dummy data for the zone.
        let mut zone = Self {
            dimensions,
            player_xy,
            player_entity,
            player_fov: Fov::new_thin(dimensions, Distance::Euclidean),
            cell_map: GridMap::new(dimensions),
            actor_map,
            avoid_map: DijkstraMap::new_thin(dimensions, Distance::Euclidean),
            chase_map: DijkstraMap::new_thin(dimensions, Distance::Euclidean),
            pathing: GridMap::new(dimensions),
        };

        zone.generate_dummy_map();
        zone.generate_dummy_mobs(world)?;
        zone.refresh();
        Ok(zone)
    }

    //---------------------------------------------------------------------------------------------
    // Refreshes the state of the zone. Should be called every turn.
    //---------------------------------------------------------------------------------------------
    pub fn refresh(&mut self) {
        self.refresh_navigation_maps();
        self.refresh_player_fov();
    }

    //---------------------------------------------------------------------------------------------
    // Determins whether a coord in the zone is passable.
    //---------------------------------------------------------------------------------------------
    pub fn is_blocked(&self, xy: ICoord) -> bool {
        // Is the position in bounds?
        if !self.cell_map.in_bounds(xy) {
            return true;
        }

        // Is the position passable?
        if !self.pathing.get_xy(xy).passable() {
            return true;
        }

        // Is the position occupied by an actor?
        if self.actor_map.get_xy(xy).is_some() {
            return true;
        }

        false
    }
}
