//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------

// Fancy border tiles.
static FANCY_CORNER_TILE: Tile = Tile {
    glyph: '■',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
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
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// Line border tiles.
static LINE_TOP_LEFT_TILE: Tile = Tile {
    glyph: '┌',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_TOP_RIGHT_TILE: Tile = Tile {
    glyph: '┐',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_BOTTOM_LEFT_TILE: Tile = Tile {
    glyph: '└',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_BOTTOM_RIGHT_TILE: Tile = Tile {
    glyph: '┘',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_HORIZONTAL_TILE: Tile = Tile {
    glyph: '─',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static LINE_VERTICAL_TILE: Tile = Tile {
    glyph: '│',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

// Double line border tiles.
static DOUBLE_LINE_TOP_LEFT_TILE: Tile = Tile {
    glyph: '╔',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_TOP_RIGHT_TILE: Tile = Tile {
    glyph: '╗',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_BOTTOM_LEFT_TILE: Tile = Tile {
    glyph: '╚',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_BOTTOM_RIGHT_TILE: Tile = Tile {
    glyph: '╝',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_HORIZONTAL_TILE: Tile = Tile {
    glyph: '═',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static DOUBLE_LINE_VERTICAL_TILE: Tile = Tile {
    glyph: '║',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::White.const_into(),
    outline_color: TileColor::TRANSPARENT,
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
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

//-------------------------------------------------------------------------------------------------
// Possible styles for the frame.
//-------------------------------------------------------------------------------------------------
enum FrameStyle {
    // A fancy border that has the appearance of a chain with square corners.
    Fancy,
    // A broken, single line border.
    Line,
    // A broken, double line border.
    DoubleLine,
    // A simple border that is comprised of number symbols (#).
    Simple,
    // A thick, solid border.
    System,
}

//-------------------------------------------------------------------------------------------------
// Frame handles drawing decorated rects. Used by other widgets.
//-------------------------------------------------------------------------------------------------
struct Frame {
    pub origin: (u32, u32),
    pub inner_dimensions: (u32, u32),
    pub style: FrameStyle,
    pub top_left_text: Option<String>,
    pub top_right_text: Option<String>,
    pub bottom_left_text: Option<String>,
    pub bottom_right_text: Option<String>,
}

impl Frame {
    pub fn new(origin: (u32, u32), inner_dimensions: (u32, u32), style: FrameStyle) -> Self {
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
}
