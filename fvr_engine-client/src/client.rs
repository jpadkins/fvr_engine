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
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::debug_gui::*;
use crate::input_manager::*;
use crate::renderer_v2::*;
use crate::terminal::*;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// Render at 60 fps.
const FRAME_DURATION: Duration = Duration::from_millis(1000 / 60);

// Duration to sleep when frame duration has not yet passed.
const SLEEP_DURATION: Duration = Duration::from_millis(1);

// Interval at which to print the FPS.
const FPS_LOG_DURATION: Duration = Duration::from_secs(5);

//-------------------------------------------------------------------------------------------------
// Client holds the window and rendering context and provides access to the terminal.
//-------------------------------------------------------------------------------------------------
pub struct Client {
    // Dimensions of the faux terminal.
    terminal_dimensions: (u32, u32),
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
    // Timer used for limiting the rendering FPS.
    frame_timer: Timer,
    // Timer used for calculating the FPS.
    fps_log_timer: Timer,
    // Stores the frame count. Used for calculating the FPS.
    fps_counter: u32,
    // Whether the window has been resized this frame.
    resized: bool,
}

impl Client {
    //---------------------------------------------------------------------------------------------
    // Creates a new client
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn new<S>(
        window_title: S,
        window_dimensions: (u32, u32),
        terminal_dimensions: (u32, u32),
        tile_dimensions: (u32, u32),
        font_name: S,
    ) -> Result<Self>
    where
        S: AsRef<str>,
    {
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

        // Build the window.
        let window = video_subsystem
            .window(window_title.as_ref(), window_dimensions.0, window_dimensions.1)
            .position_centered()
            .allow_highdpi()
            .resizable()
            .opengl()
            .build()
            .map_err(|e| anyhow!(e))
            .context("Failed to open the SDL2 window.")?;

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
        let renderer = RendererV2::new(tile_dimensions, terminal_dimensions, font_name)
            .context("Failed to create the renderer.")?;

        // ...and that's it!
        //-----------------------------------------------------------------------------------------
        Ok(Self {
            terminal_dimensions,
            _sdl2_context: sdl2_context,
            _video_subsystem: video_subsystem,
            event_pump,
            window,
            _gl_context,
            debug_gui,
            renderer,
            // Disable the debug gui by default.
            debug_enabled: false,
            // Last set last frame to current moment.
            last_frame: Instant::now(),
            // Set frame timer to an empty duration.
            frame_timer: Timer::new(FRAME_DURATION),
            // Set fps timer to an empty duration.
            fps_log_timer: Timer::new(FPS_LOG_DURATION),
            // Start the fps counter at 0.
            fps_counter: 0,
            // Set resized to true to update renderer on first frame.
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
    // Returns a new terminal matching the client's dimensions.
    //---------------------------------------------------------------------------------------------
    pub fn create_terminal(&self) -> Terminal {
        Terminal::new(self.terminal_dimensions.0, self.terminal_dimensions.1)
    }

    //---------------------------------------------------------------------------------------------
    // Returns the current key state.
    // (should be consumed once per game loop)
    //---------------------------------------------------------------------------------------------
    pub fn update_input(&self, input: &mut InputManager) {
        input.update(&self.event_pump.keyboard_state());
    }

    //---------------------------------------------------------------------------------------------
    // Renders a frame.
    // (this should be called in a loop)
    //---------------------------------------------------------------------------------------------
    pub fn render_frame(&mut self, terminal: &Terminal) -> Result<Duration> {
        // Calculate delta time.
        //-----------------------------------------------------------------------------------------
        let now = Instant::now();
        let delta = now - self.last_frame;
        self.last_frame = now;

        // Print FPS.
        // TODO: Handle this elsewhere?
        //-----------------------------------------------------------------------------------------
        self.fps_counter += 1;

        if self.fps_log_timer.update(delta) {
            const FPS_LOG_SECONDS: u32 = FPS_LOG_DURATION.as_secs() as u32;
            println!("FPS: {}", self.fps_counter / FPS_LOG_SECONDS);

            self.fps_counter = 0;
        }

        // Return early if minimum frame duration has not yet passed.
        //-----------------------------------------------------------------------------------------
        if !self.frame_timer.update(delta) {
            // Sleep for a bit.
            thread::sleep(SLEEP_DURATION);

            // Calculate the new delta time.
            let now = Instant::now();
            let delta = delta + (now - self.last_frame);
            self.last_frame = now;

            // Update the timers.
            self.frame_timer.update_without_consuming(delta);
            self.fps_log_timer.update_without_consuming(delta);

            // Return new passed time.
            return Ok(delta);
        }

        // Update the renderer viewport if the window has been resized.
        //-----------------------------------------------------------------------------------------
        if self.resized {
            self.renderer
                .update_viewport(self.window.size())
                .context("Failed to refresh renderer scaling.")?;

            // Reset the resized state.
            self.resized = false;
        }

        // Sync the render with the terminal every frame.
        //-----------------------------------------------------------------------------------------
        self.renderer
            .sync_with_terminal(terminal)
            .context("Failed to sync renderer state with terminal.")?;

        // Actually render a frame.
        //-----------------------------------------------------------------------------------------
        self.renderer.render()?;

        // Optionally render the debug gui as well.
        //-----------------------------------------------------------------------------------------
        if self.debug_enabled {
            self.debug_gui.render(&delta, &self.window, &self.event_pump.mouse_state());
        }

        // Swap the window buffers and return the delta time.
        //-----------------------------------------------------------------------------------------
        self.window.gl_swap_window();

        Ok(delta)
    }
}
