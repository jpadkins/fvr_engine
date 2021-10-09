//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use serde_derive::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------
// Describes the location of a glyph within a font atlas, as well as positioning info.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GlyphMetric {
    pub codepoint: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

//-------------------------------------------------------------------------------------------------
// Array of glyph metrics for a font.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FontMetricsV2 {
    pub metrics: Vec<GlyphMetric>,
}
