use anyhow::Result;
use specs::shred::Fetch;

use fvr_engine_core::{prelude::*, xy_iter};

use crate::zone::*;

pub enum ClientRequest {
    Move(Direction),
    Teleport(UCoord),
    Wait,
}

pub enum ServerResponse {
    Fail(Option<String>),
    Success(Option<String>),
}

pub struct Server {
    zone: Zone,
}

impl Server {
    pub fn new() -> Result<Self> {
        // TODO
        let zone = Zone::new((55, 33))?;
        Ok(Self { zone })
    }

    pub fn reload(&mut self) -> Result<()> {
        *self = Self::new()?;

        Ok(())
    }

    pub fn blit_zone<M>(&self, terminal: &mut M, src: &Rect, dst: UCoord)
    where
        M: Map2d<Tile>,
    {
        let cells = self.zone.cell_map();
        let actors = self.zone.actor_map();

        xy_iter!(x, y, src.width as u32, src.height as u32, {
            let xy = (dst.0 + x, dst.1 + y);

            if let Some(actor) = actors.0.get_xy(xy) {
                // Actors take precedence.
                *terminal.get_xy_mut(xy) = actor.thing.tile;
            } else if let Some(thing) = cells.0.get_xy(xy).things.last() {
                // Cells should always contain at least one thing.
                *terminal.get_xy_mut(xy) = thing.tile;
            } else {
                // Set tile to default to communicate missing data.
                *terminal.get_xy_mut(xy) = Tile::default();
            }
        });
    }

    pub fn player_xy(&self) -> UCoord {
        self.zone.player_xy().0
    }

    pub fn passable_map(&self) -> Fetch<PassableMap> {
        self.zone.passable_map()
    }

    pub fn request(&mut self, req: ClientRequest) -> Result<ServerResponse> {
        match req {
            ClientRequest::Move(dir) => {
                let result = self.zone.move_player(dir)?;
                self.zone.dispatch()?;

                if result {
                    Ok(ServerResponse::Success(None))
                } else {
                    Ok(ServerResponse::Fail(Some("Blocked!".into())))
                }
            }
            ClientRequest::Teleport(xy) => {
                let result = self.zone.teleport_player(xy)?;
                self.zone.dispatch()?;

                if result {
                    Ok(ServerResponse::Success(None))
                } else {
                    Ok(ServerResponse::Fail(Some("Blocked!".into())))
                }
            }
            ClientRequest::Wait => {
                self.zone.dispatch()?;
                Ok(ServerResponse::Success(None))
            }
        }
    }
}
