//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Context, Result};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;
use fvr_engine_parser::prelude::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
// Empty struct exposing a static API for "writing" rich text into types that impl Map2D<Tile>.
//-------------------------------------------------------------------------------------------------
pub struct RichTextWriter {}

impl RichTextWriter {
    //---------------------------------------------------------------------------------------------
    // Write rich text, wrapping at the map2d's width.
    //---------------------------------------------------------------------------------------------
    pub fn write<M>(map: &mut M, xy: (u32, u32), text: &str) -> Result<()>
    where
        M: Map2d<Tile>,
    {
        // Declare mutable coords and options used to store current format state.
        let (mut x, mut y) = xy;

        let mut layout: Option<TileLayout> = None;
        let mut style: Option<TileStyle> = None;
        let mut size: Option<TileSize> = None;
        let mut outlined: Option<bool> = None;
        let mut foreground_color: Option<TileColor> = None;
        let mut background_color: Option<TileColor> = None;
        let mut outline_color: Option<TileColor> = None;

        // Parse the rich text.
        let parsed = parse_rich_text(text).context("Failed to parse rich text string.")?;

        // Iterate and handle the values.
        for value in parsed.into_iter() {
            match value {
                // For format hints, parse the hint value and update the format state.
                //---------------------------------------------------------------------------------
                RichTextValue::FormatHint { key, value } => match key {
                    RichTextHintType::Layout => {
                        let v = TileLayout::from_format_hint(&value)?;
                        layout = Some(v);
                    }
                    RichTextHintType::Style => {
                        let v = TileStyle::from_format_hint(&value)?;
                        style = Some(v);
                    }
                    RichTextHintType::Size => {
                        let v = TileSize::from_format_hint(&value)?;
                        size = Some(v);
                    }
                    RichTextHintType::Outlined => {
                        let v = match value.as_str() {
                            "t" => Ok(true),
                            "f" => Ok(false),
                            _ => Err(anyhow!("Failed to parse outlined value.")),
                        }?;
                        outlined = Some(v);
                    }
                    RichTextHintType::ForegroundColor => {
                        let v = PaletteColor::from_format_hint(&value)?;
                        foreground_color = Some(v.into());
                    }
                    RichTextHintType::BackgroundColor => {
                        let v = PaletteColor::from_format_hint(&value)?;
                        background_color = Some(v.into());
                    }
                    RichTextHintType::OutlineColor => {
                        let v = PaletteColor::from_format_hint(&value)?;
                        outline_color = Some(v.into());
                    }
                },
                // For newlines, reset the x coord and move to the next line.
                //---------------------------------------------------------------------------------
                RichTextValue::Newline => {
                    x = xy.0;
                    y += 1;
                }
                // For text, iter the chars and update the tiles with the format state.
                //---------------------------------------------------------------------------------
                RichTextValue::Text(text) => {
                    for glyph in text.chars() {
                        let tile = map.get_xy_mut((x, y));

                        tile.glyph = glyph;

                        if let Some(v) = layout {
                            tile.layout = v;
                        }
                        if let Some(v) = style {
                            tile.style = v;
                        }
                        if let Some(v) = size {
                            tile.size = v;
                        }
                        if let Some(v) = outlined {
                            tile.outlined = v;
                        }
                        if let Some(v) = foreground_color {
                            tile.foreground_color = v;
                        }
                        if let Some(v) = background_color {
                            tile.background_color = v;
                        }
                        if let Some(v) = outline_color {
                            tile.outline_color = v;
                        }

                        // Increment the coords, moving to the next line if necessary.
                        x += 1;

                        if x >= map.width() {
                            x = xy.0;
                            y += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
