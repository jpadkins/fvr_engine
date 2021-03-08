use std::time::Instant;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{GLContext, GLProfile, Window, WindowContext};
use sdl2::{EventPump, Sdl, VideoSubsystem};

use crate::debug_gui::*;
use crate::renderer::*;
use crate::terminal::*;

// TODO: Load these from config.
const WINDOW_TITLE: &str = "FVR_ENGINE";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const TERMINAL_WIDTH: u32 = 81;
const TERMINAL_HEIGHT: u32 = 31;

// Provides window management and rendering.
pub struct Client {
    _sdl2_context: Sdl,
    _video_subsystem: VideoSubsystem,
    event_pump: EventPump,
    window: Window,
    _gl_context: GLContext,
    debug_gui: DebugGui,
    renderer: Renderer,
    terminal: Terminal,
    debug_enabled: bool,
    last_frame: Instant,
    resized: bool,
}

impl Client {
    pub fn new() -> Result<Self, String> {
        // SDL
        let sdl2_context = sdl2::init()?;
        let video_subsystem = sdl2_context.video()?;

        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_version(3, 3);

            debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
            debug_assert_eq!(gl_attr.context_version(), (3, 3));
        }

        let event_pump = sdl2_context.event_pump()?;

        let window = video_subsystem
            .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .allow_highdpi()
            .resizable()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        // OpenGL
        let _gl_context = window.gl_create_context()?;
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        // Debug Gui
        let debug_gui = DebugGui::new(&video_subsystem, &window);

        // Renderer
        let renderer = Renderer::new()?;

        // Terminal
        let terminal = Terminal::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);

        Ok(Self {
            _sdl2_context: sdl2_context,
            _video_subsystem: video_subsystem,
            event_pump,
            window,
            _gl_context,
            debug_gui,
            renderer,
            terminal,
            debug_enabled: false,
            last_frame: Instant::now(),
            resized: true,
        })
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        let event_option = self.event_pump.poll_event();

        if let Some(event) = event_option {
            if let Event::Window { .. } = event {
                self.resized = true;
            }

            if self.debug_enabled {
                self.debug_gui.handle_event(&event);
            }

            Some(event)
        } else {
            None
        }
    }

    pub fn toggle_debug(&mut self) {
        self.debug_enabled = !self.debug_enabled;
    }

    pub fn render_frame(&mut self) -> Result<(), String> {
        if self.resized {
            self.renderer.update_viewport(self.window.size())?;
            self.resized = false;
        }

        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        self.renderer.render()?;

        if self.debug_enabled {
            self.debug_gui
                .render(&delta, &self.window, &self.event_pump);
        }

        self.window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));

        Ok(())
    }
}
