//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the fade transition.
//-------------------------------------------------------------------------------------------------
#[derive(PartialEq, Eq)]
enum State {
    // The initial state when the transition begins.
    Initial,
    // The state when the terminal is fading in.
    Fading,
    // The final state when the transition ends.
    Finished,
}

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the fade transition.
//-------------------------------------------------------------------------------------------------
pub struct Fade {
    // Current state of the fade transition.
    state: State,
    // Total duration of the transition.
    pub timespan: Duration,
    // Initial opacity value.
    pub initial_opacity: f32,
    // Final opacity value.
    pub final_opacity: f32,
}

impl Fade {
    //---------------------------------------------------------------------------------------------
    // Creates a new fade transition.
    //---------------------------------------------------------------------------------------------
    pub fn new(timespan: &Duration, initial_opacity: f32, final_opacity: f32) -> Self {
        Self {
            state: State::Initial,
            timespan: *timespan,
            initial_opacity: initial_opacity.clamp(0.0, 1.0),
            final_opacity: final_opacity.clamp(0.0, 1.0),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the fade trasition.
    // (should be called once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn update(&mut self, dt: &Duration, terminal: &mut Terminal) -> bool {
        match self.state {
            // Set the terminal to the initial opacity and set the state to fading in.
            State::Initial => {
                terminal.set_opacity(self.initial_opacity);
                self.state = State::Fading;
            }
            // Increment the opacity and set the state to finished if the final opacity is reached.
            State::Fading => {
                // Find the diff between the initial and final opacity.
                let diff = (self.final_opacity - self.initial_opacity).abs();

                // Find the corresponding change in opacity.
                let change = diff * (dt.as_secs_f32() / self.timespan.as_secs_f32());

                // Update the opacity and check if the final opacity has been met.
                if self.initial_opacity < self.final_opacity {
                    terminal.set_opacity(terminal.opacity() + change);
                    if terminal.opacity() >= self.final_opacity {
                        terminal.set_opacity(self.final_opacity);
                        self.state = State::Finished;
                    }
                } else {
                    terminal.set_opacity(terminal.opacity() - change);
                    if terminal.opacity() <= self.final_opacity {
                        terminal.set_opacity(self.final_opacity);
                        self.state = State::Finished;
                    }
                }
            }
            // Return true when finished.
            State::Finished => {
                return true;
            }
        }

        false
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the animation has finished.
    //---------------------------------------------------------------------------------------------
    pub fn finished(&self) -> bool {
        self.state == State::Finished
    }

    //---------------------------------------------------------------------------------------------
    // Resets the state of the fade transition.
    //---------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.state = State::Initial;
    }
}
