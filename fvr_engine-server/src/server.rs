//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;
use specs::prelude::*;
use specs::shred::Fetch;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::{prelude::*, xy_iter};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::behavior::*;
use crate::components::*;
use crate::intentions::*;
use crate::systems::*;
use crate::zone::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// TODO: Remove or find a way to populate dynamically.
pub const BASIC_AVOID_PLAYER_INDEX: usize = 0;
pub const BASIC_CHASE_PLAYER_INDEX: usize = 1;

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
pub struct Server {
    // The specs world.
    world: World,
}

impl Server {
    //---------------------------------------------------------------------------------------------
    // Creates a new server. There should only ever be one.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Result<Self> {
        // TODO: Remove - generate a dummy zone and insert it as a resource.
        let mut world = World::new();
        world.register::<IsActor>();
        world.register::<HasGoals>();
        world.register::<WantsToMove>();

        let zone = Zone::dummy((255, 255), &mut world)?;
        world.insert(zone);

        // Populate behaviors and intention vecs and insert them as resources.
        let behaviors: Behaviors = vec![Box::new(BasicBehavior {})];

        #[rustfmt::skip]
        let intentions: Intentions = vec![
            Box::new(BasicAvoidPlayerIntention {}),
            Box::new(BasicChasePlayerIntention {})
        ];

        world.insert(behaviors);
        world.insert(intentions);

        Ok(Self { world })
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the current zone.
    //---------------------------------------------------------------------------------------------
    pub fn zone(&self) -> Fetch<Zone> {
        self.world.fetch::<Zone>()
    }

    //---------------------------------------------------------------------------------------------
    // Copies a section the visual state of current zone into a map2d.
    // Returns the offset from the origin of the zone of the blit.
    //---------------------------------------------------------------------------------------------
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
                let actor = actor.as_ref().lock().unwrap();
                *tile = actor.thing.tile;
            } else if let Some(thing) = zone.cell_map.get_xy(src_xy).things.last() {
                *tile = thing.tile;
            } else {
                *tile = Tile::default();
            }

            // Optionally adjust for Fov.
            if show_fov {
                tile.foreground_opacity = *zone.player_fov.get_xy(src_xy);
                tile.outline_opacity = tile.foreground_opacity;
            }
        });

        // Return the visible offset from the origin of the zone.
        (src.x, src.y)
    }

    //---------------------------------------------------------------------------------------------
    // Copies a section the visual state of current zone, centered on a coord, into a map2d.
    // Returns the offset from the origin of the zone of the blit.
    //---------------------------------------------------------------------------------------------
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
                let actor = actor.as_ref().lock().unwrap();
                *tile = actor.thing.tile;
            } else if let Some(thing) = zone.cell_map.get_xy(src_xy).things.last() {
                *tile = thing.tile;
            } else {
                *tile = Tile::default();
            }

            // Optionally adjust for Fov.
            if show_fov {
                tile.foreground_opacity = *zone.player_fov.get_xy(src_xy);
                tile.outline_opacity = tile.foreground_opacity;
            }
        });

        // Return the visible offset from the origin of the zone.
        (rect.x, rect.y)
    }

    //---------------------------------------------------------------------------------------------
    // Copies a section the visual state of current zone, centered on the player, into a map2d.
    // Returns the offset from the origin of the zone of the blit.
    //---------------------------------------------------------------------------------------------
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

    //---------------------------------------------------------------------------------------------
    // Tries to move the player in a direction. Returns the result.
    //---------------------------------------------------------------------------------------------
    fn try_move_player(&mut self, dir: Direction) -> Result<ServerResult> {
        // Calculate the tentative new player position.
        let zone = self.world.fetch::<Zone>();
        let new_xy = (zone.player_xy.0 + dir.dx(), zone.player_xy.1 + dir.dy());

        // Is the new position in bounds?
        if zone.is_blocked(new_xy) {
            return Ok(ServerResult::Fail);
        }

        // Otherwise, flag the player for moving and dispatch.
        let player_dex =
            zone.actor_map.get_xy(zone.player_xy).as_ref().unwrap().lock().unwrap().stats.DEX;
        let component = WantsToMove { direction: dir, weight: f32::MAX, priority: player_dex };
        self.world.write_component::<WantsToMove>().insert(zone.player_entity, component)?;

        Ok(ServerResult::Success)
    }

    //---------------------------------------------------------------------------------------------
    // Tries to move the player to a particular coord. Returns the result.
    //---------------------------------------------------------------------------------------------
    pub fn move_player(&mut self, dir: Direction) -> Result<ServerResult> {
        let result = self.try_move_player(dir);
        self.tick();
        result
    }

    //---------------------------------------------------------------------------------------------
    // Allow one "tick", or turn, to pass in the server.
    //---------------------------------------------------------------------------------------------
    pub fn tick(&mut self) {
        // Run the systems.
        let mut goals_system = GoalsSystem {};
        goals_system.run_now(&self.world);
        self.world.maintain();

        let mut wants_to_move_system = MoveSystem {};
        wants_to_move_system.run_now(&self.world);
        self.world.maintain();

        // Refresh zone navigation maps and fov.
        self.world.fetch_mut::<Zone>().refresh();
    }
}
