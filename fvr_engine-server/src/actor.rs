//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use rand::distributions::{Distribution, Standard};
use rand::prelude::*;
use specs::prelude::*;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::components::*;
use crate::zone::*;

//-------------------------------------------------------------------------------------------------
// Thing describes a thing in the game world.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Default)]
pub struct Thing {
    pub passable: bool,
    pub tile: Tile,
}

//-------------------------------------------------------------------------------------------------
// Subset of actor struct containing navigation related state.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Default)]
pub struct ActorNavigation {
    // Navigation weight of previously occupied cell.
    pub prev_weight: Option<f32>,
    // Count of turns the actor has remained stationary.
    pub stationary: i32,
}

//-------------------------------------------------------------------------------------------------
// Subset of actor struct containing base ability statistics state.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Default)]
#[allow(non_snake_case)]
pub struct ActorStats {
    // Strength.
    pub STR: u8,
    // Dexterity.
    pub DEX: u8,
    // Constitution.
    pub CON: u8,
    // Wisdom.
    pub WIS: u8,
    // Intelligence.
    pub INT: u8,
    // Charisma.
    pub CHA: u8,
}

impl Distribution<ActorStats> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ActorStats {
        ActorStats {
            STR: rng.gen_range(0..=18),
            DEX: rng.gen_range(0..=18),
            CON: rng.gen_range(0..=18),
            WIS: rng.gen_range(0..=18),
            INT: rng.gen_range(0..=18),
            CHA: rng.gen_range(0..=18),
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Actor describes a dynamic game entity with a position, appearance, and AI.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug)]
pub struct Actor {
    // The SPECS entity of the actor.
    pub entity: Entity,
    // The actor's thing.
    pub thing: Thing,
    // Current position.
    pub xy: ICoord,
    // Additional navigation values.
    pub navigation: ActorNavigation,
    // The actor's base stats.
    pub stats: ActorStats,
    // Index of the actor's behavior.
    pub behavior: usize,
    // Index of the actor's intention.
    pub intention: usize,
}

impl Actor {}

//-------------------------------------------------------------------------------------------------
// Behavior describe how an actor should interact with different stimuli.
//-------------------------------------------------------------------------------------------------
pub trait Behavior {}

//-------------------------------------------------------------------------------------------------
// A generic behavior implementation (for testing).
//-------------------------------------------------------------------------------------------------
pub struct BasicBehavior;

impl Behavior for BasicBehavior {}

//-------------------------------------------------------------------------------------------------
// Intention is responsible for populating an actor's goal stack.
//-------------------------------------------------------------------------------------------------
pub trait Intention {
    //---------------------------------------------------------------------------------------------
    // Called when when the actor has no goals.
    //---------------------------------------------------------------------------------------------
    fn bored(&self, actor: &mut Actor, zone: &Zone, goals: &mut GoalsVec);
}

//-------------------------------------------------------------------------------------------------
// A generic intention that does nothing but avoid the player (for testing).
//-------------------------------------------------------------------------------------------------
pub struct BasicAvoidPlayerIntention;

impl Intention for BasicAvoidPlayerIntention {
    //---------------------------------------------------------------------------------------------
    // Called when when the actor has no goals.
    //---------------------------------------------------------------------------------------------
    fn bored(&self, actor: &mut Actor, _zone: &Zone, goals: &mut GoalsVec) {
        // Reset the actor state and push a goal.
        actor.navigation.prev_weight = None;
        // actor.navigation.stationary = 0;
        goals.push(Box::new(AvoidPlayerGoal {}));
    }
}

//-------------------------------------------------------------------------------------------------
// A generic intention that does nothing but chase the player (for testing).
//-------------------------------------------------------------------------------------------------
pub struct BasicChasePlayerIntention;

impl Intention for BasicChasePlayerIntention {
    //---------------------------------------------------------------------------------------------
    // Called when when the actor has no goals.
    //---------------------------------------------------------------------------------------------
    fn bored(&self, actor: &mut Actor, _zone: &Zone, goals: &mut GoalsVec) {
        // Reset the actor state and push a goal.
        actor.navigation.prev_weight = None;
        // actor.navigation.stationary = 0;
        goals.push(Box::new(ChasePlayerGoal {}));
    }
}

//-------------------------------------------------------------------------------------------------
// Describes the state of a goal.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GoalState {
    Complete,
    Failed,
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

        let (dir, weight) = best_dir.unwrap();

        // If the weight is not less than the previous weight, stay put.
        // if let Some(prev_weight) = actor.navigation.prev_weight {
        //     if prev_weight < weight {
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

        let (dir, weight) = best_dir.unwrap();

        // If the weight is not less than the previous weight, stay put.
        // if let Some(prev_weight) = actor.navigation.prev_weight {
        //     if prev_weight < weight {
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
