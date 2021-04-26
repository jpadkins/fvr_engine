//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Result};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

pub use sdl2::pixels::Color as SdlColor;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
pub const TILE_STYLE_COUNT: usize = 4;
pub const TILE_STYLE_NAMES: &[&str] = &["regular", "bold", "italic", "bold_italic"];
pub const TILE_SIZE_COUNT: usize = 4;

//-------------------------------------------------------------------------------------------------
// Dummy external struct definition for serde.
//-------------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize)]
#[serde(remote = "SdlColor")]
struct SdlColorDef {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

//-------------------------------------------------------------------------------------------------
// TileColor represents 8bit RGBA color.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct TileColor(#[serde(with = "SdlColorDef")] pub SdlColor);

impl TileColor {
    //---------------------------------------------------------------------------------------------
    // Constants.
    //---------------------------------------------------------------------------------------------
    pub const RED: TileColor = TileColor(SdlColor::RED);
    pub const BLUE: TileColor = TileColor(SdlColor::BLUE);
    pub const GREEN: TileColor = TileColor(SdlColor::GREEN);
    pub const WHITE: TileColor = TileColor(SdlColor::WHITE);
    pub const BLACK: TileColor = TileColor(SdlColor::BLACK);
    pub const TRANSPARENT: TileColor = TileColor(SdlColor::RGBA(255, 255, 255, 0));

    //---------------------------------------------------------------------------------------------
    // Create a TileColor from RGB values.
    //---------------------------------------------------------------------------------------------
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(SdlColor { r, g, b, a: std::u8::MAX })
    }

    //---------------------------------------------------------------------------------------------
    // Create a TileColor from RGBA values.
    //---------------------------------------------------------------------------------------------
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(SdlColor { r, g, b, a })
    }
}

impl Distribution<TileColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileColor {
        TileColor::rgb(rng.gen(), rng.gen(), rng.gen())
    }
}

//-------------------------------------------------------------------------------------------------
// TileStyle describes the style of the glyph within the tile when rendered.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileStyle {
    // The glyph has the default appearance.
    Regular = 0,
    // The glyph is bold.
    Bold,
    // The glyph is italic.
    Italic,
    // The glyph is both bold and italic.
    BoldItalic,
}

impl TileStyle {
    //---------------------------------------------------------------------------------------------
    // Get the format hint string corresponding to a tile style.
    //---------------------------------------------------------------------------------------------
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            TileStyle::Regular => "r",
            TileStyle::Bold => "b",
            TileStyle::Italic => "i",
            TileStyle::BoldItalic => "bi",
        }
    }

    //---------------------------------------------------------------------------------------------
    // Retrieve the tile style for a format hint string.
    //---------------------------------------------------------------------------------------------
    pub fn from_format_hint(hint: &str) -> Result<Self> {
        match hint {
            "r" => Ok(TileStyle::Regular),
            "b" => Ok(TileStyle::Bold),
            "i" => Ok(TileStyle::Italic),
            "bi" => Ok(TileStyle::BoldItalic),
            _ => Err(anyhow!(format!("Failed to find tile style for {}.", hint))),
        }
    }
}

impl Default for TileStyle {
    fn default() -> Self {
        Self::Regular
    }
}

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

//-------------------------------------------------------------------------------------------------
// TileSize describes the size of the tile's glyph when rendered.
//-------------------------------------------------------------------------------------------------
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

impl TileSize {
    //---------------------------------------------------------------------------------------------
    // Get the format hint string corresponding to a tile size.
    //---------------------------------------------------------------------------------------------
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            TileSize::Small => "s",
            TileSize::Normal => "n",
            TileSize::Big => "b",
            TileSize::Giant => "g",
        }
    }

    //---------------------------------------------------------------------------------------------
    // Retrieve the tile size for a format hint string.
    //---------------------------------------------------------------------------------------------
    pub fn from_format_hint(hint: &str) -> Result<Self> {
        match hint {
            "s" => Ok(TileSize::Small),
            "n" => Ok(TileSize::Normal),
            "b" => Ok(TileSize::Big),
            "g" => Ok(TileSize::Giant),
            _ => Err(anyhow!(format!("Failed to find tile size for {}.", hint))),
        }
    }
}

impl Default for TileSize {
    fn default() -> Self {
        Self::Normal
    }
}

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

//-------------------------------------------------------------------------------------------------
// TileLayout enumerates the possible positions of the glyph within a tile when rendered.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TileLayout {
    // The glyph is centered within the tile
    Center,
    // The glyph is centered horizontally but aligned with the bottom of the tile vertically.
    Floor,
    // The glyph is positioned based on font metrics, as though it was text.
    Text,
    // The glyph is positioned based on offset values from the center position.
    Exact((i32, i32)),
}

impl TileLayout {
    //---------------------------------------------------------------------------------------------
    // Get the format hint string corresponding to a tile layout.
    //---------------------------------------------------------------------------------------------
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            TileLayout::Center => "c",
            TileLayout::Floor => "f",
            TileLayout::Text => "t",
            TileLayout::Exact(_) => "e",
        }
    }

    //---------------------------------------------------------------------------------------------
    // Retrieve the tile layout for a format hint string.
    //---------------------------------------------------------------------------------------------
    pub fn from_format_hint(hint: &str) -> Result<Self> {
        match hint {
            "c" => Ok(TileLayout::Center),
            "f" => Ok(TileLayout::Floor),
            "t" => Ok(TileLayout::Text),
            "e" => Ok(TileLayout::Exact((0, 0))),
            _ => Err(anyhow!(format!("Failed to find tile layout for {}.", hint))),
        }
    }
}

impl Default for TileLayout {
    fn default() -> Self {
        Self::Center
    }
}

//-------------------------------------------------------------------------------------------------
// Tile describes a visual tile that can be rendered.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Tile {
    // The character of the tile.
    pub glyph: char,
    // The layout of the tile's glyph.
    pub layout: TileLayout,
    // The style of the tile's glyph.
    pub style: TileStyle,
    // The size of the tile's glyph.
    pub size: TileSize,
    // Whether the tile's glyph is outlined.
    pub outlined: bool,
    // The color of the tile's background quad.
    pub background_color: TileColor,
    // The color of the tile's glyph.
    pub foreground_color: TileColor,
    // The color of the tile's glyph's outline.
    pub outline_color: TileColor,
    // The opacity of the tile's glyph.
    pub foreground_opacity: f32,
    // The opacity of the tile's glyph's outline.
    pub outline_opacity: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            glyph: '?',
            layout: Default::default(),
            style: Default::default(),
            size: Default::default(),
            outlined: false,
            background_color: TileColor::TRANSPARENT,
            foreground_color: TileColor::TRANSPARENT,
            outline_color: TileColor::TRANSPARENT,
            foreground_opacity: 1.0,
            outline_opacity: 1.0,
        }
    }
}
