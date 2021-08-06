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
static CENTER_DEFAULT_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Center),
    style: Some(TileStyle::Regular),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::BrightGrey.const_into()),
    outline_color: None,
    background_opacity: None,
    foreground_opacity: None,
    outline_opacity: None,
};
static TEXT_DEFAULT_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: Some(TileStyle::Regular),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::BrightGrey.const_into()),
    outline_color: None,
    background_opacity: None,
    foreground_opacity: None,
    outline_opacity: None,
};

// Format settings for focused state button.
static CENTER_FOCUSED_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Center),
    style: Some(TileStyle::Regular),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::Gold.const_into()),
    outline_color: None,
    background_opacity: None,
    foreground_opacity: None,
    outline_opacity: None,
};
static TEXT_FOCUSED_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: Some(TileStyle::Regular),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::Gold.const_into()),
    outline_color: None,
    background_opacity: None,
    foreground_opacity: None,
    outline_opacity: None,
};

// Format settings for Pressed state button.
static CENTER_PRESSED_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Center),
    style: Some(TileStyle::Bold),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::Gold.const_into()),
    outline_color: None,
    background_opacity: None,
    foreground_opacity: None,
    outline_opacity: None,
};
static TEXT_PRESSED_SETTINGS: RichTextFormatSettings = RichTextFormatSettings {
    layout: Some(TileLayout::Text),
    style: Some(TileStyle::Bold),
    size: None,
    outlined: None,
    background_color: None,
    foreground_color: Some(PaletteColor::Gold.const_into()),
    outline_color: None,
    background_opacity: None,
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
// Represents the possible layouts of a button's text.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonLayout {
    // Center the text.
    Center,
    // Use font kerning offsets for the text.
    Text,
}

//-------------------------------------------------------------------------------------------------
// Represents the response codes when updating a button.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonAction {
    // The button was not interacted with.
    Noop,
    // The button consumed user input, but was not triggered.
    Interactable,
    // The button was triggered.
    Triggered,
}

//-------------------------------------------------------------------------------------------------
// Button describes a clickable button.
//-------------------------------------------------------------------------------------------------
pub struct Button {
    // Origin of the button.
    pub origin: ICoord,
    // Text of the button (plain only).
    pub text: String,
    // Layout of the button.
    pub layout: ButtonLayout,
    // State of the button.
    state: State,
}

impl Button {
    //---------------------------------------------------------------------------------------------
    // Creates a new button.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: ICoord, text: String, layout: ButtonLayout) -> Self {
        Self { origin, text, layout, state: State::Default }
    }

    //---------------------------------------------------------------------------------------------
    // Helper function to determine whether the button contains a coord.
    //---------------------------------------------------------------------------------------------
    fn contains(&self, coord: &ICoord) -> bool {
        coord.1 == self.origin.1
            && coord.0 >= self.origin.0
            && coord.0 < self.origin.0 + self.text.chars().count() as i32
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
    pub fn update<M>(&mut self, input: &InputManager, map: &mut M) -> ButtonAction
    where
        M: Map2d<Tile>,
    {
        match self.state {
            // In default state, wait for mouse to hover over button.
            State::Default => {
                if let Some(mouse_coord) = input.mouse_coord() {
                    if self.contains(&mouse_coord) {
                        self.state = State::Focused;
                        self.redraw(map);
                        return ButtonAction::Interactable;
                    }
                }
            }
            // In focused state, wait for mouse to either move off button or click.
            State::Focused => {
                if let Some(mouse_coord) = input.mouse_coord() {
                    if !self.contains(&mouse_coord) {
                        self.state = State::Default;
                        self.redraw(map);
                        return ButtonAction::Noop;
                    } else if input.mouse_clicked().0 {
                        self.state = State::Pressed;
                        self.redraw(map);
                        return ButtonAction::Interactable;
                    } else {
                        return ButtonAction::Interactable;
                    }
                }
            }
            // In pressed state, wait for mouse to either move off button or release.
            State::Pressed => {
                if let Some(mouse_coord) = input.mouse_coord() {
                    if !self.contains(&mouse_coord) {
                        self.state = State::Default;
                        self.redraw(map);
                        return ButtonAction::Noop;
                    } else if !input.mouse_pressed().0 {
                        self.state = State::Focused;
                        self.redraw(map);
                        return ButtonAction::Triggered;
                    } else {
                        return ButtonAction::Interactable;
                    }
                }
            }
        }

        ButtonAction::Noop
    }

    //---------------------------------------------------------------------------------------------
    // Draws the button. Only necessary initially and when moving the button.
    //---------------------------------------------------------------------------------------------
    pub fn redraw<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        match self.state {
            State::Default => {
                let settings = match self.layout {
                    ButtonLayout::Center => &CENTER_DEFAULT_SETTINGS,
                    ButtonLayout::Text => &TEXT_DEFAULT_SETTINGS,
                };

                RichTextWriter::write_plain_with_settings(map, self.origin, &self.text, settings);
            }
            State::Focused => {
                let settings = match self.layout {
                    ButtonLayout::Center => &CENTER_FOCUSED_SETTINGS,
                    ButtonLayout::Text => &TEXT_FOCUSED_SETTINGS,
                };

                RichTextWriter::write_plain_with_settings(map, self.origin, &self.text, settings);
            }
            State::Pressed => {
                let settings = match self.layout {
                    ButtonLayout::Center => &CENTER_PRESSED_SETTINGS,
                    ButtonLayout::Text => &TEXT_PRESSED_SETTINGS,
                };

                RichTextWriter::write_plain_with_settings(map, self.origin, &self.text, settings);
            }
        }
    }
}
