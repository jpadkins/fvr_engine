//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::widgets::rich_text_writer::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------

// Fancy border tiles.
static FANCY_CORNER_TILE: Tile = Tile {
    glyph: '■',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static FANCY_HORIZONTAL_THIN_TILE: Tile = Tile {
    glyph: '─',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static FANCY_HORIZONTAL_THICK_TILE: Tile = Tile {
    glyph: '═',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static FANCY_VERTICAL_THIN_TILE: Tile = Tile {
    glyph: '│',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static FANCY_VERTICAL_THICK_TILE: Tile = Tile {
    glyph: '║',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// Line border tiles.
static LINE_TOP_LEFT_TILE: Tile = Tile {
    glyph: '┌',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_TOP_RIGHT_TILE: Tile = Tile {
    glyph: '┐',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_BOTTOM_LEFT_TILE: Tile = Tile {
    glyph: '└',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_BOTTOM_RIGHT_TILE: Tile = Tile {
    glyph: '┘',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_HORIZONTAL_TILE: Tile = Tile {
    glyph: '─',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_VERTICAL_TILE: Tile = Tile {
    glyph: '│',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// Double line border tiles.
static DOUBLE_LINE_TOP_LEFT_TILE: Tile = Tile {
    glyph: '╔',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_TOP_RIGHT_TILE: Tile = Tile {
    glyph: '╗',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_BOTTOM_LEFT_TILE: Tile = Tile {
    glyph: '╚',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_BOTTOM_RIGHT_TILE: Tile = Tile {
    glyph: '╝',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_HORIZONTAL_TILE: Tile = Tile {
    glyph: '═',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_VERTICAL_TILE: Tile = Tile {
    glyph: '║',
    layout: TileLayout::Text,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// Simple line border tile.
static SIMPLE_LINE_TILE: Tile = Tile {
    glyph: '#',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// System line border tile.
static SYSTEM_LINE_TILE: Tile = Tile {
    glyph: ' ',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: PaletteColor::White.const_into(),
    foreground_color: TileColor::TRANSPARENT,
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// Format settings for frame text.
static TEXT_FORMAT_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: None,
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: None,
    outline_color: None,
    background_opacity: None,
    foreground_opacity: None,
    outline_opacity: None,
};

//-------------------------------------------------------------------------------------------------
// Positions for text along the frame.
//-------------------------------------------------------------------------------------------------
pub enum FrameTextPosition {
    // Top of the frame, left-aligned.
    TopLeft,
    // Top of the frame, right-aligned.
    TopRight,
    // Bottom of the frame, left-aligned.
    BottomLeft,
    // Bottom of the frame, right-aligned.
    BottomRight,
}

//-------------------------------------------------------------------------------------------------
// Possible styles for the frame.
//-------------------------------------------------------------------------------------------------
pub enum FrameStyle {
    // A fancy border that has the appearance of a chain with square corners.
    // Works best with odd length edges.
    Fancy,
    // A broken, single line border.
    Line,
    // A broken, single line border with blocks for corners.
    LineBlockCorner,
    // A broken, double line border.
    DoubleLine,
    // A broken, double line border with blocks for corners.
    DoubleLineBlockCorner,
    // A simple border that is comprised of number symbols (#).
    Simple,
    // A thick, solid border.
    System,
}

//-------------------------------------------------------------------------------------------------
// Frame handles drawing decorated rects. Used by other widgets.
//-------------------------------------------------------------------------------------------------
pub struct Frame {
    // Origin of the frame when drawing.
    pub origin: ICoord,
    // Dimensions of the area inside the frame.
    pub inner_dimensions: ICoord,
    // Style of the frame.
    pub style: FrameStyle,
    // Optional top-left text.
    pub top_left_text: Option<String>,
    // Optional top-left text.
    pub top_right_text: Option<String>,
    // Optional bottom-left text.
    pub bottom_left_text: Option<String>,
    // Optional bottom-right text.
    pub bottom_right_text: Option<String>,
}

impl Frame {
    //---------------------------------------------------------------------------------------------
    // Creates a new frame.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: ICoord, inner_dimensions: ICoord, style: FrameStyle) -> Self {
        Self {
            origin,
            inner_dimensions,
            style,
            top_left_text: None,
            top_right_text: None,
            bottom_left_text: None,
            bottom_right_text: None,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the origin of the frame.
    //---------------------------------------------------------------------------------------------
    pub fn origin(&self) -> ICoord {
        self.origin
    }

    //---------------------------------------------------------------------------------------------
    // Returns the width of the frame.
    //---------------------------------------------------------------------------------------------
    pub fn width(&self) -> i32 {
        self.inner_dimensions.0 + 1
    }

    //---------------------------------------------------------------------------------------------
    // Returns the height of the frame.
    //---------------------------------------------------------------------------------------------
    pub fn height(&self) -> i32 {
        self.inner_dimensions.1 + 1
    }

    //---------------------------------------------------------------------------------------------
    // Returns the inner dimensions of the frame.
    //---------------------------------------------------------------------------------------------
    pub fn inner_dimensions(&self) -> ICoord {
        self.inner_dimensions
    }

    //---------------------------------------------------------------------------------------------
    // Clears all of the frame's text.
    //---------------------------------------------------------------------------------------------
    pub fn clear_text(&mut self) {
        self.top_left_text = None;
        self.top_right_text = None;
        self.bottom_left_text = None;
        self.bottom_right_text = None;
    }

    //---------------------------------------------------------------------------------------------
    // Centers the frame within a Map2dView.
    //---------------------------------------------------------------------------------------------
    pub fn center<M>(&mut self, map: &M)
    where
        M: Map2dView,
    {
        // Offset of 2 to account for the border tiles.
        self.origin = (
            Misc::centered_origin(self.inner_dimensions.0 - 2, map.width()),
            Misc::centered_origin(self.inner_dimensions.1 - 2, map.height()),
        );
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for drawing the fancy border corners.
    //---------------------------------------------------------------------------------------------
    fn draw_fancy_corners<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Top-left corner.
        *map.get_xy_mut(self.origin) = FANCY_CORNER_TILE;

        // Top-right corner.
        *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, self.origin.1)) =
            FANCY_CORNER_TILE;

        // Bottom-right corner.
        *map.get_xy_mut((
            self.origin.0 + self.inner_dimensions.0 + 1,
            self.origin.1 + self.inner_dimensions.1 + 1,
        )) = FANCY_CORNER_TILE;

        // Bottom-left corner.
        *map.get_xy_mut((self.origin.0, self.origin.1 + self.inner_dimensions.1 + 1)) =
            FANCY_CORNER_TILE;
    }

    //---------------------------------------------------------------------------------------------
    // Draws a fancy border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_fancy_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Draw the corners.
        self.draw_fancy_corners(map);

        // Horizontal border.
        for x in (self.origin.0 + 1)..(self.origin.0 + self.inner_dimensions.0 + 1) {
            if (x - (self.origin.0 + 1)) % 2 == 0 {
                *map.get_xy_mut((x, self.origin.1)) = FANCY_HORIZONTAL_THICK_TILE;
                *map.get_xy_mut((x, self.origin.1 + self.inner_dimensions.1 + 1)) =
                    FANCY_HORIZONTAL_THICK_TILE;
            } else {
                *map.get_xy_mut((x, self.origin.1)) = FANCY_HORIZONTAL_THIN_TILE;
                *map.get_xy_mut((x, self.origin.1 + self.inner_dimensions.1 + 1)) =
                    FANCY_HORIZONTAL_THIN_TILE;
            }
        }

        // Vertical border.
        for y in (self.origin.1 + 1)..(self.origin.1 + self.inner_dimensions.1 + 1) {
            if (y - (self.origin.1 + 1)) % 2 == 0 {
                *map.get_xy_mut((self.origin.0, y)) = FANCY_VERTICAL_THICK_TILE;
                *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, y)) =
                    FANCY_VERTICAL_THICK_TILE;
            } else {
                *map.get_xy_mut((self.origin.0, y)) = FANCY_VERTICAL_THIN_TILE;
                *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, y)) =
                    FANCY_VERTICAL_THIN_TILE;
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Draws a line border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_line_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Top-left corner.
        *map.get_xy_mut(self.origin) = LINE_TOP_LEFT_TILE;

        // Top-right corner.
        *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, self.origin.1)) =
            LINE_TOP_RIGHT_TILE;

        // Bottom-right corner.
        *map.get_xy_mut((
            self.origin.0 + self.inner_dimensions.0 + 1,
            self.origin.1 + self.inner_dimensions.1 + 1,
        )) = LINE_BOTTOM_RIGHT_TILE;

        // Bottom-left corner.
        *map.get_xy_mut((self.origin.0, self.origin.1 + self.inner_dimensions.1 + 1)) =
            LINE_BOTTOM_LEFT_TILE;

        // Horizontal border.
        for x in (self.origin.0 + 1)..(self.origin.0 + self.inner_dimensions.0 + 1) {
            *map.get_xy_mut((x, self.origin.1)) = LINE_HORIZONTAL_TILE;
            *map.get_xy_mut((x, self.origin.1 + self.inner_dimensions.1 + 1)) =
                LINE_HORIZONTAL_TILE;
        }

        // Vertical border.
        for y in (self.origin.1 + 1)..(self.origin.1 + self.inner_dimensions.1 + 1) {
            *map.get_xy_mut((self.origin.0, y)) = LINE_VERTICAL_TILE;
            *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, y)) = LINE_VERTICAL_TILE;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Draws a line (block corner) border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_line_block_corner_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Draw the line border.
        self.draw_line_border(map);

        // Set blocks on the corners.
        self.draw_fancy_corners(map);
    }

    //---------------------------------------------------------------------------------------------
    // Draws a double line border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_double_line_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Top-left corner.
        *map.get_xy_mut(self.origin) = DOUBLE_LINE_TOP_LEFT_TILE;

        // Top-right corner.
        *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, self.origin.1)) =
            DOUBLE_LINE_TOP_RIGHT_TILE;

        // Bottom-right corner.
        *map.get_xy_mut((
            self.origin.0 + self.inner_dimensions.0 + 1,
            self.origin.1 + self.inner_dimensions.1 + 1,
        )) = DOUBLE_LINE_BOTTOM_RIGHT_TILE;

        // Bottom-left corner.
        *map.get_xy_mut((self.origin.0, self.origin.1 + self.inner_dimensions.1 + 1)) =
            DOUBLE_LINE_BOTTOM_LEFT_TILE;

        // Horizontal border.
        for x in (self.origin.0 + 1)..(self.origin.0 + self.inner_dimensions.0 + 1) {
            *map.get_xy_mut((x, self.origin.1)) = DOUBLE_LINE_HORIZONTAL_TILE;
            *map.get_xy_mut((x, self.origin.1 + self.inner_dimensions.1 + 1)) =
                DOUBLE_LINE_HORIZONTAL_TILE;
        }

        // Vertical border.
        for y in (self.origin.1 + 1)..(self.origin.1 + self.inner_dimensions.1 + 1) {
            *map.get_xy_mut((self.origin.0, y)) = DOUBLE_LINE_VERTICAL_TILE;
            *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, y)) =
                DOUBLE_LINE_VERTICAL_TILE;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Draws a double line (block corner) border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_double_line_block_corner_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Draw the double line border.
        self.draw_double_line_border(map);

        // Set blocks on the corners.
        self.draw_fancy_corners(map);
    }

    //---------------------------------------------------------------------------------------------
    // Draws a simple border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_simple_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Top-left corner.
        *map.get_xy_mut(self.origin) = SIMPLE_LINE_TILE;

        // Top-right corner.
        *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, self.origin.1)) =
            SIMPLE_LINE_TILE;

        // Bottom-right corner.
        *map.get_xy_mut((
            self.origin.0 + self.inner_dimensions.0 + 1,
            self.origin.1 + self.inner_dimensions.1 + 1,
        )) = SIMPLE_LINE_TILE;

        // Bottom-left corner.
        *map.get_xy_mut((self.origin.0, self.origin.1 + self.inner_dimensions.1 + 1)) =
            SIMPLE_LINE_TILE;

        // Horizontal border.
        for x in (self.origin.0 + 1)..(self.origin.0 + self.inner_dimensions.0 + 1) {
            *map.get_xy_mut((x, self.origin.1)) = SIMPLE_LINE_TILE;
            *map.get_xy_mut((x, self.origin.1 + self.inner_dimensions.1 + 1)) = SIMPLE_LINE_TILE;
        }

        // Vertical border.
        for y in (self.origin.1 + 1)..(self.origin.1 + self.inner_dimensions.1 + 1) {
            *map.get_xy_mut((self.origin.0, y)) = SIMPLE_LINE_TILE;
            *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, y)) = SIMPLE_LINE_TILE;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Draws a system border frame.
    //---------------------------------------------------------------------------------------------
    fn draw_system_border<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Top-left corner.
        *map.get_xy_mut(self.origin) = SYSTEM_LINE_TILE;

        // Top-right corner.
        *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, self.origin.1)) =
            SYSTEM_LINE_TILE;

        // Bottom-right corner.
        *map.get_xy_mut((
            self.origin.0 + self.inner_dimensions.0 + 1,
            self.origin.1 + self.inner_dimensions.1 + 1,
        )) = SYSTEM_LINE_TILE;

        // Bottom-left corner.
        *map.get_xy_mut((self.origin.0, self.origin.1 + self.inner_dimensions.1 + 1)) =
            SYSTEM_LINE_TILE;

        // Horizontal border.
        for x in (self.origin.0 + 1)..(self.origin.0 + self.inner_dimensions.0 + 1) {
            *map.get_xy_mut((x, self.origin.1)) = SYSTEM_LINE_TILE;
            *map.get_xy_mut((x, self.origin.1 + self.inner_dimensions.1 + 1)) = SYSTEM_LINE_TILE;
        }

        // Vertical border.
        for y in (self.origin.1 + 1)..(self.origin.1 + self.inner_dimensions.1 + 1) {
            *map.get_xy_mut((self.origin.0, y)) = SYSTEM_LINE_TILE;
            *map.get_xy_mut((self.origin.0 + self.inner_dimensions.0 + 1, y)) = SYSTEM_LINE_TILE;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Draws the frame onto a Map2d<Tile> without clearing the center.
    //---------------------------------------------------------------------------------------------
    pub fn draw<M>(&self, map: &mut M) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        // Draw the border.
        match self.style {
            FrameStyle::Fancy => self.draw_fancy_border(map),
            FrameStyle::Line => self.draw_line_border(map),
            FrameStyle::LineBlockCorner => self.draw_line_block_corner_border(map),
            FrameStyle::DoubleLine => self.draw_double_line_border(map),
            FrameStyle::DoubleLineBlockCorner => self.draw_double_line_block_corner_border(map),
            FrameStyle::Simple => self.draw_simple_border(map),
            FrameStyle::System => self.draw_system_border(map),
        }

        // Draw top-left text if populated.
        if let Some(top_left_text) = self.top_left_text.as_ref() {
            RichTextWriter::write_plain_with_settings(
                map,
                (self.origin.0 + 2, self.origin.1),
                top_left_text,
                &TEXT_FORMAT_SETTINGS,
            );
        }

        // Draw top-right text if populated.
        if let Some(top_right_text) = self.top_right_text.as_ref() {
            let stripped_len = RichTextWriter::stripped_len(top_right_text)?;
            RichTextWriter::write_plain_with_settings(
                map,
                (self.origin.0 + self.inner_dimensions.0 - stripped_len as i32, self.origin.1),
                top_right_text,
                &TEXT_FORMAT_SETTINGS,
            );
        }

        // Draw bottom-left text if populated.
        if let Some(bottom_left_text) = self.bottom_left_text.as_ref() {
            RichTextWriter::write_plain_with_settings(
                map,
                (self.origin.0 + 2, self.origin.1 + self.inner_dimensions.1 + 1),
                bottom_left_text,
                &TEXT_FORMAT_SETTINGS,
            );
        }

        // Draw bottom-right text if populated.
        if let Some(bottom_right_text) = self.bottom_right_text.as_ref() {
            let stripped_len = RichTextWriter::stripped_len(bottom_right_text)?;
            RichTextWriter::write_plain_with_settings(
                map,
                (
                    self.origin.0 + self.inner_dimensions.0 - stripped_len as i32,
                    self.origin.1 + self.inner_dimensions.1 + 1,
                ),
                bottom_right_text,
                &TEXT_FORMAT_SETTINGS,
            );
        }

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Draws the frame onto a Map2d<Tile> and sets the glyphs of the inner tiles to space.
    //---------------------------------------------------------------------------------------------
    pub fn draw_clear<M>(&self, map: &mut M) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        // Draw the frame.
        self.draw(map)?;

        // Clear the glyphs of the center tiles.
        for x in (self.origin.0 + 1)..(self.origin.0 + self.inner_dimensions.0 + 1) {
            for y in (self.origin.1 + 1)..(self.origin.1 + self.inner_dimensions.1 + 1) {
                map.get_xy_mut((x, y)).glyph = ' ';
            }
        }

        Ok(())
    }
}
