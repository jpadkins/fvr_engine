// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Display;
use std::path::Path;
use std::{mem, ptr};

// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Context, Result};
use gl::types::*;
use glam::{Mat4, Vec3};
use image::DynamicImage;

// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::gl_helpers::*;
use crate::shader_strings::*;

// BackgroundVertex describes a vertex for a simple opaquely colored quad.
//-------------------------------------------------------------------------------------------------
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug)]
struct BackgroundVertex {
    // Position of the vertex [X, Y].
    position: [GLfloat; 2],
    // Color of the vertex [R, G, B].
    color: [GLfloat; 3],
}

// ForegroundVertex describes a vertex for a colored (+ alpha) and texture-mapped quad.
//-------------------------------------------------------------------------------------------------
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug)]
struct ForegroundVertex {
    // Position of the vertex [X, Y].
    position: [GLfloat; 2],
    // Color of the vertex [R, G, B, A].
    color: [GLfloat; 4],
    // Texture Position of the vertex [U, V].
    tex_coords: [GLfloat; 2],
}

// RendererV2: Batched and BackBuffered edition.
//-------------------------------------------------------------------------------------------------
pub struct RendererV2 {
    // Dimensions of each tile in the terminal in # of pixels.
    tile_dimensions: (u32, u32),
    // Dimensions of the terminal in # of tiles.
    terminal_dimensions: (u32, u32),
    // Single index buffer to store indices of max # of quads.
    index_buffer: GLuint,
    // Double vertex buffers to not tie the CPU and GPU.
    vertex_buffers: [GLuint; 2],
    // Shader program used for rendering the background.
    background_program: GLuint,
    // Vertex Arrays for storing background vertex attributes.
    background_vertex_arrays: [GLuint; 2],
    // Vec for collecting background quads each frame.
    background_vertices: Vec<BackgroundVertex>,
    // Location of the projection matrix in the background shader program.
    background_projection_location: GLint,
    // Shader program used for rendering the foreground.
    foreground_program: GLuint,
    // Vertex Arrays for storing foreground vertex attributes.
    foreground_vertex_arrays: [GLuint; 2],
    // Vec for collecting foreground quads each frame.
    foreground_vertices: Vec<ForegroundVertex>,
    // Location of the projection matrix in the foreground shader program.
    foreground_projection_location: GLint,
    // Main font atlas texture, containing both regular and outline glyphs.
    texture: GLuint,
    // Dimensions of the font atlas texture.
    texture_dimensions: (u32, u32),
    // Map of u32 codepoint to corresponding regular glyph metrics.
    regular_metrics: HashMap<u32, GlyphMetric>,
    // Map of u32 codepoint to corresponding outline glyph metrics.
    outline_metrics: HashMap<u32, GlyphMetric>,
}

impl RendererV2 {
    // Create a new renderer (there should only ever be one).
    //---------------------------------------------------------------------------------------------
    pub fn new<P>(
        tile_dimensions: (u32, u32),
        terminal_dimensions: (u32, u32),
        texture_path: P,
        metrics_path: P,
    ) -> Result<Self>
    where
        P: AsRef<Path> + Display,
    {
        // Generate the OpenGL objects.
        //-----------------------------------------------------------------------------------------

        // Generate the index buffer.
        let mut index_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut index_buffer);
        }
        gl_error_unwrap!();

        // Generate the two vertex buffers.
        let mut vertex_buffers: [GLuint; 2] = [0; 2];
        unsafe {
            gl::GenBuffers(2, &mut vertex_buffers[0]);
        }
        gl_error_unwrap!();

        // Generate the background program (compile shaders and link).
        let background_program = link_program_from_sources(
            BACKGROUND_VERTEX_SHADER_SOURCE,
            BACKGROUND_FRAGMENT_SHADER_SOURCE,
        )?;

        // Generate the background vertex arrays.
        let mut background_vertex_arrays: [GLuint; 2] = [0; 2];
        unsafe {
            gl::GenVertexArrays(2, &mut background_vertex_arrays[0]);
        }
        gl_error_unwrap!();

        // Generate the foreground program (compile shaders and link).
        let foreground_program = link_program_from_sources(
            FOREGROUND_VERTEX_SHADER_SOURCE,
            FOREGROUND_VERTEX_SHADER_SOURCE,
        )?;

        // Generate the foreground vertex array.
        let mut foreground_vertex_arrays: [GLuint; 2] = [0; 2];
        unsafe {
            gl::GenVertexArrays(1, &mut foreground_vertex_arrays[0]);
        }
        gl_error_unwrap!();

        // Generate the atlas texture.
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        gl_error_unwrap!();

        // Find the location of the projection matrix uniforms.
        //-----------------------------------------------------------------------------------------
        let background_projection_location = get_uniform_location(background_program, "projection")
            .context("Failed to obtain background projection matrix uniform location.")?;

        let foreground_projection_location = get_uniform_location(foreground_program, "projection")
            .context("Failed to obtain background projection matrix uniform location.")?;

        // Populate index buffer with max # of quads.
        //-----------------------------------------------------------------------------------------

        // The max # of quads is the total # of tiles in the terminal * 3.
        // (for background, foreground, and outline).
        let num_quads = terminal_dimensions.0 * terminal_dimensions.1;
        let indices = generate_indices(num_quads * 3);

        // Bind the index buffer and upload the index data (we only need to do this once).
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
        }
        gl_error_unwrap!();

        unsafe {
            gl::BufferData(
                // Type of buffer.
                gl::ELEMENT_ARRAY_BUFFER,
                // Size of the data.
                (indices.len() as usize * mem::size_of::<GLuint>()) as GLsizeiptr,
                // Pointer.
                mem::transmute(&indices[0]),
                // Type of storage access.
                gl::STATIC_DRAW,
            );
        }
        gl_error_unwrap!();

        // Populate the vertex buffers with blank data.
        //-----------------------------------------------------------------------------------------

        // The max # of bytes in the vertex buffers is:
        // max # of bytes in the background...
        let max_background_len = num_quads as usize * mem::size_of::<BackgroundVertex>();
        // plus the max # of bytes in the foreground...
        let max_foreground_len = (num_quads as usize * mem::size_of::<ForegroundVertex>()) * 2;
        // (times 2 to account for the regular and outline glyphs).
        let max_vertex_len = max_background_len * max_foreground_len;

        // Create an empty byte vec.
        let blank_vertex_data = vec![u8::default(); max_vertex_len];

        // Bind the buffers and upload the empty data.
        for i in 0..2 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffers[i]);
            }
            gl_error_unwrap!();

            unsafe {
                gl::BufferData(
                    // Storage type.
                    gl::ARRAY_BUFFER,
                    // Size of the data.
                    blank_vertex_data.len() as GLsizeiptr,
                    // Pointer.
                    mem::transmute(&blank_vertex_data[0]),
                    // Type of storage access.
                    gl::STREAM_DRAW,
                );
            }
            gl_error_unwrap!();
        }

        // Initialize the vec vertex buffers to max capacity.
        //-----------------------------------------------------------------------------------------
        let background_vertices = Vec::with_capacity(num_quads as usize);
        let foreground_vertices = Vec::with_capacity(num_quads as usize * 2);

        // Setup the background VAOs.
        //-----------------------------------------------------------------------------------------
        for i in 0..2 {
            // Bind the VAO.
            unsafe {
                gl::BindVertexArray(background_vertex_arrays[i]);
            }
            gl_error_unwrap!();

            // Bind the vertex buffer.
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffers[i]);
            }
            gl_error_unwrap!();

            // Bind the element (index) buffer.
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            }
            gl_error_unwrap!();

            // Enable the background vertex attributes.
            let location = get_attrib_location(background_program, "position")
                .context("Failed to get background position attrib location.")?;

            unsafe {
                gl::VertexAttribPointer(
                    // Attribute location.
                    location as GLuint,
                    // Size.
                    2,
                    // Type.
                    gl::FLOAT,
                    // Normalized.
                    gl::FALSE as GLboolean,
                    // Stride.
                    mem::size_of::<BackgroundVertex>() as GLsizei,
                    // Offset.
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(location as GLuint);
            }
            gl_error_unwrap!();

            let location = get_attrib_location(background_program, "color")
                .context("Failed to get background color attrib location.")?;

            unsafe {
                gl::VertexAttribPointer(
                    // Attribute location.
                    location as GLuint,
                    // Size.
                    3,
                    // Type.
                    gl::FLOAT,
                    // Normalized.
                    gl::FALSE as GLboolean,
                    // Stride.
                    mem::size_of::<BackgroundVertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 2) as *const c_void,
                );
                gl::EnableVertexAttribArray(location as GLuint);
            }
            gl_error_unwrap!();
        }

        // Setup the foreground VAOs.
        //-----------------------------------------------------------------------------------------
        for i in 0..2 {
            // Bind the VAO.
            unsafe {
                gl::BindVertexArray(foreground_vertex_arrays[i]);
            }
            gl_error_unwrap!();

            // Bind the vertex buffer.
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffers[i]);
            }
            gl_error_unwrap!();

            // Bind the element (index) buffer.
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            }
            gl_error_unwrap!();

            // Enable the foreground vertex attributes.
            let location = get_attrib_location(foreground_program, "position")
                .context("Failed to get foreground position attrib location.")?;

            unsafe {
                gl::VertexAttribPointer(
                    // Attribute location.
                    location as GLuint,
                    // Size.
                    2,
                    // Type.
                    gl::FLOAT,
                    // Normalized.
                    gl::FALSE as GLboolean,
                    // Stride.
                    mem::size_of::<ForegroundVertex>() as GLsizei,
                    // Offset.
                    ptr::null(),
                );
                gl::EnableVertexAttribArray(location as GLuint);
            }
            gl_error_unwrap!();

            let location = get_attrib_location(foreground_program, "color")
                .context("Failed to get foreground color attrib location.")?;

            unsafe {
                gl::VertexAttribPointer(
                    // Attribute location.
                    location as GLuint,
                    // Size.
                    4,
                    // Type.
                    gl::FLOAT,
                    // Normalized.
                    gl::FALSE as GLboolean,
                    // Stride.
                    mem::size_of::<ForegroundVertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 2) as *const c_void,
                );
                gl::EnableVertexAttribArray(location as GLuint);
            }
            gl_error_unwrap!();

            let location = get_attrib_location(foreground_program, "tex_coords")
                .context("Failed to get foreground tex_coords attrib location.")?;

            unsafe {
                gl::VertexAttribPointer(
                    // Attribute location.
                    location as GLuint,
                    // Size.
                    2,
                    // Type.
                    gl::FLOAT,
                    // Normalized.
                    gl::FALSE as GLboolean,
                    // Stride.
                    mem::size_of::<ForegroundVertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 6) as *const c_void,
                );
                gl::EnableVertexAttribArray(location as GLuint);
            }
            gl_error_unwrap!();
        }

        // Load the texture image into memory and convert to the proper format..
        //-----------------------------------------------------------------------------------------

        // Load the image data from disk.
        let texture_data = image::open(&texture_path)
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to load file at {}.", texture_path))?;

        // Convert to RGBA8 if not already.
        let texture_data = match texture_data {
            DynamicImage::ImageRgba8(data) => data,
            other => other.to_rgba8(),
        };

        // Query the dimensions
        let texture_dimensions = texture_data.dimensions();

        // Set the texture settings and upload the texture data.
        //-----------------------------------------------------------------------------------------
        unsafe {
            // Bind the texture.
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl_error_unwrap!();

            // Set the wrap to CLAMP_TO_EDGE to avoid seams at the edge of tiles.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl_error_unwrap!();
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl_error_unwrap!();

            // Set the filter to LINEAR to apply a blurring effect.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl_error_unwrap!();
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl_error_unwrap!();

            // Upload the texture data.
            gl::TexImage2D(
                // Target.
                gl::TEXTURE_2D,
                // Level.
                0,
                // Internal format.
                gl::RGBA as GLint,
                // Width.
                texture_dimensions.0 as GLsizei,
                // Height.
                texture_dimensions.1 as GLsizei,
                // Border.
                0,
                // Format.
                gl::RGBA,
                // Type.
                gl::UNSIGNED_BYTE,
                // Pointer.
                texture_data.as_ptr() as *const c_void,
            );
            gl_error_unwrap!();

            // Generate Mipmaps for faster rasterization when scaling.
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl_error_unwrap!();
        }

        // Misc. OpenGL settings.
        //-----------------------------------------------------------------------------------------
        unsafe {
            // Optimized blending settings for when the background is always opaque.
            // https://apoorvaj.io/alpha-compositing-opengl-blending-and-premultiplied-alpha/
            gl::Enable(gl::BLEND);
            gl_error_unwrap!();

            gl::BlendEquation(gl::FUNC_ADD);
            gl_error_unwrap!();

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl_error_unwrap!();

            // Set the initial clear color (this will change).
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl_error_unwrap!();
        }

        // Load the glyph metrics.
        //-----------------------------------------------------------------------------------------

        // Read in the data from the metrics file and parse it as TOML.
        let metrics_toml = std::fs::read_to_string(&metrics_path)
            .with_context(|| format!("Failed to read contents of file {}.", metrics_path))?;

        let font_metrics: FontMetrics =
            toml::from_str(&metrics_toml).context("Failed to parse font metrics TOML.")?;

        // Populate hash maps with regular and outline metrics for easy access.
        let mut regular_metrics = HashMap::new();

        for metric in font_metrics.regular {
            regular_metrics.insert(metric.codepoint, metric);
        }

        let mut outline_metrics = HashMap::new();

        for metric in font_metrics.outline {
            outline_metrics.insert(metric.codepoint, metric);
        }

        // ...and that's it!
        //-----------------------------------------------------------------------------------------
        Ok(Self {
            tile_dimensions,
            terminal_dimensions,
            index_buffer,
            vertex_buffers,
            background_program,
            background_vertex_arrays,
            background_vertices,
            background_projection_location,
            foreground_program,
            foreground_vertex_arrays,
            foreground_vertices,
            foreground_projection_location,
            texture,
            texture_dimensions,
            regular_metrics,
            outline_metrics,
        })
    }
}
