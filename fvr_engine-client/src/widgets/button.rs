//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::input_manager::*;
use crate::widgets::rich_text_writer::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------

// Format settings for default state button.
static DEFAULT_FORMAT_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: Some(TileStyle::Regular),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::BrightGrey.const_into()),
    outline_color: None,
    foreground_opacity: None,
    outline_opacity: None,
};

// Format settings for focused state button.
static FOCUSED_FORMAT_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: Some(TileStyle::Regular),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::Gold.const_into()),
    outline_color: None,
    foreground_opacity: None,
    outline_opacity: None,
};

// Format settings for Pressed state button.
static PRESSED_FORMAT_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: Some(TileStyle::Bold),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::Gold.const_into()),
    outline_color: None,
    foreground_opacity: None,
    outline_opacity: None,
};

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the button.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    // The button has not been interacted with.
    Default,
    // The button has focus.
    Focused,
    // The button has focus and has been pressed. When released, the button will be triggered and
    // reset to the focused state.
    Pressed,
}

//-------------------------------------------------------------------------------------------------
// Represents the response codes when updating a button.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonAction {
    // The button was not interacted with.
    Noop,
    // The button consumed user input, but was not triggered.
    Consumed,
    // The button was triggered.
    Triggered,
}

//-------------------------------------------------------------------------------------------------
// Button describes a clickable button.
//-------------------------------------------------------------------------------------------------
pub struct Button {
    pub origin: (u32, u32),
    pub text: String,
    state: State,
}

impl Button {
    //---------------------------------------------------------------------------------------------
    // Creates a new button.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: (u32, u32), text: String) -> Self {
        Self { origin, text, state: State::Default }
    }

    //---------------------------------------------------------------------------------------------
    // Helper function to determine whether the button contains a coord.
    //---------------------------------------------------------------------------------------------
    fn contains(&self, coord: &(u32, u32)) -> bool {
        coord.1 == self.origin.1
            && coord.0 >= self.origin.0
            && coord.0 <= self.origin.0 + self.text.len() as u32
    }

    //---------------------------------------------------------------------------------------------
    // Resets the button to the default state.
    //---------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.state = State::Default;
    }

    //---------------------------------------------------------------------------------------------
    // Updates the button, potentially redrawing if the state changes.
    //---------------------------------------------------------------------------------------------
    pub fn update_and_draw<M>(&mut self, input: &InputManager, map: &mut M) -> ButtonAction
    where
        M: Map2d<Tile>,
    {
        match self.state {
            // In default state, wait for mouse to hover over button.
            State::Default => {
                if let Some(mouse_coord) = input.mouse_coord() {
                    if self.contains(&mouse_coord) {
                        self.state = State::Focused;
                        self.draw(map);
                        return ButtonAction::Consumed;
                    }
                }
            }
            // In focused state, wait for mouse to either move off button or click.
            State::Focused => {
                if let Some(mouse_coord) = input.mouse_coord() {
                    if !self.contains(&mouse_coord) {
                        self.state = State::Default;
                        self.draw(map);
                        return ButtonAction::Noop;
                    } else if input.mouse_clicked().0 {
                        self.state = State::Pressed;
                        self.draw(map);
                        return ButtonAction::Consumed;
                    } else {
                        return ButtonAction::Consumed;
                    }
                }
            }
            // In pressed state, wait for mouse to either move off button or release.
            State::Pressed => {
                if let Some(mouse_coord) = input.mouse_coord() {
                    if !self.contains(&mouse_coord) {
                        self.state = State::Default;
                        self.draw(map);
                        return ButtonAction::Noop;
                    } else if !input.mouse_pressed().0 {
                        self.state = State::Focused;
                        self.draw(map);
                        return ButtonAction::Triggered;
                    } else {
                        return ButtonAction::Consumed;
                    }
                }
            }
        }

        ButtonAction::Noop
    }

    //---------------------------------------------------------------------------------------------
    // Draws the button. Only necessary initially and when moving the button.
    //---------------------------------------------------------------------------------------------
    pub fn draw<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        match self.state {
            State::Default => {
                RichTextWriter::write_plain_with_settings(
                    map,
                    self.origin,
                    &self.text,
                    &DEFAULT_FORMAT_SETTINGS,
                );
            }
            State::Focused => {
                RichTextWriter::write_plain_with_settings(
                    map,
                    self.origin,
                    &self.text,
                    &FOCUSED_FORMAT_SETTINGS,
                );
            }
            State::Pressed => {
                RichTextWriter::write_plain_with_settings(
                    map,
                    self.origin,
                    &self.text,
                    &PRESSED_FORMAT_SETTINGS,
                );
            }
        }
    }
}
