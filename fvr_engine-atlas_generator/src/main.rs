use std::fs::{self, File};
use std::io::{prelude::*, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use clap::{App, AppSettings, SubCommand};
use hashbrown::HashSet;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use xml::reader::{EventReader, XmlEvent};

use fvr_engine_core::prelude::*;

// Font used to fill in missing glyphs.
// NOTE: This font must include all possible codepage 437 glyphs.
const DEFAULT_FONT: &str = "deja_vu_sans_mono";

// Directory to save generated atlases.
const OUTPUT_DIR: &str = "./resources/font_atlases";

// Directory of input bmfont files.
const FONTS_DIR: &str = "./fvr_engine-atlas_generator/fonts";

// Dimensions of the output atlas.
// 1024x1024 is enough for most 32px font rendering.
// 1024x2048 for 64px rendering.
const OUTPUT_WIDTH: u32 = 2048;
const OUTPUT_HEIGHT: u32 = 2048;

// Names of the four bmfont files.
const REGULAR_FNT: &str = "regular.fnt";
const REGULAR_PNG: &str = "regular.png";
const OUTLINE_FNT: &str = "outline.fnt";
const OUTLINE_PNG: &str = "outline.png";

fn load_image(file_path: &str) -> Result<DynamicImage> {
    let img = image::open(file_path).context("Failed to open image")?;
    Ok(img)
}

fn parse_metrics(file_path: &str) -> Result<Vec<GlyphMetric>> {
    let mut char_metrics = Vec::new();

    // File IO plumbing.
    let file = File::open(file_path).context("Failed to open fnt file.")?;
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    // Walk the XML.
    for event in parser {
        let element = event.context("Failed to parse an XML event.")?;

        if let XmlEvent::StartElement { name, attributes, .. } = element {
            // We only care about the char elements.
            if name.to_string() != "char" {
                continue;
            }

            // Char attributes follow this order: id, x, y, width, height, xoffset, yoffset.
            let codepoint = attributes[0]
                .value
                .parse::<u32>()
                .context(format!("Failed to parse codepoint: <{}>.", attributes[0]))?;
            let x = attributes[1]
                .value
                .parse::<u32>()
                .context(format!("Failed to parse x: <{}>.", attributes[1]))?;
            let y = attributes[2]
                .value
                .parse::<u32>()
                .context(format!("Failed to parse y: <{}>.", attributes[1]))?;
            let width = attributes[3]
                .value
                .parse::<u32>()
                .context(format!("Failed to parse width: <{}>.", attributes[2]))?;
            let height = attributes[4]
                .value
                .parse::<u32>()
                .context(format!("Failed to parse height: <{}>.", attributes[3]))?;
            let x_offset = attributes[5]
                .value
                .parse::<i32>()
                .context(format!("Failed to parse x_offset: <{}>.", attributes[4]))?;
            let y_offset = attributes[6]
                .value
                .parse::<i32>()
                .context(format!("Failed to parse y_offset: <{}>.", attributes[5]))?;

            char_metrics.push(GlyphMetric { codepoint, x, y, width, height, x_offset, y_offset });
        }
    }

    Ok(char_metrics)
}

fn generate(
    name: &str,
    default_max_regular_height: u32,
    default_max_outline_height: u32,
    default_regular_metrics: &[GlyphMetric],
    default_outline_metrics: &[GlyphMetric],
    default_regular_atlas: &DynamicImage,
    default_outline_atlas: &DynamicImage,
) -> Result<()> {
    const X_PADDING: u32 = 2;
    const Y_PADDING: u32 = 2;

    // Remove files from previous run.
    let output_atlas_path_str = format!("{}/{}.png", OUTPUT_DIR, name);
    let output_atlas_path = Path::new(&output_atlas_path_str);
    if output_atlas_path.exists() {
        fs::remove_file(output_atlas_path)
            .context("Failed to remove output atlas from previous run.")?;
    }

    let output_metrics_path_str = format!("{}/{}.toml", OUTPUT_DIR, name);
    let output_metrics_path = Path::new(&output_metrics_path_str);
    if output_metrics_path.exists() {
        fs::remove_file(output_metrics_path)
            .context("Failed to remove output metrics from previous run")?;
    }

    // Load the metrics and atlases for the font to generate.
    let regular_metrics = parse_metrics(&format!("{}/{}/{}", FONTS_DIR, name, REGULAR_FNT))?;
    let outline_metrics = parse_metrics(&format!("{}/{}/{}", FONTS_DIR, name, OUTLINE_FNT))?;
    let regular_atlas = load_image(&format!("{}/{}/{}", FONTS_DIR, name, REGULAR_PNG))?;
    let outline_atlas = load_image(&format!("{}/{}/{}", FONTS_DIR, name, OUTLINE_PNG))?;

    // Create the output image buffer.
    let mut output_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(OUTPUT_WIDTH, OUTPUT_HEIGHT);

    // Vectors for capturing the new metrics lists to serialize.
    let mut output_metrics = FontMetrics { regular: Vec::new(), outline: Vec::new() };

    // Find the max height of the regular metrics and gather a set of font's codepoints.
    let mut codepoint_set = HashSet::new();
    let mut max_regular_height = 0;

    for metric in regular_metrics.iter() {
        if metric.height > max_regular_height {
            max_regular_height = metric.height;
        }

        // This codepoint will be skipped when processing the default font later.
        codepoint_set.insert(metric.codepoint);
    }

    // Iterate over all regular metrics, copying the glyphs into the output buffer.
    let mut x = 0;
    let mut y = 0;

    for metric in regular_metrics.iter() {
        // Wrap to the next row if necessary.
        if x + metric.width > OUTPUT_WIDTH {
            x = 0;
            y += max_regular_height + Y_PADDING;
        }

        // Copy the glyph.
        let view = regular_atlas.view(metric.x, metric.y, metric.width, metric.height);
        output_buffer.copy_from(&view, x, y).context("Failed to copy regular glyph")?;

        // Push the new metric.
        let output_metric = GlyphMetric {
            codepoint: metric.codepoint,
            x,
            y,
            width: metric.width,
            height: metric.height,
            x_offset: metric.x_offset,
            y_offset: metric.y_offset,
        };
        output_metrics.regular.push(output_metric);

        // Move x position forward.
        x += metric.width + X_PADDING;
    }

    // Find the max height of the outline metrics.
    let mut max_outline_height = 0;

    for metric in outline_metrics.iter() {
        if metric.height > max_outline_height {
            max_outline_height = metric.height;
        }
    }

    // Iterate over all outline metrics, copying the glyphs into the output buffer.
    for metric in outline_metrics.iter() {
        // Wrap to the next row if necessary.
        if x + metric.width > OUTPUT_WIDTH {
            x = 0;
            y += max_outline_height + Y_PADDING;
        }

        // Copy the glyph.
        let view = outline_atlas.view(metric.x, metric.y, metric.width, metric.height);
        output_buffer.copy_from(&view, x, y).context("Failed to copy outline glyph")?;

        // Push the new metric.
        let output_metric = GlyphMetric {
            codepoint: metric.codepoint,
            x,
            y,
            width: metric.width,
            height: metric.height,
            x_offset: metric.x_offset,
            y_offset: metric.y_offset,
        };
        output_metrics.outline.push(output_metric);

        // Move x position forward.
        x += metric.width + X_PADDING;
    }

    // Ensure that all of codepage 437 is covered by iterating and potentially copying the default
    // glyphs and metrics for both regular and outline chars.

    // Default regular.
    for metric in default_regular_metrics.iter() {
        // Skip chars that where included in the main font.
        if codepoint_set.contains(&metric.codepoint) {
            continue;
        }

        // Wrap to the next row if necessary.
        if x + metric.width > OUTPUT_WIDTH {
            x = 0;
            y += default_max_regular_height + Y_PADDING;
        }

        // Copy the glyph.
        let view = default_regular_atlas.view(metric.x, metric.y, metric.width, metric.height);
        output_buffer.copy_from(&view, x, y).context("Failed to copy default regular glyph")?;

        // Push the new metric.
        let output_metric = GlyphMetric {
            codepoint: metric.codepoint,
            x,
            y,
            width: metric.width,
            height: metric.height,
            x_offset: metric.x_offset,
            y_offset: metric.y_offset,
        };
        output_metrics.regular.push(output_metric);

        // Move x position forward.
        x += metric.width + X_PADDING;
    }

    // Default outline.
    for metric in default_outline_metrics.iter() {
        // Skip chars that where included in the main font.
        if codepoint_set.contains(&metric.codepoint) {
            continue;
        }

        // Wrap to the next row if necessary.
        if x + metric.width > OUTPUT_WIDTH {
            x = 0;
            y += default_max_outline_height + Y_PADDING;
        }

        // Copy the glyph.
        let view = default_outline_atlas.view(metric.x, metric.y, metric.width, metric.height);
        output_buffer.copy_from(&view, x, y).context("Failed to copy default outline glyph")?;

        // Push the new metric.
        let output_metric = GlyphMetric {
            codepoint: metric.codepoint,
            x,
            y,
            width: metric.width,
            height: metric.height,
            x_offset: metric.x_offset,
            y_offset: metric.y_offset,
        };
        output_metrics.outline.push(output_metric);

        // Move x position forward.
        x += metric.width + X_PADDING;
    }

    // Save the output atlas.
    output_buffer.save(output_atlas_path).context("Failed to save output atlas.")?;

    // Save the output metrics.
    let toml = toml::to_string(&output_metrics).context("Failed to serialize output metrics.")?;
    let mut output_metrics_file =
        File::create(output_metrics_path).context("Failed to create output metrics file.")?;
    output_metrics_file.write_all(toml.as_bytes()).context("Failed to save output metrics.")?;

    Ok(())
}

fn generate_all() -> Result<()> {
    // Load the default metrics and atlases.
    let default_regular_metrics =
        parse_metrics(&format!("{}/{}/{}", FONTS_DIR, DEFAULT_FONT, REGULAR_FNT))?;
    let default_outline_metrics =
        parse_metrics(&format!("{}/{}/{}", FONTS_DIR, DEFAULT_FONT, OUTLINE_FNT))?;
    let default_regular_atlas =
        load_image(&format!("{}/{}/{}", FONTS_DIR, DEFAULT_FONT, REGULAR_PNG))?;
    let default_outline_atlas =
        load_image(&format!("{}/{}/{}", FONTS_DIR, DEFAULT_FONT, OUTLINE_PNG))?;

    // Find the max height of the default regular and outline metrics.
    let mut default_max_regular_height = 0;
    for metric in default_regular_metrics.iter() {
        if metric.height > default_max_regular_height {
            default_max_regular_height = metric.height;
        }
    }

    let mut default_max_outline_height = 0;
    for metric in default_outline_metrics.iter() {
        if metric.height > default_max_outline_height {
            default_max_outline_height = metric.height;
        }
    }

    // Get a list of atlases to generate.
    let entries = fs::read_dir(FONTS_DIR).context("Failed to read fonts directory.")?;

    // Generate atlases and metrics for all fonts.
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .file_name()
            .context("Failed to retrieve directory name.")?
            .to_str()
            .context("Failed to convert from OsStr.")?;

        generate(
            name,
            default_max_regular_height,
            default_max_outline_height,
            &default_regular_metrics,
            &default_outline_metrics,
            &default_regular_atlas,
            &default_outline_atlas,
        )?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("FVR_ENGINE-ATLAS_GENERATOR")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("0.0.1")
        .author("Jacob Adkins (jpadkins@pm.me) 2020-2021")
        .about("CLI tool for generating atlas textures from TTF fonts for glyphs on codepage 437.")
        .subcommand(SubCommand::with_name("run").about("Generate all atlases"))
        .subcommand(SubCommand::with_name("list").about("List atlases to be generated"))
        .get_matches();

    if matches.subcommand_matches("run").is_some() {
        generate_all()?;
    } else if matches.subcommand_matches("list").is_some() {
        println!("Listing!");
    }

    Ok(())
}
