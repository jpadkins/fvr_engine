//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// Path to the config file.
const CONFIG_FILE_PATH: &str = "./config/fvr_engine.json";

// Interval at which to log fps.
pub const CONFIG_FPS_LOG_INTERVAL: Duration = Duration::from_secs(5);

// Title of the game window.
pub const CONFIG_WINDOW_TITLE: &str = "FVR_ENGINE";

// Path to default keybindings. These never change.
pub const CONFIG_DEFAULT_KEYBINDINGS_PATH: &str = "./config/default_keybindings.json";

// Relative path to the fonts directory.
pub const CONFIG_FONTS_DIR: &str = "./assets/fonts/";

// Path to current serialized keybindings. These can change.
pub const CONFIG_KEYBINDINGS_PATH: &str = "./config/keybindings.json";

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_json =
        std::fs::read_to_string(CONFIG_FILE_PATH).expect("Failed to load config file.");
    serde_json::from_str(&config_json).expect("Failed to parse config json.")
});

//-------------------------------------------------------------------------------------------------
// Enumerates the types of game windows.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum WindowType {
    // True fullscreen.
    Fullscreen,
    // Basic windowed.
    Windowed,
    // Windowed stretched to the dimensions of the monitor.
    WindowedFullscreen,
}

//-------------------------------------------------------------------------------------------------
// Config holds the global config.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    // Whether to render the full frame vignette.
    pub enable_vignette: bool,
    // Whether the window should be created fullscreen.
    pub window_type: WindowType,
    // Name of the font to use.
    pub font_name: String,
    // Minimum size of the game window.
    pub minimum_window_dimensions: ICoord,
    // Interval at which to render frames, or none for as fast as possible.
    pub render_interval: Option<Duration>,
    // Interval to sleep when the render interval has not been met.
    pub sleep_interval: Duration,
    // Whether to display current fps.
    pub show_fps: bool,
    // Dimensions (in tiles) of the terminal.
    pub terminal_dimensions: ICoord,
    // Dimensions (in pixels) of each tile.
    pub tile_dimensions: ICoord,
    // Interval to update game state.
    pub update_interval: Duration,
    // Whether to alternate rendering from / uploading data to separate VBOs.
    pub use_alternating_vbos: bool,
    // Whether to use signed distance field font rendering.
    pub use_sdf_fonts: bool,
    // Dimensions (in pixels) of the game window.
    pub window_dimensions: ICoord,
}
