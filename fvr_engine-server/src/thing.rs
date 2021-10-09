//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Thing describes a thing in the game world.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Default)]
pub struct Thing {
    // Passability of the thing.
    pub passability: Passability,
    // Transparency of the thing.
    pub transparency: Transparency,
    // Visual tile of the thing.
    pub tile: Tile,
}
