use anyhow::Result;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use fvr_engine_client::prelude::*;

fn main() -> Result<()> {
    let mut update_timer = 0;
    let mut client = Client::new()?;

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

        let key_state = client.key_state();
        if key_state.contains(&Keycode::R) {
            client.randomize_terminal();
        }

        update_timer += client.render_frame()?;

        // TODO: Remove
        if update_timer >= 3000 {
            println!("3 seconds!");
            update_timer -= 3000;
        } else {
            ::std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    Ok(())
}
