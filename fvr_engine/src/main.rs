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

    let mut test_wrapper = RichTextWrapper::new(20, 2);
    test_wrapper.append("<l:c>Hello,<<foo> world! Can't <fc:r>BELIEVE<fc:Y> this is working!!")?;
    test_wrapper.draw((0, 0));
    println!("-------------------");
    test_wrapper.scroll_down(1);
    test_wrapper.draw((0, 0));

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

        client.update_input(&mut input);

        if input.action_pressed(InputAction::Accept) {
            terminal.randomize();

            RichTextWriter::write(
                &mut terminal,
                (0, 0),
                "<l:t><fc:Y><bc:T><o:f><st:r>Hello, world!",
            )?;
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

        let _ = client.render_frame(&terminal)?;
    }

    Ok(())
}
