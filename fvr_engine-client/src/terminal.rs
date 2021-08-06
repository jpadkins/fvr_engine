//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::{map2d_iter_mut, prelude::*};

//-------------------------------------------------------------------------------------------------
// Terminal contains the state of the faux terminal and exposes an API for updating it.
//-------------------------------------------------------------------------------------------------
pub struct Terminal {
    // Grid map of the terminal's tiles.
    tiles: GridMap<Tile>,
    // Opacity of the terminal.
    opacity: f32,
}

impl Terminal {
    //---------------------------------------------------------------------------------------------
    // Creates a new terminal.
    // (there should only ever be one, for now)
    //---------------------------------------------------------------------------------------------
    pub(crate) fn new(dimensions: ICoord) -> Self {
        Self { tiles: GridMap::new(dimensions), opacity: 1.0 }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the opacity of the entire terminal.
    //---------------------------------------------------------------------------------------------
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    //---------------------------------------------------------------------------------------------
    // Sets the opacity of the entire terminal clamped to (0.0, 1.0).
    //---------------------------------------------------------------------------------------------
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    //---------------------------------------------------------------------------------------------
    // Sets the terminal opacity to 0.0.
    //---------------------------------------------------------------------------------------------
    pub fn set_transparent(&mut self) {
        self.opacity = 0.0;
    }

    //---------------------------------------------------------------------------------------------
    // Sets the terminal opacity to 1.0.
    //---------------------------------------------------------------------------------------------
    pub fn set_opaque(&mut self) {
        self.opacity = 1.0;
    }

    //---------------------------------------------------------------------------------------------
    // Sets all tiles to default.
    //---------------------------------------------------------------------------------------------
    pub fn set_all_tiles_default(&mut self) {
        map2d_iter_mut!(self.tiles, tile, {
            *tile = Default::default();
        });
    }

    //---------------------------------------------------------------------------------------------
    // Sets all tiles to blank.
    //---------------------------------------------------------------------------------------------
    pub fn set_all_tiles_blank(&mut self) {
        map2d_iter_mut!(self.tiles, tile, {
            *tile = BLANK_TILE;
        });
    }

    //---------------------------------------------------------------------------------------------
    // Updates the value of the tile at an xy coord with optional arguments.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    pub fn update_tile(
        &mut self,
        xy: ICoord,
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
    // Updates the value of all tiles in the terminal with optional arguments.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    pub fn update_all_tiles(
        &mut self,
        glyph: Option<char>,
        layout: Option<TileLayout>,
        style: Option<TileStyle>,
        size: Option<TileSize>,
        outlined: Option<bool>,
        background_color: Option<TileColor>,
        foreground_color: Option<TileColor>,
        outline_color: Option<TileColor>,
        background_opacity: Option<f32>,
        foreground_opacity: Option<f32>,
        outline_opacity: Option<f32>,
    ) {
        map2d_iter_mut!(self.tiles, tile, {
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
            if let Some(background_opacity) = background_opacity {
                tile.background_opacity = background_opacity;
            }
            if let Some(foreground_opacity) = foreground_opacity {
                tile.foreground_opacity = foreground_opacity;
            }
            if let Some(outline_opacity) = outline_opacity {
                tile.outline_opacity = outline_opacity;
            }
        });
    }

    //---------------------------------------------------------------------------------------------
    // Iterates the xy coords in the terminal and their corresponding tiles.
    //---------------------------------------------------------------------------------------------
    pub fn coords_and_tiles_iter(&self) -> impl Iterator<Item = (ICoord, &Tile)> {
        (0..self.width())
            .cartesian_product(0..self.height())
            .map(move |xy| (xy, self.tiles.get_xy(xy)))
    }

    //---------------------------------------------------------------------------------------------
    // Randomizes the tiles in the terminal for debugging purposes.
    //---------------------------------------------------------------------------------------------
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();

        map2d_iter_mut!(self.tiles, tile, {
            tile.glyph = *CP437_CHARS.choose(&mut rng).unwrap();
            tile.style = rng.gen();
            tile.outlined = rng.gen();
            tile.background_color = TileColor::TRANSPARENT;
            tile.foreground_color = rng.gen();
            tile.outline_color = rng.gen();
        });
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for Terminal.
//-------------------------------------------------------------------------------------------------
impl Map2dView for Terminal {
    type Type = Tile;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> i32 {
        self.tiles.width()
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> i32 {
        self.tiles.height()
    }

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> ICoord {
        self.tiles.dimensions()
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type {
        self.tiles.get(index)
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: ICoord) -> &Self::Type {
        self.tiles.get_xy(xy)
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dViewMut for Terminal.
//-------------------------------------------------------------------------------------------------
impl Map2dViewMut for Terminal {
    type Type = Tile;

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        self.tiles.get_mut(index)
    }

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy_mut(&mut self, xy: ICoord) -> &mut Self::Type {
        self.tiles.get_xy_mut(xy)
    }
}
