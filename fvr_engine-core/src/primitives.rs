use anyhow::{anyhow, Result};
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

// Enumerates the set of static colors defined by the color palette.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PaletteColor {
    DarkRed,
    BrightRed,
    DarkOrange,
    BrightOrange,
    Brown,
    Yellow,
    DarkGreen,
    BrightGreen,
    DarkBlue,
    BrightBlue,
    DarkPurple,
    BrightPurple,
    DarkCyan,
    BrightCyan,
    DarkMagenta,
    BrightMagenta,
    Gold,
    Black,
    DarkGrey,
    BrightGrey,
    White,
    Transparent,
}

impl PaletteColor {
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            PaletteColor::DarkRed => "r",
            PaletteColor::BrightRed => "R",
            PaletteColor::DarkOrange => "o",
            PaletteColor::BrightOrange => "O",
            PaletteColor::Brown => "w",
            PaletteColor::Yellow => "W",
            PaletteColor::DarkGreen => "g",
            PaletteColor::BrightGreen => "G",
            PaletteColor::DarkBlue => "b",
            PaletteColor::BrightBlue => "B",
            PaletteColor::DarkPurple => "p",
            PaletteColor::BrightPurple => "P",
            PaletteColor::DarkCyan => "c",
            PaletteColor::BrightCyan => "C",
            PaletteColor::DarkMagenta => "m",
            PaletteColor::BrightMagenta => "M",
            PaletteColor::Gold => "$",
            PaletteColor::Black => "k",
            PaletteColor::DarkGrey => "K",
            PaletteColor::BrightGrey => "y",
            PaletteColor::White => "Y",
            PaletteColor::Transparent => "T",
        }
    }

    pub fn from_format_hint(hint: &str) -> Result<Self> {
        match hint {
            "r" => Ok(PaletteColor::DarkRed),
            "R" => Ok(PaletteColor::BrightRed),
            "o" => Ok(PaletteColor::DarkOrange),
            "O" => Ok(PaletteColor::BrightOrange),
            "w" => Ok(PaletteColor::Brown),
            "W" => Ok(PaletteColor::Yellow),
            "g" => Ok(PaletteColor::DarkGreen),
            "G" => Ok(PaletteColor::BrightGreen),
            "b" => Ok(PaletteColor::DarkBlue),
            "B" => Ok(PaletteColor::BrightBlue),
            "p" => Ok(PaletteColor::DarkPurple),
            "P" => Ok(PaletteColor::BrightPurple),
            "c" => Ok(PaletteColor::DarkCyan),
            "C" => Ok(PaletteColor::BrightCyan),
            "m" => Ok(PaletteColor::DarkMagenta),
            "M" => Ok(PaletteColor::BrightMagenta),
            "$" => Ok(PaletteColor::Gold),
            "k" => Ok(PaletteColor::Black),
            "K" => Ok(PaletteColor::DarkGrey),
            "y" => Ok(PaletteColor::BrightGrey),
            "Y" => Ok(PaletteColor::White),
            "T" => Ok(PaletteColor::Transparent),
            _ => Err(anyhow!(format!("Failed to find palette color for {}.", hint))),
        }
    }
}

impl Into<TileColor> for PaletteColor {
    fn into(self) -> TileColor {
        // TODO: Load these from config file.
        match self {
            PaletteColor::DarkRed => TileColor::rgb(115, 24, 45),
            PaletteColor::BrightRed => TileColor::rgb(223, 62, 35),
            PaletteColor::DarkOrange => TileColor::rgb(250, 106, 10),
            PaletteColor::BrightOrange => TileColor::rgb(249, 163, 27),
            PaletteColor::Brown => TileColor::rgb(113, 65, 59),
            PaletteColor::Yellow => TileColor::rgb(255, 252, 64),
            PaletteColor::DarkGreen => TileColor::rgb(26, 122, 62),
            PaletteColor::BrightGreen => TileColor::rgb(89, 193, 53),
            PaletteColor::DarkBlue => TileColor::rgb(40, 92, 196),
            PaletteColor::BrightBlue => TileColor::rgb(36, 159, 222),
            PaletteColor::DarkPurple => TileColor::rgb(67, 28, 83),
            PaletteColor::BrightPurple => TileColor::rgb(147, 112, 219),
            PaletteColor::DarkCyan => TileColor::rgb(32, 214, 199),
            PaletteColor::BrightCyan => TileColor::rgb(166, 252, 219),
            PaletteColor::DarkMagenta => TileColor::rgb(121, 58, 128),
            PaletteColor::BrightMagenta => TileColor::rgb(188, 74, 155),
            PaletteColor::Gold => TileColor::rgb(218, 165, 32),
            PaletteColor::Black => TileColor::rgb(23, 19, 18),
            PaletteColor::DarkGrey => TileColor::rgb(109, 117, 141),
            PaletteColor::BrightGrey => TileColor::rgb(179, 185, 209),
            PaletteColor::White => TileColor::rgb(255, 255, 255),
            PaletteColor::Transparent => TileColor::rgba(0, 0, 0, 0),
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

impl TileStyle {
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            TileStyle::Regular => "r",
            TileStyle::Bold => "b",
            TileStyle::Italic => "i",
            TileStyle::BoldItalic => "bi",
        }
    }

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

impl TileSize {
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            TileSize::Small => "s",
            TileSize::Normal => "n",
            TileSize::Big => "b",
            TileSize::Giant => "g",
        }
    }

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

impl TileLayout {
    pub const fn to_format_hint(&self) -> &'static str {
        match self {
            TileLayout::Center => "c",
            TileLayout::Floor => "f",
            TileLayout::Text => "t",
            TileLayout::Exact(_) => "e",
        }
    }

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
    pub foreground_opacity: f32,
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

// Array of glyph metrics for a font.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontMetricsV2 {
    pub metrics: Vec<GlyphMetric>,
}
