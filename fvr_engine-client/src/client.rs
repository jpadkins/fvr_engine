use std::collections::HashSet;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::{GLContext, GLProfile, SwapInterval, Window};
use sdl2::{EventPump, Sdl, VideoSubsystem};

use crate::debug_gui::*;
use crate::renderer::*;
use crate::renderer_v2::*;
use crate::terminal::*;

// TODO: Load these from config.
const WINDOW_TITLE: &str = "FVR_ENGINE";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const TERMINAL_WIDTH: u32 = 81; // 103
const TERMINAL_HEIGHT: u32 = 31; // 37
const TILE_WIDTH: u32 = 48;
const TILE_HEIGHT: u32 = 64;

// Render at 60 fps.
const FRAME_DURATION: Duration = Duration::from_millis(1000 / 60);

// Provides window management and rendering.
pub struct Client {
    _sdl2_context: Sdl,
    _video_subsystem: VideoSubsystem,
    event_pump: EventPump,
    window: Window,
    _gl_context: GLContext,
    debug_gui: DebugGui,
    renderer: RendererV2,
    terminal: Terminal,
    debug_enabled: bool,
    last_frame: Instant,
    frame_timer: Duration,
    fps_timer: Duration,
    fps_counter: u32,
    resized: bool,
}

impl Client {
    pub fn new() -> Result<Self> {
        // SDL
        let sdl2_context =
            sdl2::init().map_err(|e| anyhow!(e)).context("Failed to initialize SDL2 context.")?;
        let video_subsystem = sdl2_context
            .video()
            .map_err(|e| anyhow!(e))
            .context("Failed to initialize SDL2 video subsystem.")?;

        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_version(3, 3);

            debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
            debug_assert_eq!(gl_attr.context_version(), (3, 3));
        }

        let event_pump = sdl2_context
            .event_pump()
            .map_err(|e| anyhow!(e))
            .context("Failed to obtain the SDL2 event pump.")?;

        let window = video_subsystem
            .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .allow_highdpi()
            .resizable()
            .opengl()
            .build()
            .map_err(|e| anyhow!(e))
            .context("Failed to open the SDL2 window.")?;

        // OpenGL
        let _gl_context = window
            .gl_create_context()
            .map_err(|e| anyhow!(e))
            .context("Failed to create the OpenGL context.")?;
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        // TODO: Handle vsync.
        video_subsystem
            .gl_set_swap_interval(SwapInterval::Immediate)
            .map_err(|e| anyhow!(e))
            .context("Failed to set OpenGL swap interval.")?;

        // Debug Gui
        let debug_gui = DebugGui::new(&video_subsystem, &window);

        // Renderer
        let mut renderer = RendererV2::new(
            (TILE_WIDTH, TILE_HEIGHT),
            (TERMINAL_WIDTH, TERMINAL_HEIGHT),
            "./resources/font_atlases/deja_vu_sans_mono.png",
            "./resources/font_atlases/deja_vu_sans_mono.toml",
        ).context("Failed to create the renderer.")?;

        // Terminal
        let terminal = Terminal::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);

        renderer
            .sync_with_terminal(&terminal)
            .context("Failed to sync renderer state with terminal.")?;

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
            frame_timer: Default::default(),
            fps_timer: Default::default(),
            fps_counter: 0,
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

    pub fn randomize_terminal(&mut self) {
        self.terminal.randomize();
    }

    pub fn key_state(&self) -> HashSet<Keycode> {
        self.event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect()
    }

    pub fn render_frame(&mut self) -> Result<u32> {
        // Get frame time.
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        // TODO:
        self.frame_timer += delta;
        self.fps_timer += delta;
        self.fps_counter += 1;

        const FPS_LOG_DURATION: Duration = Duration::from_millis(5000);

        if self.fps_timer > FPS_LOG_DURATION {
            println!("FPS: {}", self.fps_counter / 5);

            self.fps_counter = 0;
            self.fps_timer -= FPS_LOG_DURATION;
        }

        if self.frame_timer < FRAME_DURATION {
            return Ok(delta.as_millis() as u32);
        }
        self.frame_timer -= FRAME_DURATION;

        // Recenter the renderer if the window has been resized.
        if self.resized {
            self.renderer
                .update_viewport(self.window.size())
                .context("Failed to refresh renderer scaling.")?;
            self.resized = false;
        }

        // Sync the renderer with the terminal if changes have been made.
        // if self.terminal.dirty() {
            self.renderer
                .sync_with_terminal(&self.terminal)
                .context("Failed to sync renderer state with terminal.")?;
            // self.terminal.set_clean();
        // }

        // Render a frame.
        self.renderer.render()?;

        // Optionally render the debug gui.
        if self.debug_enabled {
            self.debug_gui.render(&delta, &self.window, &self.event_pump.mouse_state());
        }

        // Swap the backbuffer.
        self.window.gl_swap_window();

        Ok(delta.as_millis() as u32)
    }
}
