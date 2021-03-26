use rand::distributions::{Distribution, Standard};
use rand::Rng;
pub use sdl2::pixels::Color as SdlColor;
use serde_derive::{Deserialize, Serialize};

// Dummy external struct definition for serde.
#[derive(Deserialize, Serialize)]
#[serde(remote = "SdlColor")]
struct SdlColorDef {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

// Represents 8bit RGBA color.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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

// Impl random sampling of tile color.
impl Distribution<TileColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileColor {
        TileColor::rgb(rng.gen(), rng.gen(), rng.gen())
    }
}

// Describes the size of the tile's glyph when rendered.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileSize {
    // The glyph is proportional to half the size of a tile.
    Small = 0,
    // The glyph is proportional to the size of a tile.
    Normal,
    // The glyph is proportional to the size of 2x2 tiles.
    Big,
    // The glyph is proportional to the size of 4x4 tiles.
    Giant,
}
pub const TILE_SIZE_COUNT: usize = 4;

// Impl random sampling of enum.
impl Distribution<TileSize> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileSize {
        match rng.gen_range(0..=3) {
            0 => TileSize::Small,
            1 => TileSize::Normal,
            2 => TileSize::Big,
            _ => TileSize::Giant,
        }
    }
}

// Describes the style of the glyph within the tile when rendered.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileStyle {
    Regular = 0,
    Bold,
    Italic,
    BoldItalic,
}
pub const TILE_STYLE_COUNT: usize = 4;
pub const TILE_STYLE_NAMES: &[&str] = &["regular", "bold", "italic", "bold_italic"];

// Impl random sampling of enum.
impl Distribution<TileStyle> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileStyle {
        match rng.gen_range(0..=3) {
            0 => TileStyle::Regular,
            1 => TileStyle::Bold,
            2 => TileStyle::Italic,
            _ => TileStyle::BoldItalic,
        }
    }
}

// Describes the position of the glyph within the tile when rendered:
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileLayout {
    // Centered within the tile
    Center,
    // Centered horizontally but aligned with the bottom of the tile vertically
    Floor,
    // Positioned based on font metrics (as though it was text)
    Text,
    // Positioned based on offset values (from the center position)
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
    pub style: TileStyle,
    pub size: TileSize,
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
            style: TileStyle::Regular,
            size: TileSize::Normal,
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
// Note: OLD.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontMetrics {
    pub regular: Vec<GlyphMetric>,
    pub outline: Vec<GlyphMetric>,
}

// Array of glyph metrics for a font.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontMetricsV2 {
    pub metrics: Vec<GlyphMetric>,
}
