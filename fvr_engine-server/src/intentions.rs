//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::actor::*;
use crate::components::*;
use crate::goals::*;
use crate::zone::*;

//-------------------------------------------------------------------------------------------------
// Alias for convenience.
//-------------------------------------------------------------------------------------------------
pub type Intentions = Vec<Box<dyn Intention + Send + Sync>>;

//-------------------------------------------------------------------------------------------------
// Intention is responsible for populating an actor's goal stack.
//-------------------------------------------------------------------------------------------------
pub trait Intention {
    //---------------------------------------------------------------------------------------------
    // Called when when the actor has no goals.
    //---------------------------------------------------------------------------------------------
    fn bored(&self, actor: &mut Actor, zone: &Zone, goals: &mut GoalStack);
}

//-------------------------------------------------------------------------------------------------
// A generic intention that does nothing but avoid the player.
//-------------------------------------------------------------------------------------------------
pub struct BasicAvoidPlayerIntention;

impl Intention for BasicAvoidPlayerIntention {
    //---------------------------------------------------------------------------------------------
    // Called when when the actor has no goals.
    //---------------------------------------------------------------------------------------------
    fn bored(&self, actor: &mut Actor, _zone: &Zone, goals: &mut GoalStack) {
        // Reset the actor state and push a goal.
        actor.navigation.weight = None;
        // actor.navigation.stationary = 0;
        goals.push(Box::new(AvoidPlayerGoal {}));
    }
}

//-------------------------------------------------------------------------------------------------
// A generic intention that does nothing but chase the player.
//-------------------------------------------------------------------------------------------------
pub struct BasicChasePlayerIntention;

impl Intention for BasicChasePlayerIntention {
    //---------------------------------------------------------------------------------------------
    // Called when when the actor has no goals.
    //---------------------------------------------------------------------------------------------
    fn bored(&self, actor: &mut Actor, _zone: &Zone, goals: &mut GoalStack) {
        // Reset the actor state and push a goal.
        actor.navigation.weight = None;
        // actor.navigation.stationary = 0;
        goals.push(Box::new(ChasePlayerGoal {}));
    }
}
