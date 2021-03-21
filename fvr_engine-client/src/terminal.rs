use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;

use fvr_engine_core::prelude::*;

pub struct Terminal {
    tiles: GridMap<Tile>,
    dirty_tiles: GridMap<bool>,
    dirty: bool,
}

impl Terminal {
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = GridMap::<Tile>::new(width, height);
        let mut dirty_tiles = GridMap::<bool>::new(width, height);

        // Every tile is dirty be default.
        dirty_tiles.data_mut().fill(true);

        Self { tiles, dirty_tiles, dirty: true }
    }

    pub fn width(&self) -> u32 {
        self.tiles.width()
    }

    pub fn height(&self) -> u32 {
        self.tiles.height()
    }

    pub fn tile(&self, x: u32, y: u32) -> &Tile {
        self.tiles.get_xy(x, y)
    }

    pub fn update_tile(&mut self, x: u32, y: u32, tile: &Tile) {
        // Set the tile to dirty.
        self.dirty = true;
        *self.dirty_tiles.get_xy_mut(x, y) = true;
        *self.tiles.get_xy_mut(x, y) = *tile;
    }

    pub fn update_tile_fields(
        &mut self,
        x: u32,
        y: u32,
        glyph: Option<char>,
        layout: Option<TileLayout>,
        outlined: Option<bool>,
        background_color: Option<TileColor>,
        foreground_color: Option<TileColor>,
        outline_color: Option<TileColor>,
    ) {
        // Set the tile to dirty.
        self.dirty = true;
        *self.dirty_tiles.get_xy_mut(x, y) = true;
        let tile = self.tiles.get_xy_mut(x, y);

        if let Some(glyph) = glyph {
            tile.glyph = glyph;
        }
        if let Some(layout) = layout {
            tile.layout = layout;
        }
        if let Some(outlined) = outlined {
            tile.outlined = outlined;
        }
        if let Some(background_color) = background_color {
            tile.background_color = background_color;
        }
        if let Some(foreground_color) = foreground_color {
            tile.foreground_color = foreground_color;
        }
        if let Some(outline_color) = outline_color {
            tile.outline_color = outline_color;
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_clean(&mut self) {
        self.dirty = false;
        self.dirty_tiles.data_mut().fill(false);
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
        self.dirty_tiles.data_mut().fill(true);
    }

    pub fn tiles_iter(&self) -> impl Iterator<Item = ((u32, u32), &Tile)> {
        (0..self.width())
            .cartesian_product(0..self.height())
            .map(move |(x, y)| ((x, y), self.tiles.get_xy(x, y)))
    }

    pub fn dirty_tiles_iter(&self) -> impl Iterator<Item = ((u32, u32), &Tile)> {
        (0..self.width())
            .cartesian_product(0..self.height())
            .filter(move |(x, y)| *self.dirty_tiles.get_xy(*x, *y))
            .map(move |(x, y)| ((x, y), self.tiles.get_xy(x, y)))
    }

    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();

        for tile in self.tiles.data_mut() {
            tile.glyph = *CP437_CHARS.choose(&mut rng).unwrap();
            tile.outlined = rng.gen_range(0..=3) == 3;
            tile.background_color = TileColor::rgb(25, 50, 75);
            tile.foreground_color =
                TileColor(sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen()));
            tile.outline_color =
                TileColor(sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen()));
        }

        self.set_dirty();
    }
}
