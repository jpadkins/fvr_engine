use anyhow::Result;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

fn main() -> Result<(), String> {
    let sdl2_context = sdl2::init()?;
    let video_subsystem = sdl2_context.video()?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG);
    let mut event_pump = sdl2_context.event_pump().map_err(|e| e.to_string())?;
    let window = video_subsystem
        .window("FVR_ENGINE", 800, 600)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let _texture_creator = canvas.texture_creator();
    canvas.set_draw_color(Color::RGB(30, 15, 60));

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();
    }

    Ok(())
}
