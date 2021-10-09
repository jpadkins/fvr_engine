//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::thing::*;

//-------------------------------------------------------------------------------------------------
// Cell describes a single discrete point in the game world.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct Cell {
    // The things in the cell.
    pub things: Vec<Thing>,
}

impl Cell {
    //---------------------------------------------------------------------------------------------
    // Determine if the cell is passable.
    //---------------------------------------------------------------------------------------------
    pub fn passability(&self) -> Passability {
        if self.things.iter().all(|thing| thing.passability == Passability::Passable) {
            Passability::Passable
        } else {
            Passability::Blocked
        }
    }

    //---------------------------------------------------------------------------------------------
    // Determine if the cell is transparent.
    //---------------------------------------------------------------------------------------------
    pub fn transparency(&self) -> Transparency {
        if self.things.iter().all(|thing| thing.transparency == Transparency::Transparent) {
            Transparency::Transparent
        } else {
            Transparency::Opaque
        }
    }
}
