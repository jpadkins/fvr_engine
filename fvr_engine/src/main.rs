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

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
mod scene_stack;
use scene_stack::*;

mod scenes;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
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

    let mut dt = Duration::from_secs(0);
    let mut update = true;

    let mut scene_stack = SceneStack::new();
    scene_stack.push(Box::new(scenes::Initial {}), &mut terminal)?;

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

        dt += client.update_input(&mut input);

        if update {
            if !scene_stack.update(&dt, &input, &mut terminal)? {
                break 'main;
            }

            input.reset();
            dt = Duration::from_secs(0);
        }

        update = client.render_frame(&terminal)?;
    }

    Ok(())
}
