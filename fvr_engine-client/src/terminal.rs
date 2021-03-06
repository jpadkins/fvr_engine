use fvr_engine_core::prelude::*;

pub struct Terminal {
    tiles: GridMap<Tile>,
}

impl Terminal {
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = GridMap::<Tile>::new(width, height);
        Self { tiles }
    }
}
