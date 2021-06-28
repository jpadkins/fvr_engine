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
const NEWLINE_CHAR: char = '\n';

//-------------------------------------------------------------------------------------------------
// Helper struct for holding rich text format settings.
//-------------------------------------------------------------------------------------------------
#[derive(Default)]
pub struct RichTextFormatSettings {
    pub layout: Option<TileLayout>,
    pub style: Option<TileStyle>,
    pub size: Option<TileSize>,
    pub outlined: Option<bool>,
    pub background_color: Option<TileColor>,
    pub foreground_color: Option<TileColor>,
    pub outline_color: Option<TileColor>,
    pub foreground_opacity: Option<f32>,
    pub outline_opacity: Option<f32>,
}

//-------------------------------------------------------------------------------------------------
// RichTextWriter exposes a static API for "writing" rich text into types that impl Map2D<Tile>.
//-------------------------------------------------------------------------------------------------
pub struct RichTextWriter;

impl RichTextWriter {
    //---------------------------------------------------------------------------------------------
    // Find the len of a rich text string, excluding formatting tags.
    //---------------------------------------------------------------------------------------------
    pub fn stripped_len(text: &str) -> Result<usize> {
        let mut len = 0;

        // Parse the rich text.
        let parsed = parse_rich_text(text).context("Failed to parse rich text string.")?;

        // Increment len for text and newlines.
        for value in parsed.into_iter() {
            match value {
                RichTextValue::FormatHint { .. } => {}
                RichTextValue::Newline => len += 1,
                RichTextValue::Text(t) => len += t.len(),
            }
        }

        Ok(len)
    }

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
                        // Move to the next line if necessary.
                        if x >= map.width() {
                            x = xy.0;
                            y += 1;
                        }

                        // Update the tile.
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

                        // Increment the columns.
                        x += 1;
                    }
                }
            }
        }

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Write plain text (no inline hints), wrapping at the map2d's width.
    //---------------------------------------------------------------------------------------------
    pub fn write_plain<M>(map: &mut M, xy: (u32, u32), text: &str)
    where
        M: Map2d<Tile>,
    {
        // Declare mutable coords and options used to store current format state.
        let (mut x, mut y) = xy;

        // Iterate and handle the glyphs.
        for glyph in text.chars() {
            // Handle newline characters.
            if glyph == NEWLINE_CHAR {
                x = xy.0;
                y += 1;
                continue;
            }

            // Move to the next line if necessary.
            if x >= map.width() {
                x = xy.0;
                y += 1;
            }

            // Update the tile.
            let tile = map.get_xy_mut((x, y));
            tile.glyph = glyph;

            // Increment the columns.
            x += 1;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Write plain text (no inline hints) with formatting options, wrapping at the map2d's width.
    //---------------------------------------------------------------------------------------------
    pub fn write_formatted_plain<M>(
        map: &mut M,
        xy: (u32, u32),
        text: &str,
        layout: Option<TileLayout>,
        style: Option<TileStyle>,
        size: Option<TileSize>,
        outlined: Option<bool>,
        background_color: Option<TileColor>,
        foreground_color: Option<TileColor>,
        outline_color: Option<TileColor>,
        foreground_opacity: Option<f32>,
        outline_opacity: Option<f32>,
    ) where
        M: Map2d<Tile>,
    {
        // Declare mutable coords and options used to store current format state.
        let (mut x, mut y) = xy;

        // Iterate and handle the glyphs.
        for glyph in text.chars() {
            // Handle newline characters.
            if glyph == NEWLINE_CHAR {
                x = xy.0;
                y += 1;
                continue;
            }

            // Move to the next line if necessary.
            if x >= map.width() {
                x = xy.0;
                y += 1;
            }

            // Update the tile.
            let tile = map.get_xy_mut((x, y));

            tile.glyph = glyph;

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
            if let Some(foreground_opacity) = foreground_opacity {
                tile.foreground_opacity = foreground_opacity;
            }
            if let Some(outline_opacity) = outline_opacity {
                tile.outline_opacity = outline_opacity;
            }

            // Increment the columns.
            x += 1;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Write plain text (no inline hints) from format settings, wrapping at the map2d's width.
    //---------------------------------------------------------------------------------------------
    pub fn write_plain_with_settings<M>(
        map: &mut M,
        xy: (u32, u32),
        text: &str,
        settings: &RichTextFormatSettings,
    ) where
        M: Map2d<Tile>,
    {
        // Declare mutable coords and options used to store current format state.
        let (mut x, mut y) = xy;

        // Iterate and handle the glyphs.
        for glyph in text.chars() {
            // Handle newline characters.
            if glyph == NEWLINE_CHAR {
                x = xy.0;
                y += 1;
                continue;
            }

            // Move to the next line if necessary.
            if x >= map.width() {
                x = xy.0;
                y += 1;
            }

            // Update the tile.
            let tile = map.get_xy_mut((x, y));
            tile.glyph = glyph;

            if let Some(layout) = settings.layout {
                tile.layout = layout;
            }
            if let Some(style) = settings.style {
                tile.style = style;
            }
            if let Some(size) = settings.size {
                tile.size = size;
            }
            if let Some(outlined) = settings.outlined {
                tile.outlined = outlined;
            }
            if let Some(background_color) = settings.background_color {
                tile.background_color = background_color;
            }
            if let Some(foreground_color) = settings.foreground_color {
                tile.foreground_color = foreground_color;
            }
            if let Some(outline_color) = settings.outline_color {
                tile.outline_color = outline_color;
            }
            if let Some(foreground_opacity) = settings.foreground_opacity {
                tile.foreground_opacity = foreground_opacity;
            }
            if let Some(outline_opacity) = settings.outline_opacity {
                tile.outline_opacity = outline_opacity;
            }

            // Increment the columns.
            x += 1;
        }
    }
}
