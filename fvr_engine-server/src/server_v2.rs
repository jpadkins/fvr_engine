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
use fvr_engine_core::{prelude::*, xy_iter};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::actor::*;
use crate::components::*;
use crate::systems::*;
use crate::zone_v2::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// TODO: Remove or find a way to populate dynamically.
pub const BASIC_AVOID_PLAYER_INDEX: usize = 0;
pub const BASIC_CHASE_PLAYER_INDEX: usize = 1;

//-------------------------------------------------------------------------------------------------
// Aliases for convenience.
//-------------------------------------------------------------------------------------------------
pub type BehaviorsVec = Vec<Box<dyn Behavior + Send + Sync>>;
pub type IntentionsVec = Vec<Box<dyn Intention + Send + Sync>>;

//-------------------------------------------------------------------------------------------------
// Enumerates the possible results returned from server actions.
//-------------------------------------------------------------------------------------------------
pub enum ServerResult {
    // The request failed.
    Fail,
    // The request succeeded.
    Success,
}

//-------------------------------------------------------------------------------------------------
// Server encapsulates all internal game logic and exposes an API for querying/manipulating it.
//-------------------------------------------------------------------------------------------------
pub struct ServerV2 {
    world: World,
}

impl ServerV2 {
    pub fn new() -> Result<Self> {
        // TODO: Remove - generate a dummy zone and insert it as a resource.
        let mut world = World::new();
        world.register::<IsActor>();
        world.register::<HasGoals>();
        world.register::<WantsToMove>();

        let zone = Zone::dummy((255, 255), &mut world)?;
        world.insert(zone);

        // Populate behaviors and intention vecs and insert them as resources.
        let behaviors: BehaviorsVec = vec![Box::new(BasicBehavior {})];

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let intentions: IntentionsVec = vec![
            Box::new(BasicAvoidPlayerIntention {}),
            Box::new(BasicChasePlayerIntention {})
        ];

        world.insert(behaviors);
        world.insert(intentions);

        Ok(Self { world })
    }

    pub fn zone(&self) -> Fetch<Zone> {
        self.world.fetch::<Zone>()
    }

    pub fn blit<M>(
        &self,
        terminal: &mut M,
        src: &Rect,
        dest_origin: ICoord,
        show_fov: bool,
    ) -> ICoord
    where
        M: Map2d<Tile>,
    {
        let zone = self.world.fetch::<Zone>();

        // Iterate through each of the visible tiles, updating them from the zone.
        xy_iter!(x, y, src.width, src.height, {
            // Calculate the adjusted coord.
            let src_xy = (src.x + x, src.y + y);
            let dst_xy = (dest_origin.0 + x, dest_origin.1 + y);

            // Get the tile to be updated.
            let tile = terminal.get_xy_mut(dst_xy);

            // Update the tile either with an actor, a thing, or a default tile.
            if let Some(actor) = zone.actor_map.get_xy(src_xy) {
                *tile = actor.thing.tile;
            } else if let Some(thing) = zone.cell_map.get_xy(src_xy).things.last() {
                *tile = thing.tile;
            } else {
                *tile = Tile::default();
            }

            // Optionally adjust for Fov.
            if show_fov {
                tile.foreground_opacity = *zone.player_fov.get_xy(src_xy);
            }
        });

        // Return the visible offset from the origin of the zone.
        (src.x, src.y)
    }

    pub fn blit_centered<M>(
        &self,
        terminal: &mut M,
        center: ICoord,
        dimensions: ICoord,
        dest_origin: ICoord,
        show_fov: bool,
    ) -> ICoord
    where
        M: Map2d<Tile>,
    {
        let zone = self.world.fetch::<Zone>();

        // Calculate the view rect.
        let mut rect = Rect::with_center(center, dimensions.0, dimensions.1);
        rect.fit_boundary(&Rect::new((0, 0), zone.dimensions.0, zone.dimensions.1));

        // Iterate through each of the visible tiles, updating them from the zone.
        xy_iter!(x, y, rect.width, rect.height, {
            // Calculate the adjusted coord.
            let src_xy = (rect.x + x, rect.y + y);
            let dst_xy = (dest_origin.0 + x, dest_origin.1 + y);

            // Get the tile to be updated.
            let tile = terminal.get_xy_mut(dst_xy);

            // Update the tile either with an actor, a thing, or a default tile.
            if let Some(actor) = zone.actor_map.get_xy(src_xy) {
                *tile = actor.thing.tile;
            } else if let Some(thing) = zone.cell_map.get_xy(src_xy).things.last() {
                *tile = thing.tile;
            } else {
                *tile = Tile::default();
            }

            // Optionally adjust for Fov.
            if show_fov {
                tile.foreground_opacity = *zone.player_fov.get_xy(src_xy);
            }
        });

        // Return the visible offset from the origin of the zone.
        (rect.x, rect.y)
    }

    pub fn blit_centered_on_player<M>(
        &self,
        terminal: &mut M,
        dimensions: ICoord,
        dest_origin: ICoord,
        show_fov: bool,
    ) -> ICoord
    where
        M: Map2d<Tile>,
    {
        let player_xy = self.world.fetch::<Zone>().player_xy;
        self.blit_centered(terminal, player_xy, dimensions, dest_origin, show_fov)
    }

    fn move_player_impl(&mut self, dir: Direction) -> Result<ServerResult> {
        // Calculate the tentative new player position.
        let zone = self.world.fetch::<Zone>();
        let new_xy = (zone.player_xy.0 + dir.dx(), zone.player_xy.1 + dir.dy());

        // Is the new position in bounds?
        if zone.is_blocked(new_xy) {
            return Ok(ServerResult::Fail);
        }

        // Otherwise, flag the player for moving and dispatch.
        let player_dex = zone.actor_map.get_xy(zone.player_xy).unwrap().stats.DEX;
        let comp = WantsToMove { direction: dir, weight: f32::MAX, priority: player_dex };
        self.world.write_component::<WantsToMove>().insert(zone.player_entity, comp)?;

        Ok(ServerResult::Success)
    }

    pub fn move_player(&mut self, dir: Direction) -> Result<ServerResult> {
        let result = self.move_player_impl(dir);
        self.tick();
        result
    }

    pub fn tick(&mut self) {
        // Run the systems.
        let mut goals_system = GoalsSystem {};
        goals_system.run_now(&self.world);
        self.world.maintain();

        let mut wants_to_move_system = MoveSystem {};
        wants_to_move_system.run_now(&self.world);
        self.world.maintain();

        // Refresh zone navigation maps and fov.
        let mut zone = self.world.fetch_mut::<Zone>();
        zone.refresh();
    }
}
