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

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

const INITIAL_BLANK_INTERVAL: Duration = Duration::from_millis(1500);
const PAUSE_INTERVAL: Duration = Duration::from_millis(1500);
const FINAL_BLANK_INTERVAL: Duration = Duration::from_millis(750);
const LOGO_TEXT: &str = r#" _______                                       _
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
enum State {
    InitialBlank,
    FadeIn,
    Pause,
    FadeOut,
    Finished,
}

//-------------------------------------------------------------------------------------------------
// The initial scene will always be loaded first.
//-------------------------------------------------------------------------------------------------
pub struct Initial {}

impl Initial {}

impl Scene for Initial {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.update_all_tiles(
            Some(' '),
            Some(TileLayout::Text),
            None,
            None,
            None,
            Some(TileColor::TRANSPARENT),
            Some(TileColor::WHITE),
            None,
        );

        let mut logo_width = 0;
        let mut logo_height = 0;

        for line in LOGO_TEXT.lines() {
            if line.len() > logo_width {
                logo_width = line.len();
            }

            logo_height += 1;
        }

        let logo_xy =
            ((terminal.width() - logo_width as u32) / 2, (terminal.height() - logo_height) / 2);

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
        _dt: &Duration,
        input: &InputManager,
        _terminal: &mut Terminal,
    ) -> Result<SceneAction> {
        if input.action_just_pressed(InputAction::Accept) {
            Ok(SceneAction::Pop)
        } else {
            Ok(SceneAction::Noop)
        }
    }
}
