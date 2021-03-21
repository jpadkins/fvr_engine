mod client;
mod debug_gui;
#[macro_use]
mod gl_helpers;
mod renderer_v2;
mod shader_strings;
mod terminal;

pub mod prelude {
    pub use crate::client::*;
}
