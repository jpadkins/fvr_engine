//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use specs::prelude::*;
use specs::shred::{Fetch, FetchMut};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct Thing {
    pub tile: Tile,
}

#[derive(Clone, Debug, Default)]
pub struct Cell {
    pub things: Vec<Thing>,
}

pub struct Zone {
    world: specs::World,
}

pub type CellMap = GridMap<Cell>;

impl Zone {
    pub fn new(dimensions: UCoord) -> Self {
        let mut world = World::new();

        // Insert default resource state.
        world.insert(CellMap::new(dimensions));

        Self { world }
    }

    pub fn cells(&self) -> Fetch<CellMap> {
        self.world.read_resource::<CellMap>()
    }

    pub fn cells_mut(&mut self) -> FetchMut<CellMap> {
        self.world.write_resource::<CellMap>()
    }
}
