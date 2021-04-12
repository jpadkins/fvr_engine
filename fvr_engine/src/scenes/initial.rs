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
const OPACITY_STEP: f32 = 0.025;
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
    InitialBlank,
    FadeIn,
    Pause,
    FadeOut,
    FinalBlank,
    Finished,
}

//-------------------------------------------------------------------------------------------------
// The initial scene will always be loaded first.
//-------------------------------------------------------------------------------------------------
pub struct Initial {
    state: State,
    timer: Timer,
    fade_in: FadeIn,
    fade_out: FadeOut,
}

impl Initial {
    pub fn new() -> Self {
        Self {
            state: State::InitialBlank,
            timer: Timer::new(INITIAL_BLANK_INTERVAL),
            fade_in: FadeIn::new(OPACITY_STEP, 0.0, 1.0),
            fade_out: FadeOut::new(OPACITY_STEP, 1.0, 0.0),
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
    // Called whenever the scene's internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        dt: &Duration,
        input: &InputManager,
        terminal: &mut Terminal,
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
                if self.fade_in.update(terminal) {
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
                if self.fade_out.update(terminal) {
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

        if self.state == State::FinalBlank {
            Ok(SceneAction::Swap(Box::new(MainMenu::new())))
        } else {
            Ok(SceneAction::Noop)
        }
    }
}
