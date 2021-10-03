//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::input_manager::*;

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the input repeat.
//-------------------------------------------------------------------------------------------------
enum State {
    // The input is currently released.
    Released,
    // The input is currently pressed, but the initial step has not yet passed.
    Pressed,
    // The input is currently pressed and the initial step has passed.
    Held,
}

//-------------------------------------------------------------------------------------------------
// Represents either a key or an input action.
//-------------------------------------------------------------------------------------------------
enum InputKeyOrAction {
    // An InputKey.
    Key(InputKey),
    // An InputAction.
    Action(InputAction),
}

//-------------------------------------------------------------------------------------------------
// Input repeat allows of easy handling of repeated key or action input events.
//-------------------------------------------------------------------------------------------------
pub struct InputRepeat {
    // The tracked key or input action.
    key_or_action: InputKeyOrAction,
    // The current state.
    state: State,
    // The input timer.
    timer: Timer,
    // Duration between firing pressed events when the input has been continually pressed.
    pub held_step: Duration,
    // Duration between when the input is pressed and the first pressed event.
    pub initial_step: Option<Duration>,
}

impl InputRepeat {
    //-----------------------------------------------------------------------------------------------
    // Creates a new input repeat for a key.
    //-----------------------------------------------------------------------------------------------
    pub fn for_key(key: InputKey, held_step: Duration, initial_step: Option<Duration>) -> Self {
        // Set timer initially to either the initial step (if populated) or the held step.
        let timer = match initial_step {
            Some(initial_step) => Timer::new(initial_step),
            _ => Timer::new(held_step),
        };

        Self {
            timer,
            state: State::Released,
            key_or_action: InputKeyOrAction::Key(key),
            held_step,
            initial_step,
        }
    }

    //-----------------------------------------------------------------------------------------------
    // Creates a new input repeat for an action.
    //-----------------------------------------------------------------------------------------------
    pub fn for_action(
        action: InputAction,
        held_step: Duration,
        initial_step: Option<Duration>,
    ) -> Self {
        // Set timer initially to either the initial step (if populated) or the held step.
        let timer = match initial_step {
            Some(initial_step) => Timer::new(initial_step),
            _ => Timer::new(held_step),
        };

        Self {
            timer,
            state: State::Released,
            key_or_action: InputKeyOrAction::Action(action),
            held_step,
            initial_step,
        }
    }

    //-----------------------------------------------------------------------------------------------
    // Sets the input repeat to track a key.
    //-----------------------------------------------------------------------------------------------
    pub fn set_key(&mut self, key: InputKey) {
        self.key_or_action = InputKeyOrAction::Key(key);
    }

    //-----------------------------------------------------------------------------------------------
    // Sets the input repeat to track an action.
    //-----------------------------------------------------------------------------------------------
    pub fn set_action(&mut self, action: InputAction) {
        self.key_or_action = InputKeyOrAction::Action(action);
    }

    //-----------------------------------------------------------------------------------------------
    // Resets the timer to either the initial step (if populated) or the held step.
    //-----------------------------------------------------------------------------------------------
    fn reset_timer(&mut self) {
        self.timer.reset();
        self.timer.interval = match self.initial_step {
            Some(initial_step) => initial_step,
            _ => self.held_step,
        };
    }

    //-----------------------------------------------------------------------------------------------
    // Resets the state of the input repeat.
    //-----------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.state = State::Released;
        self.reset_timer();
    }

    //-----------------------------------------------------------------------------------------------
    // Updates the input repeat and returns the pressed status when the state is released.
    //-----------------------------------------------------------------------------------------------
    fn released_update(&mut self, input: &InputManager) -> bool {
        // If the input has been pressed, update the state, reset the timer, and return true.
        match self.key_or_action {
            InputKeyOrAction::Key(key) => {
                if input.key_just_pressed(key) {
                    self.state = State::Pressed;
                    self.reset_timer();
                    return true;
                }
            }
            InputKeyOrAction::Action(action) => {
                if input.action_just_pressed(action) {
                    self.state = State::Pressed;
                    self.reset_timer();
                    return true;
                }
            }
        }

        // Else return false.
        false
    }

    //-----------------------------------------------------------------------------------------------
    // Updates the input repeat and returns the pressed status when the state is pressed.
    //-----------------------------------------------------------------------------------------------
    fn pressed_update(&mut self, dt: &Duration, input: &InputManager) -> bool {
        // If the input has been released, update the state and return false.
        match self.key_or_action {
            InputKeyOrAction::Key(key) => {
                if !input.key_pressed(key) {
                    self.state = State::Released;
                    return false;
                }
            }
            InputKeyOrAction::Action(action) => {
                if !input.action_pressed(action) {
                    self.state = State::Released;
                    return false;
                }
            }
        }

        // Else if the initial step has passed, update the state and timer and return true.
        if self.timer.update(dt) {
            self.state = State::Held;
            self.timer.interval = self.held_step;
            return true;
        }

        // Else return false.
        false
    }

    //-----------------------------------------------------------------------------------------------
    // Updates the input repeat and returns the pressed status when the state is held.
    //-----------------------------------------------------------------------------------------------
    fn held_update(&mut self, dt: &Duration, input: &InputManager) -> bool {
        // If the input has been released, update the state and return false.
        match self.key_or_action {
            InputKeyOrAction::Key(key) => {
                if !input.key_pressed(key) {
                    self.state = State::Released;
                    return false;
                }
            }
            InputKeyOrAction::Action(action) => {
                if !input.action_pressed(action) {
                    self.state = State::Released;
                    return false;
                }
            }
        }

        // Otherwise, return the timer state.
        self.timer.update(dt)
    }

    //-----------------------------------------------------------------------------------------------
    // Updates the input repeat and returns the pressed status.
    //-----------------------------------------------------------------------------------------------
    pub fn update(&mut self, dt: &Duration, input: &InputManager) -> bool {
        match self.state {
            State::Released => self.released_update(input),
            State::Pressed => self.pressed_update(dt, input),
            State::Held => self.held_update(dt, input),
        }
    }
}
