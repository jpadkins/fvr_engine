use std::fmt::Display;

use anyhow::{Context, Result};

use std::collections::HashMap;
use std::path::Path;

use fvr_engine_core::prelude::*;

pub struct FontMetricsHandler {
    regular: HashMap<u32, GlyphMetric>,
    outline: HashMap<u32, GlyphMetric>,
}

impl FontMetricsHandler {
    pub fn load_from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let font_metrics_toml = std::fs::read_to_string(path)
            .context("Failed to read contents of font metrics file.")?;

        let font_metrics: FontMetrics = toml::from_str(&font_metrics_toml)
            .context("Failed to parse TOML read from font metrics file.")?;

        let mut regular = HashMap::new();

        for metric in font_metrics.regular {
            regular.insert(metric.codepoint, metric);
        }

        let mut outline = HashMap::new();

        for metric in font_metrics.outline {
            outline.insert(metric.codepoint, metric);
        }

        Ok(Self { regular, outline })
    }

    pub fn regular(&self) -> &HashMap<u32, GlyphMetric> {
        &self.regular
    }

    pub fn outline(&self) -> &HashMap<u32, GlyphMetric> {
        &self.outline
    }
}
