//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, bail, Result};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;
use fvr_engine_core::prelude::*;
use fvr_engine_server::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::scene_stack::*;
use crate::scenes::scratch::*;
use crate::scenes::transitions::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const FADE_DURATION: Duration = Duration::from_millis(250);
const TITLE_TOP_OFFSET: i32 = 2;
const TITLE_TEXT: &str = r#"
888'Y88 Y8b Y88888P 888 88e      888'Y88 Y88b Y88   e88'Y88  888 Y88b Y88 888'Y88
888 ,'Y  Y8b Y888P  888 888D     888 ,'Y  Y88b Y8  d888  'Y  888  Y88b Y8 888 ,'Y
888C8     Y8b Y8P   888 88"      888C8   b Y88b Y C8888 eeee 888 b Y88b Y 888C8
888 "      Y8b Y    888 b,       888 ",d 8b Y88b   Y888 888P 888 8b Y88b  888 ",d
888         Y8P     888 88b,     888,d88 88b Y88b   "88 88"  888 88b Y88b 888,d88
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~Y8P~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~"#;
const MENU_BUTTONS_OFFSET: i32 = 3;
const VERSION_TEXT: &str = "Alpha v0.0.1";
const COPYRIGHT_TEXT: &str = "Copyright (c) 2021 Waco Paul (wacopaul@pm.me) All Rights Reserved.";

//-------------------------------------------------------------------------------------------------
// Represents the possible states of the main menu scene.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    // The state during the brief initial fade in.
    FadeIn,
    // The state when waiting for the user to pick an option.
    WaitForInput,
    // The state during the brief final fade out to the next scene.
    FadeOut,
}

//-------------------------------------------------------------------------------------------------
// The main menu.
//-------------------------------------------------------------------------------------------------
pub struct MainMenu {
    // The state of the main menu scene.
    state: State,
    // Fade in transition helper.
    fade_in: Fade,
    // Fade out transition helper.
    fade_out: Fade,
    // Contains the final scene action to return after the user has made a selection.
    next_scene: Option<SceneAction>,
    // ButtonList containing the main menu options.
    button_list: ButtonList,
}

impl MainMenu {
    //---------------------------------------------------------------------------------------------
    // Creates a new main menu scene.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        // TODO: Should this be (lazy) static?
        let menu_buttons = vec![
            Button::new((0, 0), String::from("[n] New"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[r] Resume"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[o] Options"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[h] Help"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[c] Credits"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[d] Debug"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[s] Scratch"), ButtonLayout::Text),
            Button::new((0, 0), String::from("[esc] Quit"), ButtonLayout::Text),
        ];

        Self {
            state: State::FadeIn,
            fade_in: Fade::new(&FADE_DURATION, 0.0, 1.0),
            fade_out: Fade::new(&FADE_DURATION, 1.0, 0.0),
            next_scene: None,
            button_list: ButtonList::from_buttons_vec((0, 0), menu_buttons, false),
        }
    }
}

impl Scene for MainMenu {
    //---------------------------------------------------------------------------------------------
    // Called when the scene is added to the stack.
    //---------------------------------------------------------------------------------------------
    fn load(
        &mut self,
        server: &mut Server,
        terminal: &mut Terminal,
        input: &InputManager,
    ) -> Result<()> {
        self.focus(server, terminal, input)?;
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is removed from the stack.
    //---------------------------------------------------------------------------------------------
    fn unload(
        &mut self,
        _server: &mut Server,
        _terminal: &mut Terminal,
        _input: &InputManager,
    ) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called when the scene is made current again (e.g. a the next scene was popped).
    //---------------------------------------------------------------------------------------------
    fn focus(
        &mut self,
        _server: &mut Server,
        terminal: &mut Terminal,
        _input: &InputManager,
    ) -> Result<()> {
        // Reset state.
        self.state = State::FadeIn;
        self.fade_in.reset();
        self.fade_out.reset();
        self.next_scene = None;
        self.button_list.reset();

        // Reset the terminal.
        terminal.set_transparent();
        terminal.set_all_tiles_blank();

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
        let title_xy = ((terminal.width() - title_width as i32) / 2, TITLE_TOP_OFFSET);
        RichTextWriter::write_plain_with_settings(
            terminal,
            title_xy,
            TITLE_TEXT,
            &format_settings,
        );

        format_settings.foreground_color = Some(TileColor::TRANSPARENT);
        format_settings.outline_opacity = Some(0.5);

        // Position and draw the menu buttons.
        let buttons_origin = (
            (terminal.width() - self.button_list.width()) / 2,
            ((terminal.height() - self.button_list.height()) / 2) + MENU_BUTTONS_OFFSET,
        );
        self.button_list.set_origin(buttons_origin);
        self.button_list.redraw(terminal);

        // Draw the version text.
        let version_xy =
            ((terminal.width() - VERSION_TEXT.len() as i32) / 2, terminal.height() - 2);
        RichTextWriter::write_plain_with_settings(
            terminal,
            version_xy,
            VERSION_TEXT,
            &&format_settings,
        );

        // Draw the copyright text.
        let copyright_xy =
            ((terminal.width() - COPYRIGHT_TEXT.len() as i32) / 2, terminal.height() - 1);
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
    fn unfocus(
        &mut self,
        _server: &mut Server,
        _terminal: &mut Terminal,
        _input: &InputManager,
    ) -> Result<()> {
        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (non-visual) internal state should be updated.
    //---------------------------------------------------------------------------------------------
    fn update(
        &mut self,
        _server: &mut Server,
        terminal: &mut Terminal,
        input: &InputManager,
        _dt: &Duration,
    ) -> Result<SceneAction> {
        match self.state {
            State::FadeIn => {
                if self.fade_in.finished() {
                    self.state = State::WaitForInput;
                }
            }
            State::WaitForInput => {
                if input.action_just_pressed(InputAction::Quit)
                    || input.key_just_pressed(InputKey::Escape)
                {
                    return Ok(SceneAction::Pop);
                } else if input.action_just_pressed(InputAction::Accept) {
                    terminal.randomize();
                } else if input.key_just_pressed(InputKey::S) {
                    self.next_scene = Some(SceneAction::Push(Box::new(Scratch::new())));
                    self.state = State::FadeOut;
                } else {
                    let button_list_action = self.button_list.update(input, terminal);

                    // If a button has been triggered, prepare the next scene.
                    if let ButtonListAction::Triggered(i) = button_list_action {
                        match i {
                            // New.
                            0 => {}
                            // Resume.
                            1 => {}
                            // Options.
                            2 => {}
                            // Help.
                            3 => {}
                            // Credits.
                            4 => {}
                            // Debug.
                            5 => {}
                            // Scratch.
                            6 => {
                                self.next_scene =
                                    Some(SceneAction::Push(Box::new(Scratch::new())));
                                self.state = State::FadeOut;
                            }
                            // Quit.
                            7 => {
                                return Ok(SceneAction::Pop);
                            }
                            _ => bail!("Invalid menu option."),
                        }

                        input.set_cursor(Cursor::Hand);
                    } else if button_list_action == ButtonListAction::Interactable {
                        input.set_cursor(Cursor::Hand);
                    } else {
                        input.set_cursor(Cursor::Arrow);
                    }
                }
            }
            State::FadeOut => {
                if self.fade_out.finished() {
                    let next_scene = self
                        .next_scene
                        .take()
                        .ok_or_else(|| anyhow!("Failure: The next scene was empty."))?;
                    return Ok(next_scene);
                }
            }
        }

        Ok(SceneAction::Noop)
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (visual) internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn render(&mut self, terminal: &mut Terminal, dt: &Duration) -> Result<()> {
        match self.state {
            State::FadeIn => {
                let _ = self.fade_in.update(terminal, dt);
            }
            State::FadeOut => {
                let _ = self.fade_out.update(terminal, dt);
            }
            _ => {}
        }

        Ok(())
    }
}
