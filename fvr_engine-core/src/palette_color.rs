//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Result};
use serde_derive::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::tile::*;

//-------------------------------------------------------------------------------------------------
// Enumerates the different color palettes.
//-------------------------------------------------------------------------------------------------
pub enum ColorPalette {
    // The hardcoded default color palette.
    Static,
    // The current dynamically loaded color palette.
    Dynamic,
}

//-------------------------------------------------------------------------------------------------
// Enumerates the set of possible colors defined by the color palette.
//-------------------------------------------------------------------------------------------------
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
    //---------------------------------------------------------------------------------------------
    // Get the format hint string corresponding to a palette color.
    //---------------------------------------------------------------------------------------------
    pub const fn format_hint(&self) -> &'static str {
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

    //---------------------------------------------------------------------------------------------
    // Get the tile color corresponding to a palette color. Always uses the static palette.
    //---------------------------------------------------------------------------------------------
    pub const fn const_into(&self) -> TileColor {
        // TODO: Load these from config file. Or have a "default" palette and a "dynamic" palette.
        match self {
            PaletteColor::DarkRed => TileColor(SdlColor { r: 115, g: 24, b: 45, a: 255 }),
            PaletteColor::BrightRed => TileColor(SdlColor { r: 223, g: 62, b: 35, a: 255 }),
            PaletteColor::DarkOrange => TileColor(SdlColor { r: 250, g: 106, b: 10, a: 255 }),
            PaletteColor::BrightOrange => TileColor(SdlColor { r: 249, g: 163, b: 27, a: 255 }),
            PaletteColor::Brown => TileColor(SdlColor { r: 113, g: 65, b: 59, a: 255 }),
            PaletteColor::Yellow => TileColor(SdlColor { r: 255, g: 252, b: 64, a: 255 }),
            PaletteColor::DarkGreen => TileColor(SdlColor { r: 26, g: 122, b: 62, a: 255 }),
            PaletteColor::BrightGreen => TileColor(SdlColor { r: 89, g: 193, b: 53, a: 255 }),
            PaletteColor::DarkBlue => TileColor(SdlColor { r: 40, g: 92, b: 196, a: 255 }),
            PaletteColor::BrightBlue => TileColor(SdlColor { r: 36, g: 159, b: 222, a: 255 }),
            PaletteColor::DarkPurple => TileColor(SdlColor { r: 67, g: 28, b: 83, a: 255 }),
            PaletteColor::BrightPurple => TileColor(SdlColor { r: 147, g: 112, b: 219, a: 255 }),
            PaletteColor::DarkCyan => TileColor(SdlColor { r: 32, g: 214, b: 199, a: 255 }),
            PaletteColor::BrightCyan => TileColor(SdlColor { r: 166, g: 252, b: 219, a: 255 }),
            PaletteColor::DarkMagenta => TileColor(SdlColor { r: 121, g: 58, b: 128, a: 255 }),
            PaletteColor::BrightMagenta => TileColor(SdlColor { r: 188, g: 74, b: 155, a: 255 }),
            PaletteColor::Gold => TileColor(SdlColor { r: 218, g: 165, b: 32, a: 255 }),
            PaletteColor::Black => TileColor(SdlColor { r: 23, g: 19, b: 18, a: 255 }),
            PaletteColor::DarkGrey => TileColor(SdlColor { r: 109, g: 117, b: 141, a: 255 }),
            PaletteColor::BrightGrey => TileColor(SdlColor { r: 179, g: 185, b: 209, a: 255 }),
            PaletteColor::White => TileColor(SdlColor { r: 255, g: 255, b: 255, a: 255 }),
            PaletteColor::Transparent => TileColor(SdlColor { r: 0, g: 0, b: 0, a: 0 }),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Retrieve the palette color for a format hint string.
    //---------------------------------------------------------------------------------------------
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

impl From<PaletteColor> for TileColor {
    fn from(palette_color: PaletteColor) -> Self {
        palette_color.const_into()
    }
}
