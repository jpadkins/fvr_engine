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
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::actor::*;
use crate::components::*;
use crate::server::*;
use crate::zone::*;

pub struct GoalsSystem;

impl<'a> System<'a> for GoalsSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Zone>,
        Read<'a, LazyUpdate>,
        ReadExpect<'a, IntentionsVec>,
        WriteStorage<'a, IsActor>,
        WriteStorage<'a, HasGoals>,
    );

    fn run(
        &mut self,
        (mut zone, updater, intentions, mut is_actor, mut has_goals): Self::SystemData,
    ) {
        for (a, h) in (&mut is_actor, &mut has_goals).join() {
            // If there are no current goals, populate some from the actor's intention.
            if h.goals.is_empty() {
                intentions[a.0.intention].as_ref().bored(&mut a.0, &zone, &mut h.goals);
            }

            // Update the current goal.
            let state = h.goals.last_mut().unwrap().update(&mut a.0, &mut zone, &updater);

            // Pop the goal if it is complete or failed.
            // TODO: Add handling for failure.
            if state == GoalState::Complete || state == GoalState::Failed {
                let _ = h.goals.pop();
            }
        }
    }
}

pub struct MoveSystem;

impl<'a> System<'a> for MoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData =
        (WriteExpect<'a, Zone>, WriteStorage<'a, IsActor>, WriteStorage<'a, WantsToMove>);

    fn run(&mut self, (mut zone, mut is_actor, mut wants_to_move): Self::SystemData) {
        for (a, m) in (&mut is_actor, &mut wants_to_move).join() {
            let new_xy = (a.0.xy.0 + m.direction.dx(), a.0.xy.1 + m.direction.dy());

            // Is the new position blocked?
            if zone.is_blocked(new_xy) {
                a.0.navigation.stationary += 1;
                continue;
            }

            // Is the player occupying the new position?
            if new_xy == zone.player_xy {
                a.0.navigation.stationary += 1;
                continue;
            }

            // The new position is available - update the actor and the actor map.
            *zone.actor_map.get_xy_mut(new_xy) = zone.actor_map.get_xy_mut(a.0.xy).take();

            a.0.navigation.prev_weight = Some(m.weight);
            a.0.navigation.stationary = 0;
            a.0.xy = new_xy;

            // If the entity is the player, also update the player xy.
            if a.0.entity == zone.player_entity {
                zone.player_xy = new_xy;
            }
        }

        // Clear all pending events.
        wants_to_move.clear();
    }
}
