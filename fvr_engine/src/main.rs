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
mod scene_stack;
use scene_stack::*;

mod scenes;
use scenes::Initial;

//-------------------------------------------------------------------------------------------------
// Main.
//-------------------------------------------------------------------------------------------------
fn main() -> Result<()> {
    // Initialize everything.
    let mut render_dt;
    let mut update_dt = Duration::from_secs(0);
    let mut update_timer = Timer::new(CONFIG.update_interval);
    let mut server = Server::new()?;
    let mut client = Client::new()?;
    let mut terminal = Terminal::default();
    let mut input = InputManager::with_default_bindings()?;
    let mut scene_stack = SceneStack::new();
    scene_stack.push(Box::new(Initial::new()), &mut server, &mut terminal, &input)?;

    // Begin the game loop.
    'main: loop {
        while let Some(event) = client.poll_event() {
            match event {
                // Break immediately if quit event is received.
                InputEvent::Quit { .. } => break 'main,
                // Toggle the debug gui on space.
                // TODO: Change this, obviously.
                InputEvent::KeyDown { keycode: Some(InputKey::Space), .. } => {
                    client.toggle_debug();
                }
                _ => {}
            }
        }

        // Update the frame time counters.
        render_dt = client.update_input(&mut input);
        update_dt += render_dt;

        // If enough time has passed, update the game state.
        if update_timer.update(&render_dt) {
            if !scene_stack.update(&mut server, &mut terminal, &input, &update_dt)? {
                break 'main;
            }

            input.reset();
            update_dt -= CONFIG.update_interval;
        }

        // Always render the frame.
        scene_stack.render(&mut terminal, &render_dt)?;
        let _ = client.render_frame(&terminal)?;
    }

    Ok(())
}
