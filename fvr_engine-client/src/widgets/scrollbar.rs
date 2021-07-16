//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------

static TRACK_TILE: Tile = Tile {
    glyph: '|',
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::BrightGrey.const_into(),
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

static GRIP_TILE: Tile = Tile {
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
// Scrollbar represents a scrollable, visual indicator of current position in vertical content.
//-------------------------------------------------------------------------------------------------
pub struct Scrollbar {
    origin: (u32, u32),
    height: u32,
    content_height: u32,
}

impl Scrollbar {
    //---------------------------------------------------------------------------------------------
    // Refreshes the cached grip-related metrics. Call whenever the height/content_height changes.
    //---------------------------------------------------------------------------------------------
    fn refresh_grip_metrics(&mut self) {}

    //---------------------------------------------------------------------------------------------
    // Creates a new scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: (u32, u32), height: u32, content_height: u32) -> Self {
        let mut scrollbar = Scrollbar { origin, height, content_height };
        scrollbar.refresh_grip_metrics();
        scrollbar
    }
}
