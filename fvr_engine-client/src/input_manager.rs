//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::{HashMap, HashSet};

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use sdl2::keyboard::KeyboardState;
pub use sdl2::keyboard::Keycode as SdlKey;
use sdl2::mouse::MouseState;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

//-------------------------------------------------------------------------------------------------
// InputAction enumerates the kinds of input the user can make.
// These actions are meant to be composite and remappable and used alongside individual key inputs.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, EnumIter, Eq, PartialEq, Hash)]
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
// ModifierKey enumerates the types of modifier keys that might be pressed.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModifierKey {
    Alt,
    Ctrl,
    Shift,
}

//-------------------------------------------------------------------------------------------------
// Describes an entry in the keybindings for an input action - either a specific key or a modifier.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputBinding {
    SpecificKey(SdlKey),
    ModifierKey(ModifierKey),
    ExcludeSpecificKey(SdlKey),
    ExcludeModifierKey(ModifierKey),
}

//-------------------------------------------------------------------------------------------------
// InputManager exposes an API for managing user input state.
//-------------------------------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct InputManager {
    // Current pressed state of left and right mouse buttons.
    mouse_pressed: (bool, bool),
    // Current clicked (just pressed) state of left and right mouse buttons.
    mouse_clicked: (bool, bool),
    // Current coord of the mouse within the faux terminal (or none if it is out of bounds).
    mouse_coord: Option<(u32, u32)>,
    // Set of keys that are currently pressed.
    pressed_keys: HashSet<SdlKey>,
    // Set of keys that have become pressed this frame.
    just_pressed_keys: HashSet<SdlKey>,
    // Set of keys that have been released.
    released_keys: HashSet<SdlKey>,
    // Set of actions that are currently pressed.
    pressed_actions: HashSet<InputAction>,
    // Set of actions that have become pressed this frame.
    just_pressed_actions: HashSet<InputAction>,
    // Set of actions that have been released.
    released_actions: HashSet<InputAction>,
    // Map of input actions to their bound key combinations.
    action_bindings: HashMap<InputAction, Vec<InputBinding>>,
}

impl InputManager {
    //---------------------------------------------------------------------------------------------
    // Creates a new input manager.
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Default::default()
    }

    //---------------------------------------------------------------------------------------------
    // Returns an input manager with default action bindings.
    //---------------------------------------------------------------------------------------------
    pub fn with_default_bindings() -> Self {
        let mut this = Self::new();

        // TODO: load these from config.
        this.bind_action(InputAction::Accept, &[InputBinding::SpecificKey(SdlKey::Return)]);
        this.bind_action(
            InputAction::Decline,
            &[
                InputBinding::SpecificKey(SdlKey::Tab),
                InputBinding::ExcludeModifierKey(ModifierKey::Alt),
            ],
        );
        this.bind_action(InputAction::Quit, &[InputBinding::SpecificKey(SdlKey::Escape)]);
        this.bind_action(InputAction::North, &[InputBinding::SpecificKey(SdlKey::K)]);
        this.bind_action(InputAction::Northeast, &[InputBinding::SpecificKey(SdlKey::U)]);
        this.bind_action(InputAction::East, &[InputBinding::SpecificKey(SdlKey::L)]);
        this.bind_action(InputAction::Southeast, &[InputBinding::SpecificKey(SdlKey::N)]);
        this.bind_action(InputAction::South, &[InputBinding::SpecificKey(SdlKey::J)]);
        this.bind_action(InputAction::Southwest, &[InputBinding::SpecificKey(SdlKey::B)]);
        this.bind_action(InputAction::West, &[InputBinding::SpecificKey(SdlKey::H)]);
        this.bind_action(InputAction::Northwest, &[InputBinding::SpecificKey(SdlKey::Y)]);

        this
    }

    //---------------------------------------------------------------------------------------------
    // Helper function that returns whether an input action binding is pressed.
    //---------------------------------------------------------------------------------------------
    fn binding_pressed(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::SpecificKey(k) => self.pressed_keys.contains(&k),
            InputBinding::ModifierKey(m) => self.modifier_pressed(m),
            InputBinding::ExcludeSpecificKey(k) => !self.pressed_keys.contains(&k),
            InputBinding::ExcludeModifierKey(m) => !self.modifier_pressed(m),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the input manager from current keyboard state.
    // (should be called once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        mouse_coord: Option<(u32, u32)>,
    ) {
        // Update key states.
        //-----------------------------------------------------------------------------------------

        // Iterate over all keys.
        for (scancode, pressed) in keyboard_state.scancodes() {
            if let Some(keycode) = SdlKey::from_scancode(scancode) {
                // If pressed:
                // - insert into the pressed key set.
                // - insert into the just pressed key set if the key had previously been released.
                if pressed {
                    self.pressed_keys.insert(keycode);

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
        self.mouse_clicked.0 = !self.mouse_pressed.0 && mouse_state.left();
        self.mouse_clicked.1 = !self.mouse_pressed.1 && mouse_state.right();

        // Set remaining state.
        self.mouse_pressed.0 = mouse_state.left();
        self.mouse_pressed.1 = mouse_state.right();

        self.mouse_coord = mouse_coord;
    }

    //---------------------------------------------------------------------------------------------
    // Reset the key and action state.
    // (should be called once after input is consumed)
    //---------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        // Clear the key state.
        self.pressed_keys.clear();
        self.just_pressed_keys.clear();

        // Clear the action state.
        self.pressed_actions.clear();
        self.just_pressed_actions.clear();
    }

    //---------------------------------------------------------------------------------------------
    // Returns current pressed state of the mouse buttons
    //---------------------------------------------------------------------------------------------
    pub fn mouse_pressed(&self) -> (bool, bool) {
        self.mouse_pressed
    }

    //---------------------------------------------------------------------------------------------
    // Returns current clicked state of the mouse buttons
    //---------------------------------------------------------------------------------------------
    pub fn mouse_clicked(&self) -> (bool, bool) {
        self.mouse_clicked
    }

    //---------------------------------------------------------------------------------------------
    // Returns current mouse coord within the faux terminal (or none if out of bounds).
    //---------------------------------------------------------------------------------------------
    pub fn mouse_coord(&self) -> Option<(u32, u32)> {
        self.mouse_coord
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether a modifier key is pressed.
    //---------------------------------------------------------------------------------------------
    pub fn modifier_pressed(&self, modifier: &ModifierKey) -> bool {
        match modifier {
            ModifierKey::Alt => {
                self.pressed_keys.contains(&SdlKey::LAlt)
                    || self.pressed_keys.contains(&SdlKey::RAlt)
            }
            ModifierKey::Ctrl => {
                self.pressed_keys.contains(&SdlKey::LCtrl)
                    || self.pressed_keys.contains(&SdlKey::RCtrl)
            }
            ModifierKey::Shift => {
                self.pressed_keys.contains(&SdlKey::LShift)
                    || self.pressed_keys.contains(&SdlKey::RShift)
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether a key is pressed.
    //---------------------------------------------------------------------------------------------
    pub fn key_pressed(&self, key: SdlKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    //---------------------------------------------------------------------------------------------
    // Checks whether a key was just pressed this frame.
    //---------------------------------------------------------------------------------------------
    pub fn key_just_pressed(&self, key: SdlKey) -> bool {
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
        !self.pressed_keys.is_empty()
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether any action is currently pressed.
    //---------------------------------------------------------------------------------------------
    pub fn any_action_pressed(&self) -> bool {
        !self.pressed_actions.is_empty()
    }

    //---------------------------------------------------------------------------------------------
    // Update the key bindings for an action.
    //---------------------------------------------------------------------------------------------
    pub fn bind_action(&mut self, action: InputAction, bindings: &[InputBinding]) {
        // Do not bind empty key set.
        if bindings.is_empty() {
            return;
        }

        // Insert the new action binding.
        self.action_bindings.insert(action, bindings.to_vec());
    }
}
