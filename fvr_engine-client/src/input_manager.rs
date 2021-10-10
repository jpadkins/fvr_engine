//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use fnv::{FnvHashMap, FnvHashSet};
use serde_derive::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Result};
pub use sdl2::event::Event as InputEvent;
use sdl2::keyboard::KeyboardState;
pub use sdl2::keyboard::Keycode as InputKey;
use sdl2::mouse::{Cursor as SdlCursor, MouseState, SystemCursor};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// Path to current serialized keybindings. These can change.
const KEYBINDINGS_PATH: &str = "./config/keybindings.json";

// Path to default keybindings. These never change.
const DEFAULT_KEYBINDINGS_PATH: &str = "./config/default_keybindings.json";

//-------------------------------------------------------------------------------------------------
// InputAction enumerates the kinds of input the user can make.
// These actions are meant to be composite and remappable and used alongside individual key inputs.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, EnumIter, Eq, PartialEq, Hash, Serialize)]
pub enum InputAction {
    Accept,
    Decline,
    Quit,
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

//-------------------------------------------------------------------------------------------------
// TODO: Perhaps use SDL's keycode enum once serde can derive it de/serialization?
//-------------------------------------------------------------------------------------------------
pub type InputKeycode = i32;

//-------------------------------------------------------------------------------------------------
// ModifierKey enumerates the types of modifier keys that might be pressed.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ModifierKey {
    Alt,
    Ctrl,
    Shift,
}

//-------------------------------------------------------------------------------------------------
// InputMouse enumerates the buttons on a mouse.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InputMouse {
    Left,
    Right,
}

//-------------------------------------------------------------------------------------------------
// Describes an entry in the keybindings for an input action - either a specific key or a modifier.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InputBinding {
    SpecificKey(InputKeycode),
    ModifierKey(ModifierKey),
    ExcludeSpecificKey(InputKeycode),
    ExcludeModifierKey(ModifierKey),
}

//-------------------------------------------------------------------------------------------------
// Cursor enumerates the types of mouse cursors available.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cursor {
    Arrow,
    Crosshair,
    Hand,
    IBeam,
    No,
    Wait,
}

//-------------------------------------------------------------------------------------------------
// InputManager exposes an API for managing user input state.
//-------------------------------------------------------------------------------------------------
#[derive(Default)]
pub struct InputManager {
    // Current pressed state of left and right mouse buttons.
    mouse_pressed: (bool, bool),
    // Current clicked (just pressed) state of left and right mouse buttons.
    mouse_clicked: (bool, bool),
    // Current coord of the mouse within the faux terminal (or none if it is out of bounds).
    mouse_coord: Option<ICoord>,
    // Whether the mouse changed coords.
    mouse_moved: bool,
    // Set of keys that are currently pressed.
    pressed_keys: FnvHashSet<InputKey>,
    // Set of keys that have become pressed this frame.
    just_pressed_keys: FnvHashSet<InputKey>,
    // Set of keys that have been released.
    released_keys: FnvHashSet<InputKey>,
    // Set of actions that are currently pressed.
    pressed_actions: FnvHashSet<InputAction>,
    // Set of actions that have become pressed this frame.
    just_pressed_actions: FnvHashSet<InputAction>,
    // Set of actions that have been released.
    released_actions: FnvHashSet<InputAction>,
    // Map of input actions to their bound key combinations.
    action_bindings: FnvHashMap<InputAction, Vec<InputBinding>>,
    // Whether any key was pressed.
    pressed_any_key: bool,
    // Whether any action was pressed.
    pressed_any_action: bool,
    // Vec of cursors.
    cursors: Vec<SdlCursor>,
}

impl InputManager {
    //---------------------------------------------------------------------------------------------
    // Helper function for create a new input manager.
    //---------------------------------------------------------------------------------------------
    fn new(keybindings_path: &str) -> Result<Self> {
        let cursors = vec![
            SdlCursor::from_system(SystemCursor::Arrow).map_err(|e| anyhow!(e))?,
            SdlCursor::from_system(SystemCursor::Crosshair).map_err(|e| anyhow!(e))?,
            SdlCursor::from_system(SystemCursor::Hand).map_err(|e| anyhow!(e))?,
            SdlCursor::from_system(SystemCursor::IBeam).map_err(|e| anyhow!(e))?,
            SdlCursor::from_system(SystemCursor::No).map_err(|e| anyhow!(e))?,
            SdlCursor::from_system(SystemCursor::Wait).map_err(|e| anyhow!(e))?,
        ];
        let keybindings_json = std::fs::read_to_string(keybindings_path)?;

        Ok(Self {
            cursors,
            action_bindings: serde_json::from_str(&keybindings_json)?,
            ..Default::default()
        })
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new input manager.
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn with_keybindings() -> Result<Self> {
        Self::new(KEYBINDINGS_PATH)
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new input manager with default action bindings.
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn with_default_bindings() -> Result<Self> {
        Self::new(DEFAULT_KEYBINDINGS_PATH)
    }

    //---------------------------------------------------------------------------------------------
    // Helper function that returns whether an input action binding is pressed.
    //---------------------------------------------------------------------------------------------
    fn binding_pressed(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::SpecificKey(k) => {
                self.pressed_keys.contains(&InputKey::from_i32(*k).expect("Invalid keycode."))
            }
            InputBinding::ModifierKey(m) => self.modifier_pressed(m),
            InputBinding::ExcludeSpecificKey(k) => {
                !self.pressed_keys.contains(&InputKey::from_i32(*k).expect("Invalid keycode."))
            }
            InputBinding::ExcludeModifierKey(m) => !self.modifier_pressed(m),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Helper function that returns whether a keycode is one of: Alt, Ctrl, Shift, Gui, App.
    //---------------------------------------------------------------------------------------------
    fn is_modifier(keycode: InputKey) -> bool {
        keycode == InputKey::LAlt
            || keycode == InputKey::RAlt
            || keycode == InputKey::LCtrl
            || keycode == InputKey::RCtrl
            || keycode == InputKey::LShift
            || keycode == InputKey::RShift
            || keycode == InputKey::LGui
            || keycode == InputKey::RGui
            || keycode == InputKey::Application
    }

    //---------------------------------------------------------------------------------------------
    // Updates the input manager from current keyboard state.
    // (should be called once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        mouse_coord: Option<ICoord>,
    ) {
        // Update key states.
        //-----------------------------------------------------------------------------------------

        // Iterate over all keys.
        for (scancode, pressed) in keyboard_state.scancodes() {
            if let Some(keycode) = InputKey::from_scancode(scancode) {
                // If pressed:
                // - insert into the pressed key set.
                // - insert into the just pressed key set if the key had previously been released.
                if pressed {
                    self.pressed_keys.insert(keycode);

                    // Ignore modifier keys and alt-tab when updating pressed_any_key.
                    if !Self::is_modifier(keycode)
                        || (keycode == InputKey::Tab && !self.modifier_pressed(&ModifierKey::Alt))
                    {
                        self.pressed_any_key = true;
                    }

                    if self.released_keys.contains(&keycode) {
                        self.just_pressed_keys.insert(keycode);
                        self.released_keys.remove(&keycode);
                    }
                // If not pressed, record that the key has been released.
                } else {
                    self.released_keys.insert(keycode);
                }
            }
        }

        // Update action states.
        //-----------------------------------------------------------------------------------------

        // Iterate over all actions.
        for input_action in InputAction::iter() {
            // If the action has keybindings...
            if let Some(bindings) = self.action_bindings.get(&input_action) {
                // ...and if all of the bindings are pressed:
                // - insert into the the pressed action set.
                // - insert into the just pressed action set if the action had previously been
                //   released.
                if bindings.iter().all(|b| self.binding_pressed(b)) {
                    self.pressed_actions.insert(input_action);
                    self.pressed_any_action = true;

                    if self.released_actions.contains(&input_action) {
                        self.just_pressed_actions.insert(input_action);
                        self.released_actions.remove(&input_action);
                    }
                // ...otherwise, record that the action has been released.
                } else {
                    self.released_actions.insert(input_action);
                }
            }
        }

        // Update mouse states.
        //-----------------------------------------------------------------------------------------

        // Set clicked to true if the mouse button was not pressed last frame.
        self.mouse_clicked.0 = self.mouse_clicked.0 || !self.mouse_pressed.0 && mouse_state.left();
        self.mouse_clicked.1 =
            self.mouse_clicked.1 || !self.mouse_pressed.1 && mouse_state.right();

        // Set remaining state.
        self.mouse_pressed.0 = mouse_state.left();
        self.mouse_pressed.1 = mouse_state.right();

        // Previous mouse coord should be a record of the last different mouse coord.
        if self.mouse_coord != mouse_coord {
            self.mouse_coord = mouse_coord;
            self.mouse_moved = true;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Reset the key and action state.
    // (should be called once after input is consumed)
    //---------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        // Clear the key state.
        self.pressed_keys.clear();
        self.just_pressed_keys.clear();
        self.pressed_any_key = false;

        // Clear the action state.
        self.pressed_actions.clear();
        self.just_pressed_actions.clear();
        self.pressed_any_action = false;

        // Clear the mouse state.
        self.mouse_clicked.0 = false;
        self.mouse_clicked.1 = false;
        self.mouse_moved = false;
    }

    //---------------------------------------------------------------------------------------------
    // Returns the pressed state of a mouse button.
    //---------------------------------------------------------------------------------------------
    pub fn mouse_pressed(&self, button: InputMouse) -> bool {
        match button {
            InputMouse::Left => self.mouse_pressed.0,
            InputMouse::Right => self.mouse_pressed.1,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns current pressed state of the mouse buttons
    //---------------------------------------------------------------------------------------------
    pub fn mouse_pressed_state(&self) -> (bool, bool) {
        self.mouse_pressed
    }

    //---------------------------------------------------------------------------------------------
    // Returns the clicked state of a mouse button.
    //---------------------------------------------------------------------------------------------
    pub fn mouse_clicked(&self, button: InputMouse) -> bool {
        match button {
            InputMouse::Left => self.mouse_clicked.0,
            InputMouse::Right => self.mouse_clicked.1,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns current clicked state of the mouse buttons
    //---------------------------------------------------------------------------------------------
    pub fn mouse_clicked_state(&self) -> (bool, bool) {
        self.mouse_clicked
    }

    //---------------------------------------------------------------------------------------------
    // Returns current mouse coord within the faux terminal (or none if out of bounds).
    //---------------------------------------------------------------------------------------------
    pub fn mouse_coord(&self) -> Option<ICoord> {
        self.mouse_coord
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the mouse has moved to a new coord.
    //---------------------------------------------------------------------------------------------
    pub fn mouse_moved(&self) -> bool {
        self.mouse_moved
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether a modifier key is pressed.
    //---------------------------------------------------------------------------------------------
    pub fn modifier_pressed(&self, modifier: &ModifierKey) -> bool {
        match modifier {
            ModifierKey::Alt => {
                self.pressed_keys.contains(&InputKey::LAlt)
                    || self.pressed_keys.contains(&InputKey::RAlt)
            }
            ModifierKey::Ctrl => {
                self.pressed_keys.contains(&InputKey::LCtrl)
                    || self.pressed_keys.contains(&InputKey::RCtrl)
            }
            ModifierKey::Shift => {
                self.pressed_keys.contains(&InputKey::LShift)
                    || self.pressed_keys.contains(&InputKey::RShift)
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether a key is pressed.
    //---------------------------------------------------------------------------------------------
    pub fn key_pressed(&self, key: InputKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether a key was just pressed this frame.
    //---------------------------------------------------------------------------------------------
    pub fn key_just_pressed(&self, key: InputKey) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether an action is pressed.
    //---------------------------------------------------------------------------------------------
    pub fn action_pressed(&self, action: InputAction) -> bool {
        self.pressed_actions.contains(&action)
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether an action was just pressed this frame.
    //---------------------------------------------------------------------------------------------
    pub fn action_just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed_actions.contains(&action)
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether any key is currently pressed.
    //---------------------------------------------------------------------------------------------
    pub fn any_key_pressed(&self) -> bool {
        self.pressed_any_key
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether any action is currently pressed.
    //---------------------------------------------------------------------------------------------
    pub fn any_action_pressed(&self) -> bool {
        self.pressed_any_action
    }

    //---------------------------------------------------------------------------------------------
    // Update the key bindings for an action.
    //---------------------------------------------------------------------------------------------
    pub fn bind_action(&mut self, action: InputAction, bindings: &[InputBinding]) {
        // Do not bind empty key set.
        if bindings.is_empty() {
            debug_assert!(false);
        }

        // Insert the new action binding.
        self.action_bindings.insert(action, bindings.to_vec());
    }

    //---------------------------------------------------------------------------------------------
    // Set the current cursor.
    //---------------------------------------------------------------------------------------------
    pub fn set_cursor(&self, cursor: Cursor) {
        self.cursors[cursor as usize].set();
    }

    //---------------------------------------------------------------------------------------------
    // Equivalent to calling set_cursor with CursorStyle::Arrow.
    //---------------------------------------------------------------------------------------------
    pub fn reset_cursor(&self) {}
}
