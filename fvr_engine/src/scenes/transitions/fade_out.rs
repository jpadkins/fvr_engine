//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the fade out transition.
//-------------------------------------------------------------------------------------------------
enum State {
    // The initial state when the transition begins.
    Initial,
    // The state when the terminal is fading in.
    FadingOut,
    // The final state when the transition ends.
    Finished,
}

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the fade out transition.
//-------------------------------------------------------------------------------------------------
pub struct FadeOut {
    // Current state of the fade out transition.
    state: State,
    // Amount to decrement the opacity each update.
    pub opacity_step: f32,
    // Initial opacity value.
    pub initial_opacity: f32,
    // Final opacity value.
    pub final_opacity: f32,
}

impl FadeOut {
    //---------------------------------------------------------------------------------------------
    // Creates a new fade out transition.
    //---------------------------------------------------------------------------------------------
    pub fn new(opacity_step: f32, initial_opacity: f32, final_opacity: f32) -> Self {
        Self {
            state: State::Initial,
            opacity_step: opacity_step.clamp(0.0, 1.0),
            initial_opacity: initial_opacity.clamp(0.0, 1.0),
            final_opacity: final_opacity.clamp(0.0, 1.0),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the fade out trasition.
    // (should be called once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn update(&mut self, terminal: &mut Terminal) -> bool {
        match self.state {
            // Set the terminal to the initial opacity and set the state to fading in.
            State::Initial => {
                terminal.set_opacity(self.initial_opacity);
                self.state = State::FadingOut;
            }
            // Increment the opacity and set the state to finished if the final opacity is reached.
            State::FadingOut => {
                terminal.set_opacity(terminal.opacity() - self.opacity_step);

                if terminal.opacity() <= self.final_opacity {
                    terminal.set_opacity(self.final_opacity);
                    self.state = State::Finished;
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
    // Resets the state of the fade out transition.
    //---------------------------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.state = State::Initial;
    }
}
