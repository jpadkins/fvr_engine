use sdl2::pixels::Color as SdlColor;
use serde_derive::{Deserialize, Serialize};

// Dummy external struct definition for serde.
#[derive(Deserialize, Serialize)]
#[serde(remote = "SdlColor")]
pub struct SdlColorDef {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

// Represents 8bit RGBA color.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TileColor(#[serde(with = "SdlColorDef")] pub SdlColor);

impl TileColor {
    pub const RED: TileColor = TileColor(SdlColor::RED);
    pub const BLUE: TileColor = TileColor(SdlColor::BLUE);
    pub const GREEN: TileColor = TileColor(SdlColor::GREEN);
    pub const WHITE: TileColor = TileColor(SdlColor::WHITE);
    pub const BLACK: TileColor = TileColor(SdlColor::BLACK);
    pub const TRANSPARENT: TileColor = TileColor(SdlColor::RGBA(255, 255, 255, 0));

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(SdlColor { r, g, b, a: std::u8::MAX })
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(SdlColor { r, g, b, a })
    }
}

// Describes the position of the glyph within the tile when rendered:
// Center   - centered within the tile
// Floor    - centered horizontally but aligned with the bottom of the tile vertically
// Text     - positioned based on font metrics (as though it was text)
// Exact    - positioned based on offset values (from the center position)
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileLayout {
    Center,
    Floor,
    Text,
    Exact((i32, i32)),
}

impl Default for TileLayout {
    fn default() -> Self {
        Self::Center
    }
}

// Describes a visual tile that can be rendered.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Tile {
    pub glyph: char,
    pub layout: TileLayout,
    pub outlined: bool,
    pub background_color: TileColor,
    pub foreground_color: TileColor,
    pub outline_color: TileColor,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            glyph: '?',
            layout: Default::default(),
            outlined: false,
            background_color: TileColor::BLUE,
            foreground_color: TileColor::RED,
            outline_color: TileColor::TRANSPARENT,
        }
    }
}

// Describes the location of a glyph within a font atlas, as well as positioning info.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GlyphMetric {
    pub codepoint: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: i32,
    pub y_offset: i32,
}

// (Intended to) describe a complete set of glyph metrics for all regular and outlined chars in Code Page 437.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontMetrics {
    pub regular: Vec<GlyphMetric>,
    pub outline: Vec<GlyphMetric>,
}
