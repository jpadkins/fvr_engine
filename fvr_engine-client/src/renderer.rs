// TODO: (At some point if it becomes an issue)
// Rendering can be optimized further -
// 1. Create two vertex buffers with dynamic draw.
// 2. Populate one buffer every frame based on the terminal changes.
// 3. Render the other buffer (i.e. double buffered vertices).
// Background vertices can be updated so that there is only one vertex format.
//
// UPDATE: See RendererV2.
use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::str;

use anyhow::{anyhow, Context, Result};
use gl::types::*;
use glam::{Mat4, Vec3};
use image::DynamicImage;

use fvr_engine_core::prelude::*;

use crate::font_metrics_handler::*;
use crate::gl_helpers::*;
use crate::quad_grid::*;
use crate::shader_strings::*;
use crate::sparse_quad_grid::*;
use crate::terminal::*;

// TODO: Move these to config file and pass them into the fvr_engine-client ctor.
const FONT_ATLAS_PATH: &str = "./resources/font_atlases";
const FONT_NAME: &str = "deja_vu_sans_mono";

// Define vertex structure for background quad grid.
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug)]
struct BackgroundVertex {
    position: [GLfloat; 2],
    color: [GLfloat; 3],
}

impl Vertex for BackgroundVertex {
    fn size_of() -> usize {
        mem::size_of::<GLfloat>() * 5
    }

    fn enable_attribs(program: GLuint) -> Result<()> {
        unsafe {
            let location = get_attrib_location(program, "position")
                .context("Failed to get background position attrib location.")?;

            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(location as GLuint);
            gl_error_unwrap!();

            let location = get_attrib_location(program, "color")
                .context("Failed to get background color attrib location.")?;

            gl::VertexAttribPointer(
                location as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                (mem::size_of::<GLfloat>() * 2) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);
            gl_error_unwrap!();
        }

        Ok(())
    }
}

// Define vertex structure for foreground quad grid.
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug)]
struct ForegroundVertex {
    position: [GLfloat; 2],
    color: [GLfloat; 4],
    tex_coords: [GLfloat; 2],
}

impl Vertex for ForegroundVertex {
    fn size_of() -> usize {
        mem::size_of::<GLfloat>() * 8
    }

    fn enable_attribs(program: GLuint) -> Result<()> {
        unsafe {
            let location = get_attrib_location(program, "position")
                .context("Failed to get foreground position attrib location.")?;

            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(location as GLuint);
            gl_error_unwrap!();

            let location = get_attrib_location(program, "color")
                .context("Failed to get foreground color attrib location.")?;

            gl::VertexAttribPointer(
                location as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                (mem::size_of::<GLfloat>() * 2) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);
            gl_error_unwrap!();

            let location = get_attrib_location(program, "tex_coords")
                .context("Failed to get foreground tex_coords attrib location.")?;

            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                (mem::size_of::<GLfloat>() * 6) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);
            gl_error_unwrap!();
        }

        Ok(())
    }
}

// Impl trait for vertex structs.
impl QuadGridVertex for BackgroundVertex {}
impl QuadGridVertex for ForegroundVertex {}

// Renderer contains the OpenGL state pointers and everything required for rendering.
pub struct Renderer {
    terminal_width: u32,
    terminal_height: u32,
    tile_width: u32,
    tile_height: u32,
    background_program: GLuint,
    foreground_program: GLuint,
    background_vao: GLuint,
    foreground_vao: GLuint,
    outline_vao: GLuint,
    background_quad_grid: QuadGrid<BackgroundVertex>,
    foreground_quad_grid: QuadGrid<ForegroundVertex>,
    outline_quad_grid: SparseQuadGrid<ForegroundVertex>,
    background_mvp_location: GLint,
    foreground_mvp_location: GLint,
    atlas_texture: GLuint,
    atlas_width: u32,
    atlas_height: u32,
    font_metrics: FontMetricsHandler,
}

impl Renderer {
    pub fn new(
        terminal_width: u32,
        terminal_height: u32,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<Self> {
        // Setup background VAO.
        let mut background_vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut background_vao);
        }
        gl_error_unwrap!();

        // Link background shader program.
        let vertex_shader = compile_shader(BACKGROUND_VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER)
            .context("Failed to compile background vertex shader.")?;
        let fragment_shader =
            compile_shader(BACKGROUND_FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER)
                .context("Failed to compile background fragment shader.")?;
        let background_program = link_program(vertex_shader, fragment_shader)
            .context("Failed to link background program.")?;
        let background_mvp_location = get_uniform_location(background_program, "mvp")
            .context("Failed to obtain background MVP matrix uniform location.")?;

        // The shaders are no longer needed.
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        // Bind background VAO.
        unsafe {
            gl::BindVertexArray(background_vao);
        }
        gl_error_unwrap!();

        // Bind buffer data and enable attribs.
        let mut background_quad_grid =
            QuadGrid::<BackgroundVertex>::new(terminal_width, terminal_height)
                .context("Failed to create background quad grid.")?;

        // Position data of the background quad grid will not change.
        for x in 0..background_quad_grid.width() {
            for y in 0..background_quad_grid.height() {
                let mut quad = background_quad_grid.quad_mut(x, y);

                quad[0].position[0] = (x * tile_width) as GLfloat;
                quad[0].position[1] = (y * tile_height) as GLfloat;

                quad[1].position[0] = ((x * tile_width) + tile_width) as GLfloat;
                quad[1].position[1] = (y * tile_height) as GLfloat;

                quad[2].position[0] = ((x * tile_width) + tile_width) as GLfloat;
                quad[2].position[1] = ((y * tile_height) + tile_height) as GLfloat;

                quad[3].position[0] = (x * tile_width) as GLfloat;
                quad[3].position[1] = ((y * tile_height) + tile_height) as GLfloat;
            }
        }

        background_quad_grid.bind_data().context("Failed to bind background quad grid data.")?;

        BackgroundVertex::enable_attribs(background_program)
            .context("Failed to enable background quad grid vertex attribs.")?;

        // Setup foreground VAO.
        let mut foreground_vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut foreground_vao);
        }
        gl_error_unwrap!();

        // Link background shader program.
        let vertex_shader = compile_shader(FOREGROUND_VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER)
            .context("Failed to compile foreground vertex shader.")?;
        let fragment_shader =
            compile_shader(FOREGROUND_FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER)
                .context("Failed to compile foreground fragment shader.")?;
        let foreground_program = link_program(vertex_shader, fragment_shader)
            .context("Failed to link foreground program.")?;
        let foreground_mvp_location = get_uniform_location(foreground_program, "mvp")
            .context("Failed to obtain foreground MVP matrix uniform location.")?;

        // The shaders are no longer needed.
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        // Bind foreground VAO.
        unsafe {
            gl::BindVertexArray(foreground_vao);
        }
        gl_error_unwrap!();

        // Bind buffer data and enable attribs.
        let foreground_quad_grid =
            QuadGrid::<ForegroundVertex>::new(terminal_width, terminal_height)
                .context("Failed to create foreground quad grid.")?;

        foreground_quad_grid.bind_data().context("Failed to bind foreground quad grid data.")?;

        ForegroundVertex::enable_attribs(foreground_program)
            .context("Failed to enable foreground quad grid vertex attribs.")?;

        // Setup and bind outline VAO.
        // The outline VAO will reuse the foreground shader program and vertex attribs.
        let mut outline_vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut outline_vao);
            gl_error_unwrap!();

            gl::BindVertexArray(outline_vao);
            gl_error_unwrap!();
        }

        // Bind buffer data and enable attribs.
        let mut outline_quad_grid =
            SparseQuadGrid::<ForegroundVertex>::new(terminal_width, terminal_height)
                .context("Failed to create outline quad grid.")?;

        outline_quad_grid.bind_data().context("Failed to bind outline quad grid data.")?;

        ForegroundVertex::enable_attribs(foreground_program)
            .context("Failed to enable outline (foreground) quad grid vertex attribs.")?;

        // Load font atlas texture.
        let atlas_path = format!("{}/{}.png", FONT_ATLAS_PATH, FONT_NAME);
        let atlas_image = image::open(&atlas_path)
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to open font atlas image at {}.", atlas_path))?;
        let atlas_image = match atlas_image {
            DynamicImage::ImageRgba8(image) => image,
            other_format => other_format.to_rgba8(),
        };

        let (atlas_width, atlas_height) = atlas_image.dimensions();

        let mut atlas_texture = 0;

        unsafe {
            gl::GenTextures(1, &mut atlas_texture);
            gl_error_unwrap!();

            gl::BindTexture(gl::TEXTURE_2D, atlas_texture);
            gl_error_unwrap!();

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl_error_unwrap!();
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl_error_unwrap!();
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl_error_unwrap!();
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl_error_unwrap!();

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                atlas_width as GLsizei,
                atlas_height as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                atlas_image.as_ptr() as *const c_void,
            );
            gl_error_unwrap!();

            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl_error_unwrap!();
        }

        // Misc. OpenGL settings.
        unsafe {
            // Optimized blending for opaque background.
            // https://apoorvaj.io/alpha-compositing-opengl-blending-and-premultiplied-alpha/
            gl::Enable(gl::BLEND);
            gl_error_unwrap!();

            gl::BlendEquation(gl::FUNC_ADD);
            gl_error_unwrap!();

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl_error_unwrap!();

            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl_error_unwrap!();
        }

        // Load the font metrics.
        let metrics_path = format!("{}/{}.toml", FONT_ATLAS_PATH, FONT_NAME);
        let font_metrics = FontMetricsHandler::load_from_file(metrics_path)
            .context("Failed to load font metrics handler.")?;

        Ok(Self {
            terminal_width,
            terminal_height,
            tile_width,
            tile_height,
            background_program,
            foreground_program,
            background_vao,
            foreground_vao,
            outline_vao,
            background_quad_grid,
            foreground_quad_grid,
            outline_quad_grid,
            background_mvp_location,
            foreground_mvp_location,
            atlas_texture,
            atlas_width,
            atlas_height,
            font_metrics,
        })
    }

    pub fn update_viewport(&self, (width, height): (u32, u32)) -> Result<()> {
        // Find the dimensions (in pixels) of the quad grid.
        let effective_width = (self.terminal_width * self.tile_width) as f32;
        let effective_height = (self.terminal_height * self.tile_height) as f32;

        // Find the ratios of actual width/height to quad grid width/height.
        let x_ratio = width as f32 / effective_width;
        let y_ratio = height as f32 / effective_height;

        let x_translate;
        let y_translate;
        let scale;

        // Depending on which ratio is larger, set the translation and scale to center the quad grid.
        if x_ratio > y_ratio {
            x_translate = ((width as f32 - (effective_width * y_ratio)) / 2.0).floor();
            y_translate = 0.0;
            scale = y_ratio;
        } else {
            x_translate = 0.0;
            y_translate = ((height as f32 - (effective_height * x_ratio)) / 2.0).floor();
            scale = x_ratio;
        }

        // Calculate an orthographic projection matrix with our translation and scale.
        let projection = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, 0.0, 1.0);
        let translate = Mat4::from_translation(Vec3::new(x_translate, y_translate, 0.0));
        let scale = Mat4::from_scale(Vec3::new(scale, scale, 1.0));

        let mvp_data = (projection * translate * scale).to_cols_array();

        // Update the uniforms.
        unsafe {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);

            gl::UseProgram(self.background_program);
            gl_error_unwrap!();

            gl::UniformMatrix4fv(
                self.background_mvp_location,
                1,
                gl::FALSE as GLboolean,
                &mvp_data as *const f32,
            );
            gl_error_unwrap!();

            gl::UseProgram(self.foreground_program);
            gl_error_unwrap!();

            gl::UniformMatrix4fv(
                self.foreground_mvp_location,
                1,
                gl::FALSE as GLboolean,
                &mvp_data as *const f32,
            );
            gl_error_unwrap!();
        }

        Ok(())
    }

    pub fn update_from_terminal(&mut self, terminal: &Terminal) -> Result<()> {
        let mut x_positions = [0.0; 4];
        let mut y_positions = [0.0; 4];
        let mut u_tex_coords = [0.0; 4];
        let mut v_tex_coords = [0.0; 4];

        let u_normalize = 1.0 / self.atlas_width as f32;
        let v_normalize = 1.0 / self.atlas_height as f32;

        // Iterate over dirty tiles in terminal.
        for ((x, y), tile) in terminal.dirty_tiles_iter() {
            // Update the background colors.
            for vertex in self.background_quad_grid.quad_mut(x, y) {
                vertex.color[0] = tile.background_color.0.r as f32 / 255.0;
                vertex.color[1] = tile.background_color.0.g as f32 / 255.0;
                vertex.color[2] = tile.background_color.0.b as f32 / 255.0;
            }

            // Update the foreground positions, colors, and tex coords.
            self.assign_glyph_positions(
                x,
                y,
                tile.glyph,
                tile.layout,
                false,
                &mut x_positions,
                &mut y_positions,
                &mut u_tex_coords,
                &mut v_tex_coords,
            )
            .with_context(|| {
                format!("Failed to retrieve regular positions for glyph {}.", tile.glyph)
            })?;

            for (i, vertex) in self.foreground_quad_grid.quad_mut(x, y).iter_mut().enumerate() {
                vertex.position[0] = x_positions[i];
                vertex.position[1] = y_positions[i];
                vertex.color[0] = tile.foreground_color.0.r as f32 / std::u8::MAX as f32;
                vertex.color[1] = tile.foreground_color.0.g as f32 / std::u8::MAX as f32;
                vertex.color[2] = tile.foreground_color.0.b as f32 / std::u8::MAX as f32;
                vertex.color[3] = tile.foreground_color.0.a as f32 / std::u8::MAX as f32;
                vertex.tex_coords[0] = u_tex_coords[i] * u_normalize;
                vertex.tex_coords[1] = v_tex_coords[i] * v_normalize;
            }

            // Update outline positions, colors and tex coords.
            if !tile.outlined {
                self.outline_quad_grid.clear_xy(x, y);
                continue;
            }

            // Reset the outline quad to default if present.
            if self.outline_quad_grid.quad(x, y).is_none() {
                self.outline_quad_grid.reset_xy(x, y);
            }

            self.assign_glyph_positions(
                x,
                y,
                tile.glyph,
                tile.layout,
                true,
                &mut x_positions,
                &mut y_positions,
                &mut u_tex_coords,
                &mut v_tex_coords,
            )
            .with_context(|| {
                format!("Failed to retrieve outline positions for glyph {}.", tile.glyph)
            })?;

            // At this point the quad must be Some().
            for (i, vertex) in self.outline_quad_grid.quad_mut(x, y).unwrap().iter_mut().enumerate()
            {
                vertex.position[0] = x_positions[i];
                vertex.position[1] = y_positions[i];
                vertex.color[0] = tile.outline_color.0.r as f32 / std::u8::MAX as f32;
                vertex.color[1] = tile.outline_color.0.g as f32 / std::u8::MAX as f32;
                vertex.color[2] = tile.outline_color.0.b as f32 / std::u8::MAX as f32;
                vertex.color[3] = tile.outline_color.0.a as f32 / std::u8::MAX as f32;
                vertex.tex_coords[0] = u_tex_coords[i] * u_normalize;
                vertex.tex_coords[1] = v_tex_coords[i] * v_normalize;
            }
        }

        // Rebind the new data to the foreground and background VAOs.
        unsafe {
            gl::BindVertexArray(self.background_vao);
        }
        gl_error_unwrap!();

        self.background_quad_grid
            .rebind_vertices()
            .context("Failed to rebind updated background vertex data.")?;

        unsafe {
            gl::BindVertexArray(self.foreground_vao);
        }
        gl_error_unwrap!();

        self.foreground_quad_grid
            .rebind_vertices()
            .context("Failed to rebind updated foreground vertex data.")?;

        unsafe {
            gl::BindVertexArray(self.outline_vao);
        }
        gl_error_unwrap!();

        self.outline_quad_grid
            .rebind_vertices()
            .context("Failed to rebind updated outline vertex data.")?;

        Ok(())
    }

    pub fn render(&self) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw background.
            gl::UseProgram(self.background_program);
            gl_error_unwrap!();

            gl::BindVertexArray(self.background_vao);
            gl_error_unwrap!();

            gl::DrawElements(
                gl::TRIANGLES,
                self.background_quad_grid.indices_len(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl_error_unwrap!();

            // Draw foreground.
            gl::UseProgram(self.foreground_program);
            gl_error_unwrap!();

            gl::BindVertexArray(self.foreground_vao);
            gl_error_unwrap!();

            gl::DrawElements(
                gl::TRIANGLES,
                self.foreground_quad_grid.indices_len(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl_error_unwrap!();

            // Draw outlines.
            gl::BindVertexArray(self.outline_vao);
            gl_error_unwrap!();

            gl::DrawElements(
                gl::TRIANGLES,
                self.outline_quad_grid.indices_len(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl_error_unwrap!();

            Ok(())
        }
    }

    fn assign_glyph_positions(
        &self,
        x: u32,
        y: u32,
        glyph: char,
        layout: TileLayout,
        outline: bool,
        x_positions: &mut [f32; 4],
        y_positions: &mut [f32; 4],
        u_tex_coords: &mut [f32; 4],
        v_tex_coords: &mut [f32; 4],
    ) -> Result<()> {
        // TODO: Handle outlined
        let metric;

        if outline {
            metric =
                self.font_metrics.outline().get(&(glyph as u32)).with_context(|| {
                    format!("Failed to load outline metrics for glyph {}.", glyph)
                })?;
        } else {
            metric =
                self.font_metrics.regular().get(&(glyph as u32)).with_context(|| {
                    format!("Failed to load regular metrics for glyph {}.", glyph)
                })?;
        }

        let offset = self.calculate_glyph_offset(&metric, layout);

        // Top left.
        x_positions[0] = (x * self.tile_width) as f32 + offset.0;
        y_positions[0] = (y * self.tile_height) as f32 + offset.1;
        u_tex_coords[0] = metric.x as f32;
        v_tex_coords[0] = metric.y as f32;

        // Top right.
        x_positions[1] = ((x * self.tile_width) + metric.width) as f32 + offset.0;
        y_positions[1] = (y * self.tile_height) as f32 + offset.1;
        u_tex_coords[1] = (metric.x + metric.width) as f32;
        v_tex_coords[1] = metric.y as f32;

        // Bottom left.
        x_positions[2] = ((x * self.tile_width) + metric.width) as f32 + offset.0;
        y_positions[2] = ((y * self.tile_height) + metric.height) as f32 + offset.1;
        u_tex_coords[2] = (metric.x + metric.width) as f32;
        v_tex_coords[2] = (metric.y + metric.height) as f32;

        // Bottom right.
        x_positions[3] = (x * self.tile_width) as f32 + offset.0;
        y_positions[3] = ((y * self.tile_height) + metric.height) as f32 + offset.1;
        u_tex_coords[3] = metric.x as f32;
        v_tex_coords[3] = (metric.y + metric.height) as f32;

        Ok(())
    }

    fn calculate_glyph_offset(&self, metric: &GlyphMetric, layout: TileLayout) -> (f32, f32) {
        match layout {
            TileLayout::Center => (
                (self.tile_width as i32 - metric.width as i32) as f32 / 2.0,
                (self.tile_height as i32 - metric.height as i32) as f32 / 2.0,
            ),
            TileLayout::Floor => (
                (self.tile_width as i32 - metric.width as i32) as f32 / 2.0,
                (self.tile_height as i32 - metric.height as i32) as f32,
            ),
            TileLayout::Text => (metric.x_offset as f32, metric.y_offset as f32),
            TileLayout::Exact((x, y)) => (
                ((self.tile_width as i32 - metric.width as i32) as f32 / 2.0) + x as f32,
                ((self.tile_height as i32 - metric.height as i32) as f32 / 2.0) + y as f32,
            ),
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.atlas_texture);
            gl::DeleteProgram(self.foreground_program);
            gl::DeleteVertexArrays(1, &self.foreground_vao);
            gl::DeleteProgram(self.background_program);
            gl::DeleteVertexArrays(1, &self.background_vao);
        }
    }
}
