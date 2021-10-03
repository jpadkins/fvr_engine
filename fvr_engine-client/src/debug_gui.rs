//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use imgui::Context as ImguiContext;
use imgui_opengl_renderer::Renderer as ImguiOpenglRenderer;
use imgui_sdl2::ImguiSdl2;
use sdl2::event::Event;
use sdl2::mouse::MouseState;
use sdl2::video::Window;
use sdl2::VideoSubsystem;

// DebugGui contains everything related to the ImGui debug gui.
// TODO: Build this out.
pub struct DebugGui {
    imgui: ImguiContext,
    imgui_sdl2: ImguiSdl2,
    imgui_renderer: ImguiOpenglRenderer,
}

impl DebugGui {
    pub fn new(video_subsystem: &VideoSubsystem, window: &Window) -> Self {
        let mut imgui = ImguiContext::create();
        imgui.set_ini_filename(None);

        let imgui_sdl2 = ImguiSdl2::new(&mut imgui, window);
        let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            video_subsystem.gl_get_proc_address(s) as *const _
        });

        Self { imgui, imgui_sdl2, imgui_renderer }
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.imgui_sdl2.handle_event(&mut self.imgui, event);
    }

    pub fn render(&mut self, dt: &Duration, window: &Window, mouse_state: &MouseState) {
        self.imgui_sdl2.prepare_frame(self.imgui.io_mut(), window, mouse_state);
        self.imgui.io_mut().delta_time =
            dt.as_secs() as f32 + dt.subsec_nanos() as f32 / 1_000_000_000.0;

        let ui = self.imgui.frame();
        ui.show_demo_window(&mut true);
        self.imgui_renderer.render(ui);
    }
}
