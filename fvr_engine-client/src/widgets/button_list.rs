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
// Represents the response when updating a button list.
//-------------------------------------------------------------------------------------------------
pub struct ButtonListAction {
    // Whether the mouse was consumed.
    pub consumed: bool,
    // Optional index of triggered button.
    pub triggered: Option<u32>,
}

//-------------------------------------------------------------------------------------------------
// Button List manages a vertical list of buttons.
//-------------------------------------------------------------------------------------------------
pub struct ButtonList {
    // Origin of the button list.
    origin: (u32, u32),
    // Vec of buttons in the list.
    buttons: Vec<Button>,
    // Whether to add a space between the buttons.
    spacing: bool,
}

impl ButtonList {
    //---------------------------------------------------------------------------------------------
    // Creates a new button list.
    //---------------------------------------------------------------------------------------------
    pub fn new(origin: (u32, u32), spacing: bool) -> Self {
        Self { origin, buttons: Vec::new(), spacing }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the number of contained buttons.
    //---------------------------------------------------------------------------------------------
    pub fn height(&self) -> u32 {
        if !self.spacing {
            self.buttons.len() as u32
        } else {
            (self.buttons.len() as u32 * 2) - 1
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the maximum length of the contained buttons.
    //---------------------------------------------------------------------------------------------
    pub fn width(&self) -> u32 {
        let mut width = 0;

        // Find the maximum text length.
        for len in self.buttons.iter().map(|b| b.text.chars().count() as u32) {
            if len > width {
                width = len;
            }
        }

        width
    }

    //---------------------------------------------------------------------------------------------
    // Returns the button list's origin.
    //---------------------------------------------------------------------------------------------
    pub fn origin(&self) -> (u32, u32) {
        self.origin
    }

    //---------------------------------------------------------------------------------------------
    // Helper function to update all contained buttons' origins.
    //---------------------------------------------------------------------------------------------
    fn refresh_button_origins(&mut self) {
        let mut offset = 0;

        for (i, button) in self.buttons.iter_mut().enumerate() {
            button.origin = (self.origin.0, self.origin.1 + offset + i as u32);

            if self.spacing {
                offset += 1;
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the origin of the button list.
    //---------------------------------------------------------------------------------------------
    pub fn set_origin(&mut self, origin: (u32, u32)) {
        self.origin = origin;
        self.refresh_button_origins();
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new button list from a vec of buttons.
    //---------------------------------------------------------------------------------------------
    pub fn from_buttons_vec(origin: (u32, u32), buttons: Vec<Button>, spacing: bool) -> Self {
        let mut button_list = Self { origin, buttons, spacing };
        button_list.refresh_button_origins();
        button_list
    }

    //---------------------------------------------------------------------------------------------
    // Pushes back a new button.
    //---------------------------------------------------------------------------------------------
    pub fn push(&mut self, mut button: Button) {
        // Update button's origin to place it at the end.
        button.origin = (self.origin.0, self.origin.1 + self.height() + 1);

        self.buttons.push(button);
    }

    //---------------------------------------------------------------------------------------------
    // Removes all contained buttons.
    //---------------------------------------------------------------------------------------------
    pub fn clear(&mut self) {
        self.buttons.clear();
    }

    //---------------------------------------------------------------------------------------------
    // Resets all of the contained buttons to the default state.
    //---------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        for button in self.buttons.iter_mut() {
            button.reset();
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates each of the contained buttons, returning the index of any that are triggered.
    //---------------------------------------------------------------------------------------------
    pub fn update_and_draw<M>(&mut self, input: &InputManager, map: &mut M) -> ButtonListAction
    where
        M: Map2d<Tile>,
    {
        let mut consumed = false;
        let mut triggered = None;

        // Check each button, breaking early on triggered or consumed.
        for (i, button) in self.buttons.iter_mut().enumerate() {
            let action = button.update_and_draw(input, map);

            if action == ButtonAction::Triggered {
                consumed = true;
                triggered = Some(i as u32);
            } else if action == ButtonAction::Focused {
                consumed = true;
            }
        }

        ButtonListAction { consumed, triggered }
    }

    //---------------------------------------------------------------------------------------------
    // Draws each of the contained buttons. Should only be called after moving the button list.
    //---------------------------------------------------------------------------------------------
    pub fn draw<M>(&self, map: &mut M)
    where
        M: Map2d<Tile>,
    {
        for button in self.buttons.iter() {
            button.draw(map);
        }
    }
}
