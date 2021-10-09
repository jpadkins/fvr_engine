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
use crate::thing::*;

//-------------------------------------------------------------------------------------------------
// Subset of actor struct containing navigation related state.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Default)]
pub struct ActorNavigation {
    // Navigation weight of previously occupied cell.
    pub weight: Option<f32>,
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
