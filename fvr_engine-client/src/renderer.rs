use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::str;

use anyhow::{anyhow, Context, Result};
use gl::types::*;
use glam::{Mat4, Vec3};
use image::DynamicImage;

use fvr_engine_core::prelude::*;

use crate::gl_helpers::*;
use crate::quad_grid::*;
use crate::shader_strings::*;
use crate::terminal::*;

// TODO: Move these to config file and pass them into the fvr_engine-client ctor.
const FONT_ATLAS_PATH: &str = "./resources/font_atlases";
const FONT_NAME: &str = "input_mono";

const TILE_WIDTH: u32 = 24;
const TILE_HEIGHT: u32 = 34;
const TERMINAL_WIDTH: u32 = 103;
const TERMINAL_HEIGHT: u32 = 37;

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
    background_program: GLuint,
    foreground_program: GLuint,
    background_vao: GLuint,
    foreground_vao: GLuint,
    background_quad_grid: QuadGrid<BackgroundVertex>,
    foreground_quad_grid: QuadGrid<ForegroundVertex>,
    background_mvp_location: GLint,
    foreground_mvp_location: GLint,
    font_atlas_texture: GLuint,
    font_atlas_width: u32,
    font_atlas_height: u32,
}

impl Renderer {
    pub fn new() -> Result<Self> {
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
            QuadGrid::<BackgroundVertex>::new(TERMINAL_WIDTH, TERMINAL_HEIGHT)
                .context("Failed to create background quad grid.")?;

        // Position data of the background quad grid will not change.
        for x in 0..background_quad_grid.width() {
            for y in 0..background_quad_grid.height() {
                let mut quad = background_quad_grid.quad_mut(x, y);

                quad[0].position[0] = (x * TILE_WIDTH) as GLfloat;
                quad[0].position[1] = (y * TILE_HEIGHT) as GLfloat;

                quad[1].position[0] = ((x * TILE_WIDTH) + TILE_WIDTH) as GLfloat;
                quad[1].position[1] = (y * TILE_HEIGHT) as GLfloat;

                quad[2].position[0] = ((x * TILE_WIDTH) + TILE_WIDTH) as GLfloat;
                quad[2].position[1] = ((y * TILE_HEIGHT) + TILE_HEIGHT) as GLfloat;

                quad[3].position[0] = (x * TILE_WIDTH) as GLfloat;
                quad[3].position[1] = ((y * TILE_HEIGHT) + TILE_HEIGHT) as GLfloat;
            }
        }

        background_quad_grid.bind_data().context("Failed to bind background quad grid data.")?;

        BackgroundVertex::enable_attribs(background_program)
            .context("Failed to enable background quad grid vertex attribs.")?;

        let background_mvp_location = get_uniform_location(background_program, "mvp")
            .context("Failed to obtain background MVP matrix uniform location.")?;

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
            QuadGrid::<ForegroundVertex>::new(TERMINAL_WIDTH, TERMINAL_HEIGHT)
                .context("Failed to create foreground quad grid.")?;

        foreground_quad_grid.bind_data().context("Failed to bind foreground quad grid data.")?;

        ForegroundVertex::enable_attribs(foreground_program)
            .context("Failed to enable foreground quad grid vertex attribs.")?;

        let foreground_mvp_location = get_uniform_location(foreground_program, "mvp")
            .context("Failed to obtain foreground MVP matrix uniform location.")?;

        // Load font atlas texture.
        let font_atlas_path = format!("{}/{}.png", FONT_ATLAS_PATH, FONT_NAME);
        let font_atlas_image = image::open(&font_atlas_path)
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to open font atlas image at {}.", font_atlas_path))?;
        let font_atlas_image = match font_atlas_image {
            DynamicImage::ImageRgba8(image) => image,
            other_format => other_format.to_rgba8(),
        };

        let (font_atlas_width, font_atlas_height) = font_atlas_image.dimensions();

        let mut font_atlas_texture = 0;

        unsafe {
            gl::GenTextures(1, &mut font_atlas_texture);
            gl_error_unwrap!();

            gl::BindTexture(gl::TEXTURE_2D, font_atlas_texture);
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
                font_atlas_width as GLsizei,
                font_atlas_height as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                font_atlas_image.as_ptr() as *const c_void,
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

        Ok(Self {
            background_program,
            foreground_program,
            background_vao,
            foreground_vao,
            background_quad_grid,
            foreground_quad_grid,
            background_mvp_location,
            foreground_mvp_location,
            font_atlas_texture,
            font_atlas_width,
            font_atlas_height,
        })
    }

    pub fn update_viewport(&self, (width, height): (u32, u32)) -> Result<()> {
        // Find the dimensions (in pixels) of the quad grid.
        const EFFECTIVE_WIDTH: f32 = (TILE_WIDTH * TERMINAL_WIDTH) as f32;
        const EFFECTIVE_HEIGHT: f32 = (TILE_HEIGHT * TERMINAL_HEIGHT) as f32;

        // Find the ratios of actual width/height to quad grid width/height.
        let x_ratio = width as f32 / EFFECTIVE_WIDTH;
        let y_ratio = height as f32 / EFFECTIVE_HEIGHT;

        let x_translate;
        let y_translate;
        let scale;

        // Depending on which ratio is larger, set the translation and scale to center the quad grid.
        if x_ratio > y_ratio {
            x_translate = ((width as f32 - (EFFECTIVE_WIDTH * y_ratio)) / 2.0).floor();
            y_translate = 0.0;
            scale = y_ratio;
        } else {
            x_translate = 0.0;
            y_translate = ((height as f32 - (EFFECTIVE_HEIGHT * x_ratio)) / 2.0).floor();
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
        // Iterate over dirty tiles in terminal.
        for ((x, y), tile) in terminal.dirty_tiles_iter() {
            // Update the background colors.
            for vertex in self.background_quad_grid.quad_mut(x, y) {
                vertex.color[0] = tile.background_color.0.r as f32 / 255.0;
                vertex.color[1] = tile.background_color.0.g as f32 / 255.0;
                vertex.color[2] = tile.background_color.0.b as f32 / 255.0;
            }

            // Update the foreground positions, colors, and tex coords.
            let (x_position, y_position, u_tex_coord, v_tex_coord) =
                self.positions_for_glyph(tile.glyph, tile.layout).with_context(|| {
                    format!("Failed to retrieve positions for glyph {}.", tile.glyph)
                })?;

            for vertex in self.foreground_quad_grid.quad_mut(x, y) {
                vertex.position[0] = x_position;
                vertex.position[0] = y_position;
                vertex.color[0] = tile.foreground_color.0.r as f32 / 255.0;
                vertex.color[1] = tile.foreground_color.0.g as f32 / 255.0;
                vertex.color[2] = tile.foreground_color.0.b as f32 / 255.0;
                vertex.color[3] = tile.foreground_color.0.a as f32 / 255.0;
                vertex.tex_coords[0] = u_tex_coord;
                vertex.tex_coords[1] = v_tex_coord;
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

        Ok(())
    }

    pub fn render(&self) -> Result<()> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw background.
            gl::BindVertexArray(self.background_vao);
            gl_error_unwrap!();

            gl::UseProgram(self.background_program);
            gl_error_unwrap!();

            gl::DrawElements(
                gl::TRIANGLES,
                self.background_quad_grid.indices_len(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl_error_unwrap!();

            // Draw foreground.
            gl::BindVertexArray(self.foreground_vao);
            gl_error_unwrap!();

            gl::UseProgram(self.foreground_program);
            gl_error_unwrap!();

            gl::DrawElements(
                gl::TRIANGLES,
                self.foreground_quad_grid.indices_len() as GLint,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl_error_unwrap!();

            Ok(())
        }
    }

    fn positions_for_glyph(&self, glyph: char, layout: TileLayout) -> Result<(f32, f32, f32, f32)> {
        Ok((0.0, 0.0, 0.0, 0.0))
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.font_atlas_texture);
            gl::DeleteProgram(self.foreground_program);
            gl::DeleteVertexArrays(1, &self.foreground_vao);
            gl::DeleteProgram(self.background_program);
            gl::DeleteVertexArrays(1, &self.background_vao);
        }
    }
}