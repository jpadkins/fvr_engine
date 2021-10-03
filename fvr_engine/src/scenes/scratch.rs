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
use fvr_engine_server::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::scene_stack::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const SHOW_FOV: bool = false;

//-------------------------------------------------------------------------------------------------
// An empty scene used for testing and other development tasks.
//-------------------------------------------------------------------------------------------------
pub struct Scratch {
    scroll_log: ScrollLog,
    view: Rect,
    path: Vec<ICoord>,
    a_star: AStar,
    last_offset: ICoord,
    show_path: bool,
}

impl Scratch {
    //---------------------------------------------------------------------------------------------
    // Creates a new scratch scene.
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            scroll_log: ScrollLog::new(
                (85 - 30, 33 - 11),
                (30, 11),
                FrameStyle::LineBlockCorner,
                9,
            ),
            view: Rect::new((0, 0), 55, 33),
            path: Vec::new(),
            a_star: AStar::fast(Distance::Euclidean),
            last_offset: (0, 0),
            show_path: false,
        }
    }

    fn handle_move(
        &mut self,
        server: &mut Server,
        terminal: &mut Terminal,
        direction: &Direction,
    ) -> Result<()> {
        let _ = server.move_player(*direction);

        self.last_offset = server.blit_centered_on_player(terminal, (55, 33), (0, 0), SHOW_FOV);

        Ok(())
    }

    fn handle_teleport(
        &mut self,
        server: &mut Server,
        terminal: &mut Terminal,
        xy: ICoord,
    ) -> Result<()> {
        let zone_xy = (xy.0 + self.last_offset.0, xy.1 + self.last_offset.1);
        // let response = server.handle(Request::Teleport(zone_xy));

        // match response {
        //     Response::Fail(resp) => {
        //         if let Some(msg) = resp {
        //             self.scroll_log.append(&format!("\n<fc:y>> {}", msg))?;
        //         }
        //     }
        //     Response::Success(resp) => {
        //         if let Some(msg) = resp {
        //             self.scroll_log.append(&format!("\n<fc:y>> {}", msg))?;
        //         }
        //     }
        // }

        // self.last_offset = server.blit_player_centered(terminal, (55, 33), (0, 0), true);
        self.last_offset = server.blit_centered(terminal, zone_xy, (55, 33), (0, 0), SHOW_FOV);

        Ok(())
    }

    fn draw_path(&mut self, server: &mut Server, terminal: &mut Terminal, xy: ICoord) {
        if !self.show_path {
            return;
        }

        self.last_offset = server.blit_centered_on_player(terminal, (55, 33), (0, 0), SHOW_FOV);
        let rect = Rect::new(self.last_offset, 55, 33);
        let player_xy = server.zone().player_xy;

        self.path.clear();
        self.a_star.push_path(
            player_xy,
            rect.insert_xy(xy),
            &server.zone().pathing,
            None,
            &mut self.path,
        );

        for coord in self.path.iter().rev().skip(1) {
            if let Some(norm) = &Rect::new(self.last_offset, 55, 33).extract_xy(*coord) {
                let tile = terminal.get_xy_mut(*norm);
                tile.background_color = PaletteColor::Gold.const_into();
                tile.background_opacity = 0.25;
            }
        }
    }
}

impl Scene for Scratch {
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
        server: &mut Server,
        terminal: &mut Terminal,
        _input: &InputManager,
    ) -> Result<()> {
        terminal.set_opaque();
        terminal.set_all_tiles_blank();

        *server = Server::new()?;
        self.last_offset = server.blit_centered_on_player(terminal, (55, 33), (0, 0), SHOW_FOV);

        let mut stats_frame =
            Frame::new((85 - 30, 0), (28, 33 - 11 - 1), FrameStyle::LineBlockCorner);
        stats_frame.top_left_text = Some("<character name>".into());
        stats_frame.draw(terminal)?;

        self.scroll_log.append("<l:t><fc:$>Welcome to FVR_ENGINE")?;
        self.scroll_log.redraw(terminal)?;

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
        server: &mut Server,
        terminal: &mut Terminal,
        input: &InputManager,
        _dt: &Duration,
    ) -> Result<SceneAction> {
        let scroll_log_action = self.scroll_log.update(input, terminal)?;

        if input.action_just_pressed(InputAction::Quit) || input.key_just_pressed(SdlKey::Escape) {
            return Ok(SceneAction::Pop);
        } else if input.action_just_pressed(InputAction::Accept) {
            let _ = server.tick();
            self.last_offset =
                server.blit_centered_on_player(terminal, (55, 33), (0, 0), SHOW_FOV);
        } else if input.action_just_pressed(InputAction::North) {
            self.handle_move(server, terminal, &NORTH_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::South) {
            self.handle_move(server, terminal, &SOUTH_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::East) {
            self.handle_move(server, terminal, &EAST_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::West) {
            self.handle_move(server, terminal, &WEST_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::Northeast) {
            self.handle_move(server, terminal, &NORTHEAST_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::Southeast) {
            self.handle_move(server, terminal, &SOUTHEAST_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::Southwest) {
            self.handle_move(server, terminal, &SOUTHWEST_DIRECTION)?;
        } else if input.action_just_pressed(InputAction::Northwest) {
            self.handle_move(server, terminal, &NORTHWEST_DIRECTION)?;
        } else if scroll_log_action == ScrollLogAction::Interactable {
            input.set_cursor(Cursor::Hand);
        } else if input.mouse_moved() {
            if let Some(xy) = input.mouse_coord() {
                self.draw_path(server, terminal, xy);
                let zone_xy = (self.last_offset.0 + xy.0, self.last_offset.1 + xy.1);
                self.scroll_log.append(&format!("\n<fc:y>> mouse: <fc:$>{:?}", zone_xy))?;
            }
        } else {
            input.set_cursor(Cursor::Arrow);
        }

        Ok(SceneAction::Noop)
    }

    //---------------------------------------------------------------------------------------------
    // Called whenever the scene's (visual) internal state should be updated and rendered.
    //---------------------------------------------------------------------------------------------
    fn render(&mut self, _terminal: &mut Terminal, _dt: &Duration) -> Result<()> {
        Ok(())
    }
}
