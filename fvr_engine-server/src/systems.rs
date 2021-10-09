//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use specs::prelude::*;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::components::*;
use crate::goals::*;
use crate::intentions::*;
use crate::zone::*;

//-------------------------------------------------------------------------------------------------
// The goals system maintains the goals stack for an actor based on their intention.
//-------------------------------------------------------------------------------------------------
pub struct GoalsSystem;

impl<'a> System<'a> for GoalsSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Zone>,
        Read<'a, LazyUpdate>,
        ReadExpect<'a, Intentions>,
        WriteStorage<'a, IsActor>,
        WriteStorage<'a, HasGoals>,
    );

    //---------------------------------------------------------------------------------------------
    // Specs system run impl.
    // Ensures the actor's goals vec is populated from their intention, and cleans and goals that
    // are complete or failed.
    //---------------------------------------------------------------------------------------------
    fn run(
        &mut self,
        (mut zone, updater, intentions, mut is_actor, mut has_goals): Self::SystemData,
    ) {
        for (a, h) in (&mut is_actor, &mut has_goals).join() {
            // Aquire a mutable ref to the actor.
            let mut actor = a.0.as_ref().lock().expect("Failed to lock actor mutex.");

            // If there are no current goals, populate some from the actor's intention.
            if h.goals.is_empty() {
                intentions[actor.intention].as_ref().bored(&mut actor, &zone, &mut h.goals);
            }

            // Update the current goal.
            let state = match h.goals.last_mut() {
                Some(goal) => goal.update(&mut actor, &mut zone, &updater),
                None => panic!("Goal vec empty!"),
            };

            // Pop the goal if it is complete or failed.
            if state == GoalState::Complete || state == GoalState::Failed {
                let _ = h.goals.pop();
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------
// The move system handles actor movement within the zone.
//-------------------------------------------------------------------------------------------------
pub struct MoveSystem;

impl<'a> System<'a> for MoveSystem {
    #[allow(clippy::type_complexity)]
    type SystemData =
        (WriteExpect<'a, Zone>, WriteStorage<'a, IsActor>, WriteStorage<'a, WantsToMove>);

    //---------------------------------------------------------------------------------------------
    // Specs system run impl.
    // Ensures the actor's goals vec is populated from their intention, and cleans and goals that
    // are complete or failed.
    //---------------------------------------------------------------------------------------------
    fn run(&mut self, (mut zone, mut is_actor, mut wants_to_move): Self::SystemData) {
        for (a, m) in (&mut is_actor, &mut wants_to_move).join() {
            // Aquire a mutable ref to the actor.
            let mut actor = a.0.as_ref().lock().expect("Failed to lock actor mutex.");

            // Calculate the new xy.
            let new_xy = (actor.xy.0 + m.direction.dx(), actor.xy.1 + m.direction.dy());

            // Return if the position is blocked.
            if zone.is_blocked(new_xy) {
                actor.navigation.stationary += 1;
                continue;
            }

            // The new position is available - update the actor and the actor map.
            *zone.actor_map.get_xy_mut(new_xy) = zone.actor_map.get_xy_mut(actor.xy).take();

            actor.navigation.weight = Some(m.weight);
            actor.navigation.stationary = 0;
            actor.xy = new_xy;

            // If the entity is the player, also update the player xy.
            if actor.entity == zone.player_entity {
                zone.player_xy = new_xy;
            }
        }

        // Clear all components.
        wants_to_move.clear();
    }
}
