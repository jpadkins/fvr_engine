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
use scenes::Initial;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
// TODO: Load these from config.
const WINDOW_TITLE: &str = "FVR_ENGINE";
const WINDOW_DIMENSIONS: (u32, u32) = (1280, 720);
const TERMINAL_DIMENSIONS: (u32, u32) = (85, 33);
const TILE_DIMENSIONS: (u32, u32) = (48, 64);
const FONT_NAME: &str = "deja_vu_sans_mono";
const UPDATE_INTERVAL: Duration = Duration::from_millis(1000 / 60);

fn main() -> Result<()> {
    let mut client = Client::new(
        WINDOW_TITLE,
        WINDOW_DIMENSIONS,
        TERMINAL_DIMENSIONS,
        TILE_DIMENSIONS,
        FONT_NAME,
    )?;
    let mut terminal = client.create_terminal();
    let mut input = InputManager::with_default_bindings()?;

    let mut render_dt;
    let mut update_dt = Duration::from_secs(0);
    let mut update_timer = Timer::new(UPDATE_INTERVAL);

    let mut scene_stack = SceneStack::new();
    scene_stack.push(Box::new(Initial::new()), &input, &mut terminal)?;

    'main: loop {
        while let Some(event) = client.poll_event() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    client.toggle_debug();
                }
                _ => {}
            }
        }

        render_dt = client.update_input(&mut input);
        update_dt += render_dt;

        if update_timer.update(&render_dt) {
            if !scene_stack.update(&update_dt, &input, &mut terminal)? {
                break 'main;
            }

            input.reset();
            update_dt -= UPDATE_INTERVAL;
        }

        scene_stack.render(&render_dt, &mut terminal)?;

        let _ = client.render_frame(&terminal)?;
    }

    Ok(())
}
