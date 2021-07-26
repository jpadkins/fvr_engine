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
// Constants
//-------------------------------------------------------------------------------------------------
const TOP_CHAR: char = '▲';
const BOTTOM_CHAR: char = '▼';

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
    background_opacity: 1.0,
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
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};
static LIMIT_TOP_TILE: Tile = Tile {
    glyph: TOP_CHAR,
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::DarkGrey.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
    foreground_opacity: 1.0,
    outline_opacity: 1.0,
};
static LIMIT_BOTTOM_TILE: Tile = Tile {
    glyph: BOTTOM_CHAR,
    layout: TileLayout::Center,
    style: TileStyle::Bold,
    size: TileSize::Normal,
    outlined: false,
    background_color: TileColor::TRANSPARENT,
    foreground_color: PaletteColor::DarkGrey.const_into(),
    outline_color: TileColor::TRANSPARENT,
    background_opacity: 1.0,
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
    // The scrollbar has focus (consumed user input).
    Focused,
    // The mouse is over an interactable area of the scrollbar.
    Interactable,
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
    origin: UCoord,
    // Height of the scrollbar.
    height: u32,
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
    // Whether the scrollbar needs to be redrawn.
    dirty: bool,
}

impl Scrollbar {
    //---------------------------------------------------------------------------------------------
    // Refreshes the state of the scrollbar. Call whenever the height/content_height changes.
    //---------------------------------------------------------------------------------------------
    fn refresh(&mut self) {
        // Set the top/bottom button origins.
        self.top_button.origin = self.origin;
        self.bottom_button.origin = (self.origin.0, self.origin.1 + self.height - 1);

        // Find how many lines the grip represents, taking into account the top/bottom buttons.
        self.track_ratio = self.content_height / self.height;

        // If the content height is shorter than the height, don't draw the grip or buttons.
        if self.track_ratio == 0 {
            return;
        }

        // Set the grip size to a % of the track height equal to the ratio of content/height.
        self.grip_size =
            ((self.height - 2) as f32 * (self.height as f32 / self.content_height as f32)) as u32;

        // The grip should always have a length of at least 1.
        if self.grip_size == 0 {
            self.grip_size = 1;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: UCoord, height: u32, content_height: u32) -> Self {
        debug_assert!(height > 2);

        let top_button = Button::new(Default::default(), TOP_CHAR.into(), ButtonLayout::Center);
        let bottom_button =
            Button::new(Default::default(), BOTTOM_CHAR.into(), ButtonLayout::Center);

        let mut scrollbar = Scrollbar {
            origin,
            height,
            content_height,
            current_line: 0,
            track_ratio: 0,
            grip_size: 0,
            top_button,
            bottom_button,
            dirty: true,
        };

        scrollbar.refresh();
        scrollbar
    }

    //---------------------------------------------------------------------------------------------
    // Returns the origin of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn origin(&self) -> UCoord {
        self.origin
    }

    //---------------------------------------------------------------------------------------------
    // Returns the height of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn height(&self) -> u32 {
        self.height
    }

    //---------------------------------------------------------------------------------------------
    // Update the origin of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_origin(&mut self, origin: UCoord) {
        self.origin = origin;
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Update the height of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_height(&mut self, height: u32) {
        debug_assert!(height > 2);

        self.height = height;
        self.refresh();
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Update the content height of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_content_height(&mut self, content_height: u32) {
        self.content_height = content_height;
        self.refresh();
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Update the current line of the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn set_current_line(&mut self, current_lint: u32) {
        debug_assert!(current_lint < self.content_height - self.height);

        self.current_line = current_lint;
        self.dirty = true;
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for drawing the track and grip.
    //---------------------------------------------------------------------------------------------
    fn draw_track_and_grip<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        // Draw the track.
        for y in (self.origin.1 + 1)..(self.origin.1 + self.height - 1) {
            *map.get_xy_mut((self.origin.0, y)) = TRACK_TILE;
        }

        // If all content is visible, do not draw the grip.
        if self.track_ratio == 0 {
            return;
        }

        // Calculate the grip offset.
        let mut grip_offset;
        let track_height = self.height - 2;

        if self.current_line + self.height == self.content_height {
            // Ensure grip reaches the end of the track if the end of the content is visible.
            grip_offset = track_height - self.grip_size;
        } else {
            // Set grip offset to a % of the track height equal to the current line/content ratio.
            grip_offset = (track_height as f32
                * (self.current_line as f32 / self.content_height as f32))
                as u32;

            // Ensure grip does not cover bottom button in instances where the content height is
            // equal to a multiple of the height.
            if self.grip_size + grip_offset > self.origin.1 + 1 + track_height {
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
    // Helper function to determine whether the scrollbar contains a coord.
    //---------------------------------------------------------------------------------------------
    fn contains(&self, coord: &UCoord) -> bool {
        coord.0 == self.origin.0
            && coord.1 >= self.origin.1
            && coord.1 < self.origin.1 + self.height
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for updating the top/bottom buttons.
    //---------------------------------------------------------------------------------------------
    fn update_buttons<M>(
        &mut self,
        input: &InputManager,
        map: &mut M,
    ) -> (ButtonAction, ButtonAction)
    where
        M: Map2d<Tile>,
    {
        // Update the top button if applicable.
        let show_top_button = self.current_line > 0;
        let top_action = if show_top_button {
            let action = self.top_button.update(input, map);
            // Force a redraw in case the limit tile was drawn previously.
            self.top_button.redraw(map);
            action
        } else {
            // If the top limit has been reached, draw a static arrow instead of a button.
            *map.get_xy_mut(self.origin) = LIMIT_TOP_TILE;
            ButtonAction::Noop
        };

        // Update the bottom button if applicable,
        let show_bottom_button = self.content_height - self.current_line > self.height;
        let bottom_action = if show_bottom_button {
            let action = self.bottom_button.update(input, map);
            // Force a redraw in case the limit tile was drawn previously.
            self.bottom_button.redraw(map);
            action
        } else {
            // If the bottom limit has been reached, draw a static arrow instead of a button.
            *map.get_xy_mut((self.origin.0, self.origin.1 + self.height - 1)) = LIMIT_BOTTOM_TILE;
            ButtonAction::Noop
        };

        (top_action, bottom_action)
    }

    //---------------------------------------------------------------------------------------------
    // Updates the scrollbar, potentially redrawing if the state changes.
    //---------------------------------------------------------------------------------------------
    pub fn update<M>(&mut self, input: &InputManager, map: &mut M) -> ScrollbarAction
    where
        M: Map2d<Tile>,
    {
        let mut action = ScrollbarAction::Noop;

        // Update the buttons and the action.
        let (top_action, bottom_action) = self.update_buttons(input, map);

        // Determine the response.
        if top_action == ButtonAction::Triggered {
            // If the top button was triggered, scroll up the bar.
            let lines = cmp::min(self.track_ratio, self.current_line);
            action = ScrollbarAction::ScrollUp(lines);
            self.current_line -= lines;
            self.dirty = true;
        } else if top_action == ButtonAction::Interactable {
            // Else if the top button was consumed, update action and break early.
            action = ScrollbarAction::Interactable;
        } else if bottom_action == ButtonAction::Triggered {
            // Else if the bottom button was triggered, scroll down the bar, ensuring the content
            // is not overscrolled.
            let lines = cmp::min(
                self.track_ratio,
                (self.content_height - self.height) - self.current_line,
            );
            action = ScrollbarAction::ScrollDown(lines);
            self.current_line += lines;
            self.dirty = true;
        } else if bottom_action == ButtonAction::Interactable {
            // Else if the bottom button was consumed, update action.
            action = ScrollbarAction::Interactable;
        } else {
            // Else check if the scrollbar contains the mouse coord and update action.
            if let Some(coord) = input.mouse_coord() {
                if self.contains(&coord) {
                    action = ScrollbarAction::Focused;
                }
            }
        }

        // Draw the scrollbar if dirty.
        if self.dirty {
            self.draw_track_and_grip(map);
            self.dirty = false;
        }

        action
    }

    //---------------------------------------------------------------------------------------------
    // Draws the scrollbar. Only necessary initially and when moving the scrollbar.
    //---------------------------------------------------------------------------------------------
    pub fn redraw<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        self.top_button.redraw(map);
        self.bottom_button.redraw(map);
        self.draw_track_and_grip(map);
    }
}
