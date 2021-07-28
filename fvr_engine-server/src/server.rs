use fvr_engine_core::prelude::*;

use crate::zone::*;

pub enum Command {
    Wait,
    Move(Direction),
}

pub struct Server {
    zone: Zone,
}

impl Server {
    pub fn new() -> Self {
        // TODO
        Self { zone: Zone::new((55, 33)) }
    }

    pub fn blit_zone<M>(&self, terminal: &mut M, src: &Rect, dst: UCoord)
    where
        M: Map2d<Tile>,
    {
        let cells = self.zone.cells();

        for x in 0..src.width {
            for y in 0..src.height {
                let xy = (x as u32 + dst.0, y as u32 + dst.1);
                if let Some(thing) = cells.get_xy(xy).things.last() {
                    // Cells should always contain at least one thing.
                    *terminal.get_xy_mut(xy) = thing.tile;
                } else {
                    // Set tile to default to communicate missing data.
                    *terminal.get_xy_mut(xy) = Tile::default();
                }
            }
        }
    }
}
