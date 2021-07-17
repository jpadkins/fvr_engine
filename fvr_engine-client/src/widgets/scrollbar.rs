//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::cmp;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::input_manager::*;
use crate::widgets::button::*;

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
    foreground_color: PaletteColor::DarkGrey.const_into(),
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
    background_color: PaletteColor::DarkGrey.const_into(),
    foreground_color: TileColor::TRANSPARENT,
    outline_color: TileColor::TRANSPARENT,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};

//-------------------------------------------------------------------------------------------------
// Enumerates the response codes when updating a scrollbar.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarAction {
    // The scrollbar was not interacted with.
    Noop,
    // The scrollbar has focus.
    Focused,
    // The content should scroll up by some number of lines.
    ScrollUp(u32),
    // The content should scroll down by some number of lines.
    ScrollDown(u32),
}

//-------------------------------------------------------------------------------------------------
// Scrollbar represents a scrollable, visual indicator of current position in vertical content.
//-------------------------------------------------------------------------------------------------
pub struct Scrollbar {
    // Origin of the scrollbar.
    origin: (u32, u32),
    // Height of the scrollbar.
    track_height: u32,
    // Height of the content that the scrollbar represents.
    content_height: u32,
    // Index of the current line at the top of the visible area.
    current_line: u32,
    // # of content lines represented by a segment of the track.
    track_ratio: u32,
    // Size of the grip.
    grip_size: u32,
    // Button at the top of the track.
    top_button: Button,
    // Button at the bottom of the track.
    bottom_button: Button,
}

impl Scrollbar {
    //---------------------------------------------------------------------------------------------
    // Refreshes the state of the scrollbar. Call whenever the height/content_height changes.
    //---------------------------------------------------------------------------------------------
    fn refresh(&mut self) {
        // Set the top/bottom button origins.
        self.top_button.origin = self.origin;
        self.bottom_button.origin = (self.origin.0, self.origin.1 + (self.track_height + 2) - 1);

        // Find how many lines the grip represents, taking into account the top/bottom buttons.
        self.track_ratio = self.content_height / self.track_height;

        // If the content height is shorter than the track height, don't draw the grip or buttons.
        if self.track_ratio == 0 {
            return;
        }

        // Set the grip size to a % of the track height equal to the ratio of content/track height.
        self.grip_size = (self.track_height as f32
            * (self.track_height as f32 / self.content_height as f32))
            as u32;

        if self.grip_size == 0 {
            // The grip should always have a length of at least 1.
            self.grip_size = 1;
        } else if self.content_height % self.track_height != 0 {
            // Extend the length to account for the remainder.
            self.grip_size += 1;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: (u32, u32), height: u32, content_height: u32) -> Self {
        debug_assert!(height > 2);

        let mut scrollbar = Scrollbar {
            origin,
            // Subtract 2 to account for the top/bottom buttons.
            track_height: height - 2,
            content_height,
            current_line: 0,
            track_ratio: 0,
            grip_size: 0,
            top_button: Button::new(Default::default(), "▲".into(), ButtonLayout::Center),
            bottom_button: Button::new(Default::default(), "▼".into(), ButtonLayout::Center),
        };

        scrollbar.refresh();
        scrollbar
    }

    //---------------------------------------------------------------------------------------------
    // Update the origin of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_origin(&mut self, origin: (u32, u32)) {
        self.origin = origin;
    }

    //---------------------------------------------------------------------------------------------
    // Update the height of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_height(&mut self, height: u32) {
        debug_assert!(height > 2);

        self.track_height = height - 2;
        self.refresh();
    }

    //---------------------------------------------------------------------------------------------
    // Update the content height of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_content_height(&mut self, content_height: u32) {
        self.content_height = content_height;
        self.refresh();
    }

    //---------------------------------------------------------------------------------------------
    // Update the current line of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_current_line(&mut self, current_lint: u32) {
        debug_assert!(current_lint < self.content_height - self.track_height);

        self.current_line = current_lint;
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for drawing the track and grip.
    //---------------------------------------------------------------------------------------------
    fn draw_track_and_grip<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Draw the track.
        for y in (self.origin.1 + 1)..(self.origin.1 + (self.track_height + 2) - 1) {
            *map.get_xy_mut((self.origin.0, y)) = TRACK_TILE;
        }

        // If all content is visible, do not draw the grip.
        if self.track_ratio == 0 {
            return;
        }

        // Calculate the grip offset.
        let mut grip_offset;

        if self.current_line + self.track_height == self.content_height {
            // Ensure grip reaches the end of the track if the end of the content is visible.
            grip_offset = self.track_height - self.grip_size;
        } else {
            // Set grip offset to a % of the track height equal to the current line/content ratio.
            grip_offset = (self.current_line as f32
                * (self.track_height as f32 / self.content_height as f32))
                as u32;

            // Ensure grip does not cover bottom button in instances where the content height is
            // equal to a multiple of the track height.
            if self.grip_size + grip_offset > self.origin.1 + 1 + self.track_height {
                grip_offset -= 1;
            }
        }

        // Draw the grip.
        for y in
            (self.origin.1 + 1 + grip_offset)..(self.origin.1 + 1 + grip_offset + self.grip_size)
        {
            *map.get_xy_mut((self.origin.0, y)) = GRIP_TILE;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the scrollbar, potentially redrawing if the state changes.
    //---------------------------------------------------------------------------------------------
    pub fn update_and_draw<M>(&mut self, input: &InputManager, map: &mut M) -> ScrollbarAction
    where
        M: Map2d<Tile>,
    {
        let mut action = ScrollbarAction::Noop;

        // Update the buttons and the action.
        let show_top_button = self.current_line > 0;
        let show_bottom_button = self.content_height - self.current_line > self.track_height;

        let top_action = if show_top_button {
            self.top_button.update_and_draw(input, map)
        } else {
            ButtonAction::Noop
        };
        let bottom_action = if show_bottom_button {
            self.bottom_button.update_and_draw(input, map)
        } else {
            ButtonAction::Noop
        };

        if top_action == ButtonAction::Triggered {
            // If the top button was triggered, scroll up the bar.
            let lines = cmp::min(self.track_ratio, self.current_line);
            action = ScrollbarAction::ScrollUp(lines);
            self.current_line -= lines;
        } else if top_action == ButtonAction::Focused {
            // Else if the top button was consumed, update action and break early.
            action = ScrollbarAction::Focused;
        } else if bottom_action == ButtonAction::Triggered {
            // Else if the bottom button was triggered, scroll down the bar, ensuring the content
            // is not overscrolled.
            let lines = cmp::min(
                self.track_ratio,
                (self.content_height - self.track_height) - self.current_line,
            );
            action = ScrollbarAction::ScrollDown(lines);
            self.current_line += lines;
        } else if bottom_action == ButtonAction::Focused {
            // Else if the bottom button was consumed, update action.
            action = ScrollbarAction::Focused;
        }

        // Draw the scrollbar.
        self.draw_track_and_grip(map);

        action
    }

    //---------------------------------------------------------------------------------------------
    // Draws the scrollbar. Only necessary initially and when moving the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn draw<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        self.top_button.draw(map);
        self.bottom_button.draw(map);
        self.draw_track_and_grip(map);
    }
}
