//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::scene_stack::*;
use crate::scenes::main_menu::*;
use crate::scenes::transitions::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const FADE_DURATION: Duration = Duration::from_millis(750);
const INITIAL_BLANK_INTERVAL: Duration = Duration::from_millis(1500);
const PAUSE_INTERVAL: Duration = Duration::from_millis(2000);
const FINAL_BLANK_INTERVAL: Duration = Duration::from_millis(750);
const LOGO_TEXT: &str = r#"<l:t><st:b><fc:Y>
 _______                                       _
(_______)                                     (_)
 _       ___  ____  ____  _____ ____  _   _    _       ___   ____  ___
| |     / _ \|    \|  _ \(____ |  _ \| | | |  | |     / _ \ / _  |/ _ \
| |____| |_| | | | | |_| / ___ | | | | |_| |  | |____| |_| ( (_| | |_| |
 \______)___/|_|_|_|  __/\_____|_| |_|\__  |  |_______)___/ \___ |\___/
                   |_|               (____/                (_____|



                                ( TODO )"#;

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the initial scene.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    // The initial blank state.
    InitialBlank,
    // The state when the logo text is fading in.
    FadeIn,
    // The state when the logo text pauses at full opacity.
    Pause,
    // The state when the logo text is fading out.
    FadeOut,
    // The final blank state.
    FinalBlank,
    // Ready to swap to the main menu.
    Finished,
}

//-------------------------------------------------------------------------------------------------
// The initial scene will always be loaded first.
//-------------------------------------------------------------------------------------------------
pub struct Initial {
    // The state of the initial scene.
    state: State,
    // Timer for handling timing between state changes.
    timer: Timer,
    // Fade in transition helper.
    fade_in: Fade,
    // Fade out transition helper.
    fade_out: Fade,
}

impl Initial {
    //---------------------------------------------------------------------------------------------
    // Creates a new initial scene.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            state: State::InitialBlank,
            timer: Timer::new(INITIAL_BLANK_INTERVAL),
            fade_in: Fade::new(&FADE_DURATION, 0.0, 1.0),
            fade_out: Fade::new(&FADE_DURATION, 1.0, 0.0),
        }
    }
}

impl Scene for Initial {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(&mut self, terminal: &mut Terminal) -> Result<()> {
        // Reset the terminal.
        terminal.set_transparent();
        terminal.set_all_tiles_default();

        // Find dimensions of the title text.
        let mut logo_width = 0;
        let mut logo_height = 0;

        for line in LOGO_TEXT.lines() {
            if line.len() > logo_width {
                logo_width = line.len();
            }

            logo_height += 1;
        }

        // Ignore the first newline.
        logo_height -= 1;

        // -1 y aligns the logo in the center.
        let logo_xy = (
            (terminal.width() - logo_width as u32) / 2,
            ((terminal.height() - logo_height) / 2) - 1,
        );

        RichTextWriter::write(terminal, logo_xy, LOGO_TEXT)?;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is removed from the stack.
    //---------------------------------------------------------------------------------------------
    fn unload(&mut self, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made current again (e.g. a the next scene was popped).
    //---------------------------------------------------------------------------------------------
    fn focus(&mut self, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made no longer current (e.g. a new scene is pushed).
    //---------------------------------------------------------------------------------------------
    fn unfocus(&mut self, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (non-visual) internal state should be updated.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        dt: &Duration,
        input: &InputManager,
        _terminal: &mut Terminal,
    ) -> Result<SceneAction> {
        if input.any_key_pressed() {
            return Ok(SceneAction::Swap(Box::new(MainMenu::new())));
        }

        match self.state {
            State::InitialBlank => {
                if self.timer.update(dt) {
                    self.state = State::FadeIn;
                }
            }
            State::FadeIn => {
                if self.fade_in.finished() {
                    self.timer.reset();
                    self.timer.interval = PAUSE_INTERVAL;
                    self.state = State::Pause;
                }
            }
            State::Pause => {
                if self.timer.update(dt) {
                    self.state = State::FadeOut;
                }
            }
            State::FadeOut => {
                if self.fade_out.finished() {
                    self.timer.reset();
                    self.timer.interval = FINAL_BLANK_INTERVAL;
                    self.state = State::FinalBlank;
                }
            }
            State::FinalBlank => {
                if self.timer.update(dt) {
                    self.state = State::Finished;
                }
            }
            _ => {}
        }

        if self.state == State::Finished {
            Ok(SceneAction::Swap(Box::new(MainMenu::new())))
        } else {
            Ok(SceneAction::Noop)
        }
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (visual) internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn render(&mut self, dt: &Duration, terminal: &mut Terminal) -> Result<()> {
        match self.state {
            State::FadeIn => {
                let _ = self.fade_in.update(dt, terminal);
            }
            State::FadeOut => {
                let _ = self.fade_out.update(dt, terminal);
            }
            _ => {}
        }

        Ok(())
    }
}
