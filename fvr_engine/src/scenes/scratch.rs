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
use fvr_engine_core::{map2d_iter_mut, prelude::*};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::scene_stack::*;

const BACK_BUTTON_TEXT: &str = "◄ [esc] Main Menu";

//-------------------------------------------------------------------------------------------------
// An empty scene used for testing and other development tasks.
//-------------------------------------------------------------------------------------------------
pub struct Scratch {
    back_button: Button,
    scroll_log: ScrollLog,
    astar: AStar,
    path: Vec<UCoord>,
    passability: GridMap<Passability>,
}

impl Scratch {
    //---------------------------------------------------------------------------------------------
    // Creates a new scratch scene.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            back_button: Button::new((0, 0), BACK_BUTTON_TEXT.into(), ButtonLayout::Text),
            scroll_log: ScrollLog::new(
                (85 - 30, 33 - 11),
                (30, 11),
                FrameStyle::LineBlockCorner,
                9,
            ),
            astar: AStar::fast(Distance::Euclidean),
            path: Vec::new(),
            passability: GridMap::new((55, 33)),
        }
    }

    fn blit(&mut self, terminal: &mut Terminal, start: UCoord) {
        for x in 0..55 {
            for y in 0..33 {
                terminal.get_xy_mut((x, y)).background_opacity = 1.0;
                terminal.get_xy_mut((x, y)).background_color = TileColor::TRANSPARENT;
            }
        }

        self.path.clear();
        self.astar.push_path(start, (28, 17), &self.passability, None, &mut self.path);

        for xy in self.path.iter().skip(1) {
            terminal.get_xy_mut(*xy).background_opacity = 0.25;
            terminal.get_xy_mut(*xy).background_color = PaletteColor::Gold.const_into();
        }

        terminal.get_xy_mut((28, 17)).glyph = '@';
        terminal.get_xy_mut((28, 17)).foreground_color = TileColor::WHITE;
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

        map2d_iter_mut!(terminal, tile, {
            tile.glyph = ' ';
            tile.foreground_color = TileColor::WHITE;
        });

        let mut rng = rand::thread_rng();

        for x in 0..55 {
            for y in 0..33 {
                if rng.gen::<u32>() % 4 == 0 {
                    terminal.get_xy_mut((x, y)).glyph = 'T';
                    terminal.get_xy_mut((x, y)).foreground_color =
                        PaletteColor::BrightGreen.into();
                    *self.passability.get_xy_mut((x, y)) = Passability::Blocked;
                } else {
                    terminal.get_xy_mut((x, y)).glyph = '.';
                    terminal.get_xy_mut((x, y)).foreground_color = PaletteColor::DarkGreen.into();
                    *self.passability.get_xy_mut((x, y)) = Passability::Passable;
                }

                terminal.get_xy_mut((x, y)).background_color = TileColor::TRANSPARENT;
            }
        }
        *self.passability.get_xy_mut((28, 17)) = Passability::Passable;

        let mut stats_frame =
            Frame::new((85 - 30, 0), (28, 33 - 11 - 1), FrameStyle::LineBlockCorner);
        stats_frame.top_left_text = Some("<character name>".into());
        stats_frame.draw(terminal)?;

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
        if input.mouse_moved() || input.action_just_pressed(InputAction::Decline) {
            if let Some(xy) = input.mouse_coord() {
                if xy.0 < 55 {
                    self.blit(terminal, xy);
                }
            }
        }

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
        } else if scroll_log_action == ScrollLogAction::Interactable
            || back_button_action == ButtonAction::Interactable
        {
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
