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
const TITLE_TOP_OFFSET: u32 = 2;
const TITLE_TEXT: &str = r#"
888'Y88 Y8b Y88888P 888 88e      888'Y88 Y88b Y88   e88'Y88  888 Y88b Y88 888'Y88
888 ,'Y  Y8b Y888P  888 888D     888 ,'Y  Y88b Y8  d888  'Y  888  Y88b Y8 888 ,'Y
888C8     Y8b Y8P   888 88"      888C8   b Y88b Y C8888 eeee 888 b Y88b Y 888C8
888 "      Y8b Y    888 b,       888 ",d 8b Y88b   Y888 888P 888 8b Y88b  888 ",d
888         Y8P     888 88b,     888,d88 88b Y88b   "88 88"  888 88b Y88b 888,d88
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~Y8P~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~"#;
const VERSION_TEXT: &str = "Alpha v0.0.1";
const COPYRIGHT_TEXT: &str =
    "Copyright (c) 2019-2021 Waco Paul (wacopaul@pm.me) All Rights Reserved.";

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
        // Reset state.
        self.state = State::FadeIn;
        self.fade_in.reset();
        self.fade_out.reset();

        // Reset the terminal.
        terminal.set_transparent();
        terminal.set_all_tiles_default();

        // Find dimensions of the title text.
        let mut title_width = 0;

        for line in TITLE_TEXT.lines() {
            if line.len() > title_width {
                title_width = line.len();
            }
        }

        let mut format_settings = RichTextFormatSettings {
            layout: Some(TileLayout::Text),
            style: Some(TileStyle::Bold),
            outlined: Some(true),
            foreground_color: Some(PaletteColor::DarkGrey.into()),
            outline_color: Some(PaletteColor::White.into()),
            ..Default::default()
        };

        // Draw the title text
        let title_xy = ((terminal.width() - title_width as u32) / 2, TITLE_TOP_OFFSET);
        RichTextWriter::write_plain_with_settings(
            terminal,
            title_xy,
            TITLE_TEXT,
            &format_settings,
        );

        format_settings.foreground_color = Some(TileColor::TRANSPARENT);
        format_settings.outline_opacity = Some(0.5);

        // Draw the version text.
        let version_xy =
            ((terminal.width() - VERSION_TEXT.len() as u32) / 2, terminal.height() - 2);
        RichTextWriter::write_plain_with_settings(
            terminal,
            version_xy,
            VERSION_TEXT,
            &&format_settings,
        );

        // Draw the copyright text.
        let copyright_xy =
            ((terminal.width() - COPYRIGHT_TEXT.len() as u32) / 2, terminal.height() - 1);
        RichTextWriter::write_plain_with_settings(
            terminal,
            copyright_xy,
            COPYRIGHT_TEXT,
            &&format_settings,
        );

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
        }

        Ok(SceneAction::Noop)
    }
}
