//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Result};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::scene_stack::*;
use crate::scenes::transitions::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const OPACITY_STEP: f32 = 0.025;

//   ________ ___      ___ ________   _______   ________   ________  ___  ________   _______
// |\  _____\\  \    /  /|\   __  \ |\  ___ \ |\   ___  \|\   ____\|\  \|\   ___  \|\  ___ \
//  \ \  \__/\ \  \  /  / | \  \|\  \\ \   __/|\ \  \\ \  \ \  \___|\ \  \ \  \\ \  \ \   __/|
//   \ \   __\\ \  \/  / / \ \   _  _\\ \  \_|/_\ \  \\ \  \ \  \  __\ \  \ \  \\ \  \ \  \_|/__
//    \ \  \_| \ \    / /   \ \  \\  \|\ \  \_|\ \ \  \\ \  \ \  \|\  \ \  \ \  \\ \  \ \  \_|\ \
//     \ \__\   \ \__/ /     \ \__\\ _\ \ \_______\ \__\\ \__\ \_______\ \__\ \__\\ \__\ \_______\
//      \|__|    \|__|/       \|__|\|__| \|_______|\|__| \|__|\|_______|\|__|\|__| \|__|\|_______|

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the main menu scene.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    FadeIn,
    WaitForInput,
    FadeOut,
}

//-------------------------------------------------------------------------------------------------
// The main menu.
//-------------------------------------------------------------------------------------------------
pub struct MainMenu {
    state: State,
    fade_in: FadeIn,
    fade_out: FadeOut,
    next_scene: Option<SceneAction>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            state: State::FadeIn,
            fade_in: FadeIn::new(OPACITY_STEP, 0.0, 1.0),
            fade_out: FadeOut::new(OPACITY_STEP, 1.0, 0.0),
            next_scene: None,
        }
    }
}

impl Scene for MainMenu {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.set_transparent();

        terminal.update_all_tiles(
            Some(' '),
            Some(TileLayout::Text),
            Some(TileStyle::Bold),
            None,
            Some(false),
            Some(TileColor::TRANSPARENT),
            Some(TileColor::WHITE),
            None,
        );

        terminal.randomize();

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
        self.state = State::FadeIn;
        self.fade_in.reset();
        self.fade_out.reset();
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
        match self.state {
            State::FadeIn => {
                if self.fade_in.update(terminal) {
                    self.state = State::WaitForInput;
                }
            }
            State::WaitForInput => {
                if input.action_pressed(InputAction::Accept) {
                    terminal.randomize();
                } else if input.action_pressed(InputAction::Decline) {
                    self.state = State::FadeOut;
                    self.next_scene = Some(SceneAction::Pop);
                }
            }
            State::FadeOut => {
                if self.fade_out.update(terminal) {
                    let next_scene = self
                        .next_scene
                        .take()
                        .ok_or(anyhow!("Failure: The next scene was empty."))?;
                    return Ok(next_scene);
                }
            }
            _ => {}
        }

        Ok(SceneAction::Noop)
    }
}
