use fvr_engine_core::{prelude::*, xy_iter};

use crate::zone::*;

pub enum Request {
    Move(Direction),
    Teleport(ICoord),
    Wait,
}

pub enum Response {
    Fail(Option<String>),
    Success(Option<String>),
}

pub struct Server {
    zone: Zone,
}

impl Server {
    pub fn new(zone: Zone) -> Self {
        Self { zone }
    }

    pub fn zone(&self) -> &Zone {
        &self.zone
    }

    pub fn zone_mut(&mut self) -> &mut Zone {
        &mut self.zone
    }

    pub fn blit<M>(&self, terminal: &mut M, src: &Rect, dst_origin: ICoord, fov: bool) -> ICoord
    where
        M: Map2d<Tile>,
    {
        let cells = self.zone.cell_map();
        let actors = self.zone.actor_map();

        xy_iter!(x, y, src.width, src.height, {
            let src_xy = (src.x + x, src.y + y);
            let dst_xy = (dst_origin.0 + x, dst_origin.1 + y);

            let tile = terminal.get_xy_mut(dst_xy);

            if let Some(actor) = actors.0.get_xy(src_xy) {
                *tile = actor.thing.tile;
            } else if let Some(thing) = cells.0.get_xy(src_xy).things.last() {
                *tile = thing.tile;
            } else {
                *tile = Tile::default();
            }

            if fov {
                tile.foreground_opacity = *self.zone.player().fov.get_xy(src_xy);
            }
        });

        if let Some(relative_xy) = self.zone.relative_xy(src, self.zone.player().xy) {
            let tile = terminal.get_xy_mut(relative_xy);
            tile.foreground_color = TileColor::WHITE;
            tile.glyph = '@';
        }

        (src.x, src.y)
    }

    pub fn blit_centered<M>(
        &self,
        terminal: &mut M,
        center: ICoord,
        dimensions: ICoord,
        dst_origin: ICoord,
        fov: bool,
    ) -> ICoord
    where
        M: Map2d<Tile>,
    {
        let cells = self.zone.cell_map();
        let actors = self.zone.actor_map();
        let mut rect = Rect::with_center(center, dimensions.0, dimensions.1);
        rect.fit_boundary(&Rect::new((0, 0), self.zone.width(), self.zone.height()));

        xy_iter!(x, y, rect.width, rect.height, {
            let src_xy = (rect.x + x, rect.y + y);
            let dst_xy = (dst_origin.0 + x, dst_origin.1 + y);

            let tile = terminal.get_xy_mut(dst_xy);

            if let Some(actor) = actors.0.get_xy(src_xy) {
                *tile = actor.thing.tile;
            } else if let Some(thing) = cells.0.get_xy(src_xy).things.last() {
                *tile = thing.tile;
            } else {
                *tile = Tile::default();
            }

            if fov {
                tile.foreground_opacity = *self.zone.player().fov.get_xy(src_xy);
            }
        });

        if let Some(relative_xy) = self.zone.relative_xy(&rect, self.zone.player().xy) {
            let tile = terminal.get_xy_mut(relative_xy);
            tile.foreground_color = TileColor::WHITE;
            tile.glyph = '@';
        }

        (rect.x, rect.y)
    }

    pub fn blit_player_centered<M>(
        &self,
        terminal: &mut M,
        dimensions: ICoord,
        dst_origin: ICoord,
        fov: bool,
    ) -> ICoord
    where
        M: Map2d<Tile>,
    {
        self.blit_centered(terminal, self.zone.player().xy, dimensions, dst_origin, fov)
    }

    pub fn handle(&mut self, request: Request) -> Response {
        match request {
            Request::Move(dir) => {
                let response = self.zone.move_player(dir);
                self.zone.dispatch();
                response
            }
            Request::Teleport(xy) => {
                let response = self.zone.teleport_player(xy);
                self.zone.dispatch();
                response
            }
            Request::Wait => {
                self.zone.dispatch();
                Response::Success(None)
            }
        }
    }
}
