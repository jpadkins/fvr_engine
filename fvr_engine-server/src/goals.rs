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
use crate::actor::*;
use crate::components::*;
use crate::zone::*;

//-------------------------------------------------------------------------------------------------
// Describes the state of a goal.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GoalState {
    // The goal has been completed.
    Complete,
    // The goal is unable to be completed.
    Failed,
    // The goal is currently in progress.
    InProgress,
}

//-------------------------------------------------------------------------------------------------
// Goal describes a failable objective and is responsible for generating related tasks.
//-------------------------------------------------------------------------------------------------
pub trait Goal {
    //---------------------------------------------------------------------------------------------
    // Updates the goal, returning the new state.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        actor: &mut Actor,
        zone: &mut Zone,
        updater: &Read<LazyUpdate>,
    ) -> GoalState;
}

//-------------------------------------------------------------------------------------------------
// Avoid the player until reaching the furthest cell.
//-------------------------------------------------------------------------------------------------
pub struct AvoidPlayerGoal;

impl Goal for AvoidPlayerGoal {
    //---------------------------------------------------------------------------------------------
    // Updates the goal, returning the new state.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        actor: &mut Actor,
        zone: &mut Zone,
        updater: &Read<LazyUpdate>,
    ) -> GoalState {
        // Complete if the actor has been stationary for more than one turn.
        // if actor.navigation.stationary > 1 {
        //     return GoalState::Complete;
        // }
        // TODO: Complete after distance from player?

        // Get the best avoid direction.
        let best_dir = zone.avoid_map.best_direction(actor.xy);

        if best_dir.is_none() {
            actor.navigation.stationary += 1;
            return GoalState::InProgress;
        }

        let (dir, weight) = best_dir.expect("Unreachable.");

        // If the weight is not less than the previous weight, stay put.
        // if let Some(weight) = actor.navigation.weight {
        //     if weight < weight {
        //         actor.navigation.stationary += 1;
        //         return GoalState::InProgress;
        //     }
        // }

        // Flag the actor for moving.
        let component = WantsToMove { direction: dir, weight, priority: actor.stats.DEX };
        updater.insert(actor.entity, component);

        GoalState::InProgress
    }
}

//-------------------------------------------------------------------------------------------------
// Chase the player until reaching an adjacent cell.
//-------------------------------------------------------------------------------------------------
pub struct ChasePlayerGoal;

impl Goal for ChasePlayerGoal {
    //---------------------------------------------------------------------------------------------
    // Updates the goal, returning the new state.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        actor: &mut Actor,
        zone: &mut Zone,
        updater: &Read<LazyUpdate>,
    ) -> GoalState {
        // Complete if the actor occupies a neighboring coord to the player.
        // if Adjacency::is_neighbor(actor.xy, zone.player_xy) {
        //     return GoalState::Complete;
        // }

        // Get the best chase direction.
        let best_dir = zone.chase_map.best_direction(actor.xy);

        if best_dir.is_none() {
            actor.navigation.stationary += 1;
            return GoalState::InProgress;
        }

        let (dir, weight) = best_dir.expect("Unreachable.");

        // If the weight is not less than the previous weight, stay put.
        // if let Some(weight) = actor.navigation.weight {
        //     if weight < weight {
        //         actor.navigation.stationary += 1;
        //         return GoalState::InProgress;
        //     }
        // }

        // Flag the actor for moving.
        let component = WantsToMove { direction: dir, weight, priority: actor.stats.DEX };
        updater.insert(actor.entity, component);

        GoalState::InProgress
    }
}

//-------------------------------------------------------------------------------------------------
// Idle doing nothing for a set number of turns.
//-------------------------------------------------------------------------------------------------
pub struct IdleGoal {
    // Number of turns to idle.
    turns: i32,
}

impl IdleGoal {
    pub fn new(turns: i32) -> Self {
        Self { turns }
    }
}

impl Goal for IdleGoal {
    //---------------------------------------------------------------------------------------------
    // Updates the goal, returning the new state.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        _actor: &mut Actor,
        _zone: &mut Zone,
        _updater: &Read<LazyUpdate>,
    ) -> GoalState {
        // Check if the goal is complete.
        if self.turns == 0 {
            return GoalState::Complete;
        }

        // Otherwise, do nothing and decrement the turns.
        self.turns -= 1;
        GoalState::InProgress
    }
}

//-------------------------------------------------------------------------------------------------
// Move to a specific cell.
//-------------------------------------------------------------------------------------------------
pub struct MoveToGoal {
    // Target coord.
    pub xy: ICoord,
    // Path cache.
    pub path: Vec<ICoord>,
}

//-------------------------------------------------------------------------------------------------
// Roam the cells within a radius randomly.
//-------------------------------------------------------------------------------------------------
pub struct RoamGoal {
    // Number of turns to roam.
    pub turns: i32,
    // Boundary radius.
    pub radius: i32,
}
