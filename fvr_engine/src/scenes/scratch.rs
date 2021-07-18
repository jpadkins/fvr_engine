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
    frame: Frame,
    scrollbar: Scrollbar,
    wrapper: RichTextWrapper,
}

impl Scratch {
    //---------------------------------------------------------------------------------------------
    // Creates a new scratch scene.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            back_button: Button::new((0, 0), BACK_BUTTON_TEXT.into(), ButtonLayout::Text),
            frame: Frame::new((0, 2), (41, 21), FrameStyle::Fancy),
            scrollbar: Scrollbar::new((42, 3), 21, 0),
            wrapper: RichTextWrapper::new(41, 21),
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

        self.frame.draw_and_clear(terminal)?;
        self.wrapper.draw(terminal, (1, 3))?;
        self.scrollbar.redraw(terminal);
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
        let scrollbar_action = self.scrollbar.update(input, terminal);
        let back_button_action = self.back_button.update(input, terminal);

        if input.action_just_pressed(InputAction::Quit)
            || input.key_just_pressed(SdlKey::Escape)
            || back_button_action == ButtonAction::Triggered
        {
            return Ok(SceneAction::Pop);
        }

        if input.action_just_pressed(InputAction::Accept) {
            let mut rng = rand::thread_rng();
            let hint = match rng.gen::<u32>() % 5 {
                0 => "<fc:R>",
                1 => "<fc:G>",
                2 => "<fc:B>",
                3 => "<fc:W>",
                4 => "<fc:M>",
                _ => "",
            };
            const text: &str =
                "<l:t>Hello! This is some example text. Just a long string that should wrap.";
            self.wrapper.append(&format!("{}{}", hint, text))?;
            self.wrapper.draw(terminal, (1, 3))?;
            self.scrollbar.set_content_height(self.wrapper.total_lines());
            println!("total lines: {}", self.wrapper.total_lines());
        } else if input.action_just_pressed(InputAction::North) {
            self.wrapper.scroll_up(1);
            self.wrapper.draw(terminal, (1, 3))?;
            self.scrollbar.set_current_line(self.wrapper.lines_up());
        } else if input.action_just_pressed(InputAction::South) {
            self.wrapper.scroll_down(1);
            self.wrapper.draw(terminal, (1, 3))?;
            self.scrollbar.set_current_line(self.wrapper.lines_up());
        } else if let ScrollbarAction::ScrollUp(lines) = scrollbar_action {
            self.wrapper.scroll_up(lines);
            self.wrapper.draw(terminal, (1, 3))?;
            self.scrollbar.set_current_line(self.wrapper.lines_up());
            input.set_cursor(Cursor::Hand);
        } else if let ScrollbarAction::ScrollDown(lines) = scrollbar_action {
            self.wrapper.scroll_down(lines);
            self.wrapper.draw(terminal, (1, 3))?;
            self.scrollbar.set_current_line(self.wrapper.lines_up());
            input.set_cursor(Cursor::Hand);
        } else if scrollbar_action == ScrollbarAction::Interactable {
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
