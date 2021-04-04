mod client;
mod debug_gui;
#[macro_use]
mod gl_helpers;
mod input_manager;
mod input_repeat;
mod renderer_v2;
mod shader_strings;
mod terminal;

mod widgets;

pub mod prelude {
    pub use crate::client::*;
    pub use crate::input_manager::*;
    pub use crate::input_repeat::*;
    pub use crate::terminal::*;

    pub use crate::widgets::prelude::*;
}
