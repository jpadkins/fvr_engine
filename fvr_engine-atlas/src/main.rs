use std::fs::{self, File};
use std::io::{prelude::*, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::{App, AppSettings, SubCommand};
use hashbrown::HashSet;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgba};
use rect_packer::Packer;
use xml::reader::{EventReader, XmlEvent};

use fvr_engine_core::prelude::*;

// Font used to fill in missing glyphs.
// NOTE: This font must include all possible codepage 437 glyphs.
const DEFAULT_FONT: &str = "deja_vu_sans_mono";

// Directory to save generated atlases.
const OUTPUT_DIR: &str = "./resources/font_atlases";

// Directory of input bmfont files.
const FONTS_DIR: &str = "./fvr_engine-atlas/fonts";

// Glyphs that are always copied from the default font.
// const ALWAYS_DEFAULT_GLYPHS: &[i32] = &[
// '♥' as i32,
// '•' as i32,
// '◘' as i32,
// '○' as i32,
// '◙' as i32,
// ];

// Dimensions of the output atlas.
// 1024x1024 is enough for most 32px font rendering.
// 1024x2048 for 64px rendering.
const OUTPUT_WIDTH: i32 = 1024;
const OUTPUT_HEIGHT: i32 = 1024;

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
                .parse::<i32>()
                .context(format!("Failed to parse codepoint: <{}>.", attributes[0]))?;
            let x = attributes[1]
                .value
                .parse::<i32>()
                .context(format!("Failed to parse x: <{}>.", attributes[1]))?;
            let y = attributes[2]
                .value
                .parse::<i32>()
                .context(format!("Failed to parse y: <{}>.", attributes[1]))?;
            let width = attributes[3]
                .value
                .parse::<i32>()
                .context(format!("Failed to parse width: <{}>.", attributes[2]))?;
            let height = attributes[4]
                .value
                .parse::<i32>()
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

fn generate(name: &str, font_name: &str) -> Result<()> {
    // Load default metric and atlas.
    let default_metrics =
        parse_metrics(&format!("{}/{}/{}.fnt", FONTS_DIR, DEFAULT_FONT, font_name))?;
    let default_atlas =
        load_image(&format!("{}/{}/{}_0.png", FONTS_DIR, DEFAULT_FONT, font_name))?;

    // Load font metric and atlas.
    let metrics = parse_metrics(&format!("{}/{}/{}.fnt", FONTS_DIR, name, font_name))?;
    let atlas = load_image(&format!("{}/{}/{}_0.png", FONTS_DIR, name, font_name))?;

    // Create the output image buffer.
    let mut output_buffer =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::new(OUTPUT_WIDTH as u32, OUTPUT_HEIGHT as u32);

    // Vectors for capturing the new metrics lists to serialize.
    let mut output_metrics = FontMetricsV2 { metrics: Vec::new() };

    // Gather a set of the font's codepoints.
    let mut codepoint_set = HashSet::new();

    // This codepoint will be skipped when processing the default font later.
    for metric in metrics.iter() {
        codepoint_set.insert(metric.codepoint);
    }

    // Initialize the rect packer.
    let config = rect_packer::Config {
        width: OUTPUT_WIDTH as i32,
        height: OUTPUT_HEIGHT as i32,
        border_padding: 2,
        rectangle_padding: 2,
    };
    let mut packer = Packer::new(config);

    // Iterate over all regular metrics, copying the glyphs into the output buffer.
    for metric in metrics.iter() {
        // Copy the glyph.
        let view = atlas.view(
            metric.x as u32,
            metric.y as u32,
            metric.width as u32,
            metric.height as u32,
        );
        let rect = packer
            .pack(metric.width as i32, metric.height as i32, false)
            .ok_or(anyhow!("Failed to pack rect."))?;

        output_buffer
            .copy_from(&view, rect.x as u32, rect.y as u32)
            .context("Failed to copy glyph")?;

        // Push the new metric.
        let output_metric = GlyphMetric {
            codepoint: metric.codepoint,
            x: rect.x as i32,
            y: rect.y as i32,
            width: metric.width,
            height: metric.height,
            x_offset: metric.x_offset,
            y_offset: metric.y_offset,
        };
        output_metrics.metrics.push(output_metric);
    }

    // Ensure all glyphs are covered by iterating default font.
    for metric in default_metrics.iter() {
        // Skip chars that where included in the main font.
        if codepoint_set.contains(&metric.codepoint) {
            continue;
        }

        // Copy the glyph.
        let view = default_atlas.view(
            metric.x as u32,
            metric.y as u32,
            metric.width as u32,
            metric.height as u32,
        );
        let rect = packer
            .pack(metric.width as i32, metric.height as i32, false)
            .ok_or(anyhow!("Failed to pack rect."))?;

        output_buffer
            .copy_from(&view, rect.x as u32, rect.y as u32)
            .context("Failed to copy default glyph")?;

        // Push the new metric.
        let output_metric = GlyphMetric {
            codepoint: metric.codepoint,
            x: rect.x as i32,
            y: rect.y as i32,
            width: metric.width,
            height: metric.height,
            x_offset: metric.x_offset,
            y_offset: metric.y_offset,
        };
        output_metrics.metrics.push(output_metric);
    }

    // Save the atlas and metrics.
    let output_atlas_path = format!("{}/{}/{}.png", OUTPUT_DIR, name, font_name);

    output_buffer.save(output_atlas_path).context("Failed to save output atlas.")?;

    let output_metrics_path = format!("{}/{}/{}.toml", OUTPUT_DIR, name, font_name);
    let toml = toml::to_string(&output_metrics).context("Failed to serialize output metrics.")?;
    let mut output_metrics_file =
        File::create(output_metrics_path).context("Failed to create output metrics file.")?;
    output_metrics_file.write_all(toml.as_bytes()).context("Failed to save output metrics.")?;

    Ok(())
}

fn generate_all() -> Result<()> {
    // Names of the bmfonts.
    const FONT_NAMES: &[&str] = &[
        "regular",
        "regular_outline",
        "bold",
        "bold_outline",
        "italic",
        "italic_outline",
        "bold_italic",
        "bold_italic_outline",
    ];

    let entries = fs::read_dir(FONTS_DIR).context("Failed to read fonts directory.")?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .file_name()
            .context("Failed to read directory name.")?
            .to_str()
            .context("Failed to convert from OsStr.")?;

        // Ensure the output dir exists and is empty.
        let output_dir = format!("{}/{}", OUTPUT_DIR, name);
        let output_dir = Path::new(&output_dir);
        if !output_dir.exists() {
            fs::create_dir(output_dir).context("Failed to create directory")?;
        }

        for entry in fs::read_dir(output_dir)? {
            fs::remove_file(entry?.path()).context("Failed to remove directory entries")?;
        }

        // Generate the fonts.
        for font_name in FONT_NAMES.iter() {
            generate(name, font_name).context("Failed to generate font.")?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("FVR_ENGINE-ATLAS")
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
