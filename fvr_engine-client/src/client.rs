//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::thread;
use std::time::{Duration, Instant};

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Context, Result};
use sdl2::event::Event;
use sdl2::video::{GLContext, GLProfile, SwapInterval, Window};
use sdl2::{EventPump, Sdl, VideoSubsystem};

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::debug_gui::*;
use crate::input_manager::*;
use crate::renderer_v2::*;
use crate::terminal::*;

//-------------------------------------------------------------------------------------------------
// Client holds the window and rendering context and provides access to the terminal.
//-------------------------------------------------------------------------------------------------
pub struct Client {
    // The SDL2 context (not used after initialization, but it must stay in scope).
    _sdl2_context: Sdl,
    // The SDL2 video context (not used after initialization, but it must stay in scope).
    _video_subsystem: VideoSubsystem,
    // The SDL2 window's event pump for handling user input events.
    event_pump: EventPump,
    // The SDL2 window.
    window: Window,
    // The OpenGL context (not used after initialization, but it must stay in scope).
    _gl_context: GLContext,
    // The debug gui manages the ImGUI debug gui.
    debug_gui: DebugGui,
    // The renderer manages the OpenGL calls for displaying the terminal.
    renderer: RendererV2,
    // Whether to display the debug gui.
    debug_enabled: bool,
    // Time that the last frame began. Used to calculate frame delta time.
    last_frame: Instant,
    // Delta time for the current frame.
    delta_time: Duration,
    // Timer used for limiting the rendering FPS.
    render_timer: Timer,
    // Timer used for calculating the FPS.
    fps_log_timer: Timer,
    // Stores the frame count. Used for calculating the FPS.
    fps_counter: i32,
    // Whether the window has been resized this frame.
    resized: bool,
}

impl Client {
    //---------------------------------------------------------------------------------------------
    // Creates a new client.
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn new() -> Result<Self> {
        // Initialize SDL2.
        //-----------------------------------------------------------------------------------------
        let sdl2_context =
            sdl2::init().map_err(|e| anyhow!(e)).context("Failed to initialize SDL2 context.")?;
        let video_subsystem = sdl2_context
            .video()
            .map_err(|e| anyhow!(e))
            .context("Failed to initialize SDL2 video subsystem.")?;

        // Set the preferred OpenGL context hints.
        //-----------------------------------------------------------------------------------------
        {
            let gl_attr = video_subsystem.gl_attr();
            gl_attr.set_context_profile(GLProfile::Core);
            gl_attr.set_context_version(3, 3);

            debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
            debug_assert_eq!(gl_attr.context_version(), (3, 3));
        }

        // Initialize SDL2 objects.
        //-----------------------------------------------------------------------------------------

        // Create the event pump.
        let event_pump = sdl2_context
            .event_pump()
            .map_err(|e| anyhow!(e))
            .context("Failed to obtain the SDL2 event pump.")?;

        // Build the window depending on the type.
        // Because window builder returns a ref, there's no good way to DRY this up.
        let mut window = match CONFIG.window_type {
            WindowType::Fullscreen => video_subsystem
                .window(
                    CONFIG_WINDOW_TITLE,
                    CONFIG.window_dimensions.0 as u32,
                    CONFIG.window_dimensions.1 as u32,
                )
                .fullscreen()
                .position_centered()
                .resizable()
                .opengl()
                .build()
                .map_err(|e| anyhow!(e))
                .context("Failed to open the SDL2 window."),
            WindowType::Windowed => video_subsystem
                .window(
                    CONFIG_WINDOW_TITLE,
                    CONFIG.window_dimensions.0 as u32,
                    CONFIG.window_dimensions.1 as u32,
                )
                .position_centered()
                .resizable()
                .opengl()
                .build()
                .map_err(|e| anyhow!(e))
                .context("Failed to open the SDL2 window."),
            WindowType::WindowedFullscreen => video_subsystem
                .window(
                    CONFIG_WINDOW_TITLE,
                    CONFIG.window_dimensions.0 as u32,
                    CONFIG.window_dimensions.1 as u32,
                )
                .fullscreen_desktop()
                .position_centered()
                .resizable()
                .opengl()
                .build()
                .map_err(|e| anyhow!(e))
                .context("Failed to open the SDL2 window."),
        }?;

        window.set_minimum_size(
            CONFIG.minimum_window_dimensions.0 as u32,
            CONFIG.minimum_window_dimensions.1 as u32,
        )?;

        // Initialize the OpenGL context.
        //-----------------------------------------------------------------------------------------

        // Query and load the OpenGL context.
        let _gl_context = window
            .gl_create_context()
            .map_err(|e| anyhow!(e))
            .context("Failed to create the OpenGL context.")?;
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as _);

        // Set the OpenGL swap interval to immediate.
        // TODO: Handle vsync.
        video_subsystem
            .gl_set_swap_interval(SwapInterval::Immediate)
            .map_err(|e| anyhow!(e))
            .context("Failed to set OpenGL swap interval.")?;

        // Initialize the debug gui.
        //-----------------------------------------------------------------------------------------
        let debug_gui = DebugGui::new(&video_subsystem, &window);

        // Initialize the renderer.
        //-----------------------------------------------------------------------------------------
        let renderer = RendererV2::new().context("Failed to create the renderer.")?;

        // If the render interval is none, cap at 1000 fps.
        let render_interval = CONFIG.render_interval.unwrap_or_else(|| Duration::from_millis(1));

        // ...and that's it!
        //-----------------------------------------------------------------------------------------
        Ok(Self {
            _sdl2_context: sdl2_context,
            _video_subsystem: video_subsystem,
            event_pump,
            window,
            _gl_context,
            debug_gui,
            renderer,
            debug_enabled: false,
            last_frame: Instant::now(),
            delta_time: Duration::from_secs(0),
            render_timer: Timer::new(render_interval),
            fps_log_timer: Timer::new(CONFIG_FPS_LOG_INTERVAL),
            fps_counter: 0,
            resized: true,
        })
    }

    //---------------------------------------------------------------------------------------------
    // Polls a user input event from the event pump.
    // (or returns none if the event pump is empty)
    //---------------------------------------------------------------------------------------------
    pub fn poll_event(&mut self) -> Option<Event> {
        let event_option = self.event_pump.poll_event();

        // If an event is present, check for resized and also pass to debug gui.
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

    //---------------------------------------------------------------------------------------------
    // Toggles the debug gui.
    //---------------------------------------------------------------------------------------------
    pub fn toggle_debug(&mut self) {
        self.debug_enabled = !self.debug_enabled;
    }

    //---------------------------------------------------------------------------------------------
    // Sets the the current input state and returns the delta time.
    // (should be consumed once per game loop)
    //---------------------------------------------------------------------------------------------
    pub fn update_input(&mut self, input: &mut InputManager) -> Duration {
        // Skip updating input if the debug gui is currently enabled.
        if self.debug_enabled {
            return self.delta_time;
        }

        // Calculate the terminal coord of the mouse if it is within bounds.
        let mouse_state = &self.event_pump.mouse_state();
        let mouse_coord =
            self.renderer.screen_to_terminal_coords((mouse_state.x(), mouse_state.y()));

        // Update input.
        input.update(&self.event_pump.keyboard_state(), mouse_state, mouse_coord);

        // Calculate and return the delta time since input was last updated.
        let now = Instant::now();
        self.delta_time = now - self.last_frame;
        self.last_frame = now;

        self.delta_time
    }

    //---------------------------------------------------------------------------------------------
    // Renders a frame if enough time as passed and returns whether a frame was rendererd.
    // (this should be called in a loop)
    //---------------------------------------------------------------------------------------------
    pub fn render_frame(&mut self, terminal: &Terminal) -> Result<bool> {
        // Print FPS.
        // TODO: Handle this elsewhere?
        //-----------------------------------------------------------------------------------------
        if self.fps_log_timer.update(&self.delta_time) {
            const FPS_LOG_SECONDS: i32 = CONFIG_FPS_LOG_INTERVAL.as_secs() as i32;
            println!("FPS: {}", self.fps_counter / FPS_LOG_SECONDS);

            self.fps_counter = 0;
        }

        // Return early if minimum frame duration has not yet passed.
        //-----------------------------------------------------------------------------------------
        if !self.render_timer.update(&self.delta_time) {
            // Sleep for a bit.
            thread::sleep(CONFIG.sleep_interval);

            return Ok(false);
        }

        // Update frame counter - we are rendering a frame this loop.
        self.fps_counter += 1;

        // Update the renderer viewport if the window has been resized.
        //-----------------------------------------------------------------------------------------
        if self.resized {
            self.renderer
                .update_viewport(Misc::utoi(self.window.size()))
                .context("Failed to refresh renderer scaling.")?;

            // Reset the resized state.
            self.resized = false;
        }

        // Sync the render with the terminal every frame.
        //-----------------------------------------------------------------------------------------
        self.renderer
            .sync_with_terminal(terminal)
            .context("Failed to sync renderer state with terminal.")?;

        // Render a frame.
        //-----------------------------------------------------------------------------------------
        self.renderer.render()?;

        // Optionally render the debug gui as well.
        //-----------------------------------------------------------------------------------------
        if self.debug_enabled {
            self.debug_gui.render(&self.delta_time, &self.window, &self.event_pump.mouse_state());
        }

        // Swap the window buffers and return the delta time.
        //-----------------------------------------------------------------------------------------
        self.window.gl_swap_window();

        Ok(true)
    }
}
