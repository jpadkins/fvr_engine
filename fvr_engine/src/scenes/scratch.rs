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

const SCRATCH_TEXT: &str = "<l:t><fc:Y>This is the scratch scene. Should you be here?";

//-------------------------------------------------------------------------------------------------
// An empty scene used for testing and other development tasks.
//-------------------------------------------------------------------------------------------------
pub struct Scratch;

impl Scene for Scratch {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(&mut self, terminal: &mut Terminal) -> Result<()> {
        self.focus(terminal)?;
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
    fn focus(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.set_opaque();
        terminal.set_all_tiles_default();
        for tile in terminal.iter_mut() {
            tile.glyph = '.';
            tile.foreground_color = TileColor::WHITE;
        }

        let scratch_text_len = RichTextWriter::stripped_len(SCRATCH_TEXT)?;

        RichTextWriter::write(
            terminal,
            ((terminal.width() - scratch_text_len as u32) / 2, 1),
            SCRATCH_TEXT,
        )?;

        let mut frame = Frame::new((0, 2), (21, 21), FrameStyle::Line);
        frame.top_left_text = Some("Items".into());
        frame.top_right_text = Some("Stats".into());
        frame.bottom_right_text = Some("Spells".into());
        frame.bottom_left_text = Some("Skills".into());
        frame.draw(terminal)?;

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
        _dt: &Duration,
        input: &InputManager,
        _terminal: &mut Terminal,
    ) -> Result<SceneAction> {
        if input.action_just_pressed(InputAction::Quit) || input.key_just_pressed(SdlKey::Escape) {
            Ok(SceneAction::Pop)
        } else {
            Ok(SceneAction::Noop)
        }
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (visual) internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn render(&mut self, _dt: &Duration, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }
}
