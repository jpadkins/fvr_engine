//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;
use rand::prelude::*;

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
const BACK_BUTTON_TEXT: &str = "◄ [esc] Main Menu";

//-------------------------------------------------------------------------------------------------
// An empty scene used for testing and other development tasks.
//-------------------------------------------------------------------------------------------------
pub struct Scratch {
    back_button: Button,
    scroll_log: ScrollLog,
}

impl Scratch {
    //---------------------------------------------------------------------------------------------
    // Creates a new scratch scene.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            back_button: Button::new((0, 0), BACK_BUTTON_TEXT.into(), ButtonLayout::Text),
            scroll_log: ScrollLog::new((85 - 31, 33 - 11), (31, 11), FrameStyle::LineBlockCorner),
        }
    }
}

impl Scene for Scratch {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(&mut self, input: &InputManager, terminal: &mut Terminal) -> Result<()> {
        self.focus(input, terminal)?;
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is removed from the stack.
    //---------------------------------------------------------------------------------------------
    fn unload(&mut self, _input: &InputManager, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made current again (e.g. a the next scene was popped).
    //---------------------------------------------------------------------------------------------
    fn focus(&mut self, _input: &InputManager, terminal: &mut Terminal) -> Result<()> {
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

        self.scroll_log.append("<l:t><fc:$>Welcome to FVR_ENGINE")?;

        self.scroll_log.redraw(terminal)?;
        self.back_button.redraw(terminal);

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made no longer current (e.g. a new scene is pushed).
    //---------------------------------------------------------------------------------------------
    fn unfocus(&mut self, _input: &InputManager, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (non-visual) internal state should be updated.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        _dt: &Duration,
        input: &InputManager,
        terminal: &mut Terminal,
    ) -> Result<SceneAction> {
        let scroll_log_action = self.scroll_log.update(input, terminal)?;
        let back_button_action = self.back_button.update(input, terminal);

        if input.action_just_pressed(InputAction::Quit)
            || input.key_just_pressed(SdlKey::Escape)
            || back_button_action == ButtonAction::Triggered
        {
            return Ok(SceneAction::Pop);
        }

        if input.action_just_pressed(InputAction::Accept) {
            let mut rng = rand::thread_rng();
            let text = match rng.gen::<u32>() % 5 {
                0 => "\n<l:t><fc:y>> a rat <fc:R>bites<fc:y> YOU for <fc:M>17<fc:y>!",
                1 => "\n<l:t><fc:y>> YOU <fc:B>slash<fc:y> at rat for <fc:M>31<fc:y>!",
                2 => "\n<l:t><fc:y>> You hear clicking in the distance...",
                3 => "\n<l:t><fc:y>> North.",
                4 => "\n<l:t><fc:y>> <fc:G>Poison<fc:y> damages YOU for <fc:M>5<fc:y>!",
                _ => "",
            };
            self.scroll_log.append(text)?;
            self.scroll_log.scroll_to_bottom();
        } else if input.action_just_pressed(InputAction::North) {
            self.scroll_log.scroll_up(1);
        } else if input.action_just_pressed(InputAction::South) {
            self.scroll_log.scroll_down(1);
        } else if scroll_log_action == ScrollLogAction::Interactable {
            input.set_cursor(Cursor::Hand);
        } else if back_button_action == ButtonAction::Interactable {
            input.set_cursor(Cursor::Hand);
        } else {
            input.set_cursor(Cursor::Arrow);
        }

        Ok(SceneAction::Noop)
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (visual) internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn render(&mut self, _dt: &Duration, _terminal: &mut Terminal) -> Result<()> {
        Ok(())
    }
}
