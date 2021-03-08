use anyhow::Result;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use fvr_engine_client::prelude::*;

fn main() -> Result<(), String> {
    let mut client = Client::new()?;

    'main: loop {
        while let Some(event) = client.poll_event() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    client.toggle_debug();
                }
                _ => {}
            }
        }

        client.render_frame()?;
    }

    Ok(())
}
