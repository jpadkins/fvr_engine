//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::Result;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_client::prelude::*;
use fvr_engine_core::prelude::*;

// TODO: Load these from config.
const WINDOW_TITLE: &str = "FVR_ENGINE";
const WINDOW_DIMENSIONS: (u32, u32) = (800, 600);
const TERMINAL_DIMENSIONS: (u32, u32) = (81, 31); // 103, 37.
const TILE_DIMENSIONS: (u32, u32) = (48, 64);
const FONT_NAME: &str = "deja_vu_sans_mono";

const TEST_STR: &str = r#"<l:t><st:bi><o:f><fc:Y><bc:T>Hello
World
This-
Text-
has a
Seven
Lines"#;

fn main() -> Result<()> {
    let mut client = Client::new(
        WINDOW_TITLE,
        WINDOW_DIMENSIONS,
        TERMINAL_DIMENSIONS,
        TILE_DIMENSIONS,
        FONT_NAME,
    )?;
    let mut terminal = client.create_terminal();
    let mut input = InputManager::with_default_bindings();

    let mut text_wrapper = RichTextWrapper::new(20, 3);
    text_wrapper.append(TEST_STR)?;

    let mut accept_repeat = InputRepeat::for_action(
        InputAction::Accept,
        Duration::from_millis(250),
        Some(Duration::from_millis(500)),
    );

    let mut dt;

    'main: loop {
        while let Some(event) = client.poll_event() {
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } | Event::Quit { .. } => {
                    break 'main
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    client.toggle_debug();
                }
                _ => {}
            }
        }

        dt = client.update_input(&mut input);

        if accept_repeat.update(&dt, &input) {
            terminal.randomize();
            text_wrapper.draw(&mut terminal, (0, 0))?;
        }

        if input.action_just_pressed(InputAction::North) {
            text_wrapper.scroll_up(1);
            text_wrapper.draw(&mut terminal, (0, 0))?;
        }

        if input.action_just_pressed(InputAction::South) {
            text_wrapper.scroll_down(1);
            text_wrapper.draw(&mut terminal, (0, 0))?;
        }

        if let Some(coord) = input.mouse_coord() {
            if input.mouse_pressed().0 {
                terminal.update_tile_fields(
                    coord,
                    Some('#'),
                    None,
                    Some(TileStyle::Regular),
                    None,
                    Some(false),
                    Some(TileColor::BLUE),
                    Some(TileColor::RED),
                    None,
                );
            }
        }
        input.reset();

        client.render_frame(&terminal)?;
    }

    Ok(())
}
