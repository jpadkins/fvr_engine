use std::time::Duration;

use anyhow::Result;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use fvr_engine_client::prelude::*;

// TODO: Load these from config.
const WINDOW_TITLE: &str = "FVR_ENGINE";
const WINDOW_DIMENSIONS: (u32, u32) = (800, 600);
const TERMINAL_DIMENSIONS: (u32, u32) = (81, 31); // 103, 37.
const TILE_DIMENSIONS: (u32, u32) = (48, 64);
const TEXTURE_PATH: &str = "./resources/font_atlases/deja_vu_sans_mono.png";
const METRICS_PATH: &str = "./resources/font_atlases/deja_vu_sans_mono.toml";

fn main() -> Result<()> {
    let mut update_timer = Duration::default();
    let mut client = Client::new(
        WINDOW_TITLE,
        WINDOW_DIMENSIONS,
        TERMINAL_DIMENSIONS,
        TILE_DIMENSIONS,
        TEXTURE_PATH,
        METRICS_PATH,
    )?;
    let mut terminal = client.create_terminal();

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

        if update_timer.as_millis() >= 16 {
            let key_state = client.key_state();
            if key_state.contains(&Keycode::R) {
                terminal.randomize();
            }

            update_timer -= Duration::from_millis(16);
        }

        update_timer += client.render_frame(&terminal)?;
    }

    Ok(())
}
