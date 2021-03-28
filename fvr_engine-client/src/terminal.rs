//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Terminal contains the state of the faux terminal and exposes an API for updating it.
//-------------------------------------------------------------------------------------------------
pub struct Terminal {
    tiles: GridMap<Tile>,
}

impl Terminal {
    //---------------------------------------------------------------------------------------------
    // Creates a new terminal.
    // (there should only ever be one, for now)
    //---------------------------------------------------------------------------------------------
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self { tiles: GridMap::<Tile>::new(width, height) }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the value of the tile at an xy coord with optional arguments.
    //---------------------------------------------------------------------------------------------
    pub fn update_tile_fields(
        &mut self,
        xy: (u32, u32),
        glyph: Option<char>,
        layout: Option<TileLayout>,
        style: Option<TileStyle>,
        size: Option<TileSize>,
        outlined: Option<bool>,
        background_color: Option<TileColor>,
        foreground_color: Option<TileColor>,
        outline_color: Option<TileColor>,
    ) {
        let tile = self.tiles.get_xy_mut(xy);

        if let Some(glyph) = glyph {
            tile.glyph = glyph;
        }
        if let Some(layout) = layout {
            tile.layout = layout;
        }
        if let Some(style) = style {
            tile.style = style;
        }
        if let Some(size) = size {
            tile.size = size;
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

    //---------------------------------------------------------------------------------------------
    // Iterates the xy coords in the terminal and their corresponding tiles.
    //---------------------------------------------------------------------------------------------
    pub fn coords_and_tiles_iter(&self) -> impl Iterator<Item = ((u32, u32), &Tile)> {
        (0..self.width())
            .cartesian_product(0..self.height())
            .map(move |xy| (xy, self.tiles.get_xy(xy)))
    }

    //---------------------------------------------------------------------------------------------
    // Randomizes the tiles in the terminal for debugging purposes.
    //---------------------------------------------------------------------------------------------
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();

        for tile in self.tiles.data_mut() {
            tile.glyph = *CP437_CHARS.choose(&mut rng).unwrap();
            tile.style = rng.gen();
            tile.outlined = rng.gen();
            tile.background_color = TileColor::rgb(25, 50, 75);
            tile.foreground_color = rng.gen();
            tile.outline_color = rng.gen();
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for Terminal.
//-------------------------------------------------------------------------------------------------
impl Map2dView for Terminal {
    type Type = Tile;

    fn width(&self) -> u32 {
        self.tiles.width()
    }

    fn height(&self) -> u32 {
        self.tiles.height()
    }

    fn data(&self) -> &[Self::Type] {
        self.tiles.data()
    }

    fn get(&self, index: usize) -> &Self::Type {
        self.tiles.get(index)
    }

    fn get_xy(&self, xy: (u32, u32)) -> &Self::Type {
        self.tiles.get_xy(xy)
    }

    fn get_point(&self, point: &Point) -> &Self::Type {
        self.tiles.get_point(point)
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dViewMut for Terminal.
//-------------------------------------------------------------------------------------------------
impl Map2dViewMut for Terminal {
    type Type = Tile;

    fn data_mut(&mut self) -> &mut [Self::Type] {
        self.tiles.data_mut()
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        self.tiles.get_mut(index)
    }

    fn get_xy_mut(&mut self, xy: (u32, u32)) -> &mut Self::Type {
        self.tiles.get_xy_mut(xy)
    }

    fn get_point_mut(&mut self, point: &Point) -> &mut Self::Type {
        self.tiles.get_point_mut(point)
    }
}
