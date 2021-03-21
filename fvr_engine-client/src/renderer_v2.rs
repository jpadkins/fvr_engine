//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Display;
use std::path::Path;
use std::{mem, ptr};

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, Context, Result};
use gl::types::*;
use glam::{Mat4, Vec3};
use image::DynamicImage;

//-------------------------------------------------------------------------------------------------
// Workspace includes.
//-------------------------------------------------------------------------------------------------
use fvr_engine_core::prelude::*;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::gl_helpers::*;
use crate::shader_strings::*;
use crate::terminal::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// Normalization value to convert u8 color to OpenGL float representation.
const COLOR_NORMALIZE_8BIT: GLfloat = 1.0 / 255.0;

//-------------------------------------------------------------------------------------------------
// Describes a vertex for a colored (+ alpha) and texture-mapped quad.
// The background shader program will only use position and color[3].
// The foreground shader program will use all properties.
//-------------------------------------------------------------------------------------------------
#[repr(C, packed)]
#[derive(Clone, Copy, Default, Debug)]
struct Vertex {
    // Position of the vertex [X, Y].
    position: [GLfloat; 2],
    // Color of the vertex [R, G, B, A].
    color: [GLfloat; 4],
    // Texture Position of the vertex [U, V].
    tex_coords: [GLfloat; 2],
}

//-------------------------------------------------------------------------------------------------
// RendererV2: Batched and BackBuffered edition.
//
// RendererV2 creates two sets of array buffers and vertex arrays and flips them every frame to
// avoid tying up the CPU and GPU. Drawing is done via DrawElements with a single index buffer.
//
// Every frame vertex data is generated from the current terminal state and uploaded to the array
// buffer that is not currently in use (being drawn from).
//
// A single vertex specification is used for both the "background" (basic colored quads) and
// "foreground" (colored and textured quads of glyphs or outlines). The background shader program
// simply ignores the unneeded data from the array buffer. This allows us to only use one array
// buffer for bother draw calls and avoid switching bindings.
//-------------------------------------------------------------------------------------------------
pub struct RendererV2 {
    // Dimensions of each tile in the terminal in # of pixels.
    tile_dimensions: (u32, u32),
    // Dimensions of the terminal in # of tiles.
    terminal_dimensions: (u32, u32),
    // Frame clear color.
    clear_color: SdlColor,
    // Stores index of current vertex buffer and vertex array (0 or 1).
    target_backbuffer: bool,
    // Single index buffer to store indices of max # of quads.
    index_buffer: GLuint,
    // Double vertex buffers to not tie the CPU and GPU.
    // (one will be mapped to memory and updated during the frame, the other rendered from)
    vertex_buffers: [GLuint; 2],
    // Shader program used for rendering the background.
    background_program: GLuint,
    // Vertex Arrays for storing background vertex attributes.
    background_vertex_arrays: [GLuint; 2],
    // Vec for collecting background quads each frame.
    background_vertices: Vec<Vertex>,
    // Location of the projection matrix in the background shader program.
    background_projection_location: GLint,
    // Shader program used for rendering the foreground.
    foreground_program: GLuint,
    // Vertex Arrays for storing foreground vertex attributes.
    foreground_vertex_arrays: [GLuint; 2],
    // Vec for collecting foreground quads each frame.
    foreground_vertices: Vec<Vertex>,
    // Location of the projection matrix in the foreground shader program.
    foreground_projection_location: GLint,
    // Main font atlas texture, containing both regular and outline glyphs.
    texture: GLuint,
    // Dimensions of the font atlas texture.
    texture_dimensions: (u32, u32),
    // Normalization values for texel in pixels to texel in OpenGL space.
    texel_normalize: (f32, f32),
    // Map of u32 codepoint to corresponding regular glyph metrics.
    regular_metrics: HashMap<u32, GlyphMetric>,
    // Map of u32 codepoint to corresponding outline glyph metrics.
    outline_metrics: HashMap<u32, GlyphMetric>,
}

impl RendererV2 {
    //---------------------------------------------------------------------------------------------
    // Create a new renderer.
    // (there should only ever be one)
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
        // Default clear color (this will change).
        let clear_color = SdlColor::RGB(25, 50, 75);

        // Start the vertex buffer / vertex array index at 0.
        let target_backbuffer = false;

        // Generate the OpenGL name values.
        //-----------------------------------------------------------------------------------------

        // Generate the index buffer.
        let mut index_buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut index_buffer);
        }
        gl_error_unwrap!("Failed to generate index buffer.");

        // Generate the two vertex buffers.
        let mut vertex_buffers: [GLuint; 2] = [0; 2];
        unsafe {
            gl::GenBuffers(2, &mut vertex_buffers[0]);
        }
        gl_error_unwrap!("Failed to generate vertex buffer.");

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
        gl_error_unwrap!("Failed to generate background vertex arrays.");

        // Generate the foreground program (compile shaders and link).
        let foreground_program = link_program_from_sources(
            FOREGROUND_VERTEX_SHADER_SOURCE,
            FOREGROUND_FRAGMENT_SHADER_SOURCE,
        )?;

        // Generate the foreground vertex array.
        let mut foreground_vertex_arrays: [GLuint; 2] = [0; 2];
        unsafe {
            gl::GenVertexArrays(2, &mut foreground_vertex_arrays[0]);
        }
        gl_error_unwrap!("Failed to generate foreground vertex arrays");

        // Generate the atlas texture.
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
        }
        gl_error_unwrap!("Failed to generate texture.");

        // Find the location of the projection matrix uniforms.
        //-----------------------------------------------------------------------------------------
        let background_projection_location = get_uniform_location(background_program, "projection")
            .context("Failed to obtain background projection matrix uniform location.")?;

        let foreground_projection_location = get_uniform_location(foreground_program, "projection")
            .context("Failed to obtain foreground projection matrix uniform location.")?;

        // Populate index buffer with max # of quads.
        //-----------------------------------------------------------------------------------------

        // The max # of quads is the total # of tiles in the terminal * 3.
        // (for background, foreground, and outline).
        let num_quads = (terminal_dimensions.0 * terminal_dimensions.1) as usize;
        let indices = generate_indices(num_quads * 3);

        // Bind the index buffer and upload the index data (we only need to do this once).
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
        }
        gl_error_unwrap!("Failed to bind index buffer.");

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
        gl_error_unwrap!("Failed to upload index buffer data.");

        // Populate the vertex buffers with blank data.
        //-----------------------------------------------------------------------------------------

        // The max # of bytes in the vertex buffers is:
        // max # of bytes in the background...
        let max_background_len = num_quads * VERTICES_PER_QUAD * mem::size_of::<Vertex>();
        // plus the max # of bytes in the foreground...
        let max_foreground_len = (num_quads * VERTICES_PER_QUAD * mem::size_of::<Vertex>()) * 2;
        // (times 2 to account for the regular and outline glyphs).
        let max_vertex_len = max_background_len + max_foreground_len;

        // Create an empty byte vec.
        let blank_vertex_data = vec![u8::default(); max_vertex_len];

        // Bind the buffers and upload the empty data.
        for i in 0..2 {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffers[i]);
            }
            gl_error_unwrap!("Failed to bind vertex buffer.");

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
            gl_error_unwrap!("Failed to upload vertex buffer data.");
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
            gl_error_unwrap!("Failed to bind background vertex array.");

            // Bind the element (index) buffer.
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            }
            gl_error_unwrap!("Failed to bind index buffer for background vertex array.");

            // Bind the vertex buffer.
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffers[i]);
            }
            gl_error_unwrap!("Failed to bind vertex buffer for background vertex array.");

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
                    mem::size_of::<Vertex>() as GLsizei,
                    // Offset.
                    ptr::null(),
                );
                gl_error_unwrap!(
                    "Failed to set position attrib pointer for background vertex array."
                );

                gl::EnableVertexAttribArray(location as GLuint);
                gl_error_unwrap!("Failed to enable position attrib for background vertex array.");
            }

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
                    mem::size_of::<Vertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 2) as *const c_void,
                );
                gl_error_unwrap!("Failed to set color attrib pointer for background vertex array.");

                gl::EnableVertexAttribArray(location as GLuint);
                gl_error_unwrap!("Failed to enable color attrib for background vertex array.");
            }
        }

        // Setup the foreground VAOs.
        //-----------------------------------------------------------------------------------------
        for i in 0..2 {
            // Bind the VAO.
            unsafe {
                gl::BindVertexArray(foreground_vertex_arrays[i]);
            }
            gl_error_unwrap!("Failed to bind foreground vertex array.");

            // Bind the element (index) buffer.
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            }
            gl_error_unwrap!("Failed to bind index buffer for foreground vertex array.");

            // Bind the vertex buffer.
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffers[i]);
            }
            gl_error_unwrap!("Failed to bind vertex buffer for foreground vertex array.");

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
                    mem::size_of::<Vertex>() as GLsizei,
                    // Offset.
                    ptr::null(),
                );
                gl_error_unwrap!(
                    "Failed to set position attrib pointer for foreground vertex array."
                );

                gl::EnableVertexAttribArray(location as GLuint);
                gl_error_unwrap!("Failed to enable position attrib for foreground vertex array.");
            }

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
                    mem::size_of::<Vertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 2) as *const c_void,
                );
                gl_error_unwrap!("Failed to set color attrib pointer for foreground vertex array.");

                gl::EnableVertexAttribArray(location as GLuint);
                gl_error_unwrap!("Failed to enable color attrib for foreground vertex array.");
            }

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
                    mem::size_of::<Vertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 6) as *const c_void,
                );
                gl_error_unwrap!(
                    "Failed to set tex_coords attrib pointer for foreground vertex array."
                );

                gl::EnableVertexAttribArray(location as GLuint);
                gl_error_unwrap!("Failed to enable tex_coords attrib for foreground vertex array.");
            }
        }

        // Load the texture image into memory and convert to the proper format.
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

        // Calculate texel normalization values.
        let texel_normalize =
            (1.0 / texture_dimensions.0 as f32, 1.0 / texture_dimensions.1 as f32);

        // Set the texture settings and upload the texture data.
        //-----------------------------------------------------------------------------------------
        unsafe {
            // Bind the texture.
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl_error_unwrap!("Failed to bind texture.");

            // Set the wrap to CLAMP_TO_EDGE to avoid seams at the edge of tiles.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl_error_unwrap!("Failed to set TEXTURE_WRAP_S parameter.");
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl_error_unwrap!("Failed to set TEXTURE_WRAP_T parameter.");

            // Set the filter to LINEAR to apply a blurring effect.
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl_error_unwrap!("Failed to set TEXTURE_MIN_FILTER parameter.");
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl_error_unwrap!("Failed to set TEXTURE_MAG_FILTER parameter.");

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
            gl_error_unwrap!("Failed to upload texture data.");

            // Generate Mipmaps for faster rasterization when scaling.
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl_error_unwrap!("Failed to generate mipmaps.");
        }

        // Misc. OpenGL settings.
        //-----------------------------------------------------------------------------------------
        unsafe {
            // Optimized blending settings for when the background is always opaque.
            // https://apoorvaj.io/alpha-compositing-opengl-blending-and-premultiplied-alpha/
            gl::Enable(gl::BLEND);
            gl_error_unwrap!("Failed to enable blend.");

            gl::BlendEquation(gl::FUNC_ADD);
            gl_error_unwrap!("Failed to set blend equation.");

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl_error_unwrap!("Failed to set blend func.");

            // Update the OpenGL clear color.
            gl::ClearColor(
                clear_color.r as GLfloat * COLOR_NORMALIZE_8BIT,
                clear_color.g as GLfloat * COLOR_NORMALIZE_8BIT,
                clear_color.b as GLfloat * COLOR_NORMALIZE_8BIT,
                1.0,
            );
            gl_error_unwrap!("Failed to set clear color.");
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
            clear_color,
            target_backbuffer,
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
            texel_normalize,
            regular_metrics,
            outline_metrics,
        })
    }

    //---------------------------------------------------------------------------------------------
    // Update the OpenGL viewport and projection matrices for a new window size.
    // (should be called whenever the window size changes and no more than once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn update_viewport(&self, (width, height): (u32, u32)) -> Result<()> {
        // Update the OpenGL viewport.
        unsafe {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
        }

        // Find the dimensions (in pixels) of the quad grid.
        let effective_width = (self.terminal_dimensions.0 * self.tile_dimensions.0) as f32;
        let effective_height = (self.terminal_dimensions.1 * self.tile_dimensions.1) as f32;

        // Find the ratios of actual width/height to quad grid width/height.
        let x_ratio = width as f32 / effective_width;
        let y_ratio = height as f32 / effective_height;

        // Depending on which ratio is larger, set the translation and scale to center the quad grid.
        let x_translate;
        let y_translate;
        let scale;

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

        // Upload the new uniform data to both the background and foreground shader programs.
        unsafe {
            gl::UseProgram(self.background_program);
            gl_error_unwrap!("Failed to use background program for updating projection.");

            gl::UniformMatrix4fv(
                self.background_projection_location,
                1,
                gl::FALSE as GLboolean,
                &mvp_data as *const f32,
            );
            gl_error_unwrap!("Failed to update background projection matrix.");

            gl::UseProgram(self.foreground_program);
            gl_error_unwrap!("Failed to use foreground program for updating projection.");

            gl::UniformMatrix4fv(
                self.foreground_projection_location,
                1,
                gl::FALSE as GLboolean,
                &mvp_data as *const f32,
            );
            gl_error_unwrap!("Failed to update foreground projection matrix.");
        }

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Push a colored quad onto the background vertices, based on a tile.
    //---------------------------------------------------------------------------------------------
    fn push_background_quad(&mut self, (x, y): (u32, u32), tile: &Tile) {
        let mut vertex = Vertex::default();

        // Each vertex of the quad shares the same color values (for now).
        vertex.color[0] = tile.background_color.0.r as GLfloat * COLOR_NORMALIZE_8BIT;
        vertex.color[1] = tile.background_color.0.g as GLfloat * COLOR_NORMALIZE_8BIT;
        vertex.color[2] = tile.background_color.0.b as GLfloat * COLOR_NORMALIZE_8BIT;

        // Top left.
        vertex.position[0] = (x * self.tile_dimensions.0) as GLfloat;
        vertex.position[1] = (y * self.tile_dimensions.1) as GLfloat;
        self.background_vertices.push(vertex);

        // Top right.
        vertex.position[0] = ((x * self.tile_dimensions.0) + self.tile_dimensions.0) as GLfloat;
        vertex.position[1] = (y * self.tile_dimensions.1) as GLfloat;
        self.background_vertices.push(vertex);

        // Bottom left.
        vertex.position[0] = ((x * self.tile_dimensions.0) + self.tile_dimensions.0) as GLfloat;
        vertex.position[1] = ((y * self.tile_dimensions.1) + self.tile_dimensions.1) as GLfloat;
        self.background_vertices.push(vertex);

        // Bottom right.
        vertex.position[0] = (x * self.tile_dimensions.0) as GLfloat;
        vertex.position[1] = ((y * self.tile_dimensions.1) + self.tile_dimensions.1) as GLfloat;
        self.background_vertices.push(vertex);
    }

    //---------------------------------------------------------------------------------------------
    // Calculate the offset for a glyph (in pixels) given a tile layout.
    //---------------------------------------------------------------------------------------------
    fn calculate_glyph_offset(&self, metric: &GlyphMetric, layout: TileLayout) -> (f32, f32) {
        match layout {
            // Center the glyph.
            TileLayout::Center => (
                (self.tile_dimensions.0 as i32 - metric.width as i32) as f32 / 2.0,
                (self.tile_dimensions.1 as i32 - metric.height as i32) as f32 / 2.0,
            ),
            // Center the glyph horizontally but align with the base of the quad vertically.
            TileLayout::Floor => (
                (self.tile_dimensions.0 as i32 - metric.width as i32) as f32 / 2.0,
                (self.tile_dimensions.1 as i32 - metric.height as i32) as f32,
            ),
            // Adjust the glyph based on font metrics.
            TileLayout::Text => (metric.x_offset as f32, metric.y_offset as f32),
            // Adjust the glyph from the center position by an exact offset.
            TileLayout::Exact((x, y)) => (
                ((self.tile_dimensions.0 as i32 - metric.width as i32) as f32 / 2.0) + x as f32,
                ((self.tile_dimensions.1 as i32 - metric.height as i32) as f32 / 2.0) + y as f32,
            ),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Push a colored and textured quad onto the foreground vertices, based on a tile.
    //---------------------------------------------------------------------------------------------
    fn push_foreground_quad(
        &mut self,
        (x, y): (u32, u32),
        tile: &Tile,
        outlined: bool,
    ) -> Result<()> {
        let mut vertex = Vertex::default();

        // Retrieve either the regular or outline metrics for the tile's glyph.
        let metric = if outlined {
            self.outline_metrics
                .get(&(tile.glyph as u32))
                .with_context(|| format!("Failed to load outline metric for glyph {}.", tile.glyph))
        } else {
            self.regular_metrics
                .get(&(tile.glyph as u32))
                .with_context(|| format!("Failed to load regular metric for glyph {}.", tile.glyph))
        }?;

        // Use either the foreground or outline color from the tile.
        let color = if outlined { tile.outline_color } else { tile.foreground_color };

        // Calculate the glyph offset for the tile's layout.
        let offset = self.calculate_glyph_offset(&metric, tile.layout);

        // Each vertex of the quad shares the same color values (for now).
        vertex.color[0] = color.0.r as GLfloat * COLOR_NORMALIZE_8BIT;
        vertex.color[1] = color.0.g as GLfloat * COLOR_NORMALIZE_8BIT;
        vertex.color[2] = color.0.b as GLfloat * COLOR_NORMALIZE_8BIT;
        vertex.color[3] = color.0.a as GLfloat * COLOR_NORMALIZE_8BIT;

        // Top left.
        vertex.position[0] = (x * self.tile_dimensions.0) as f32 + offset.0;
        vertex.position[1] = (y * self.tile_dimensions.1) as f32 + offset.1;
        vertex.tex_coords[0] = (metric.x as f32) * self.texel_normalize.0;
        vertex.tex_coords[1] = (metric.y as f32) * self.texel_normalize.1;
        self.foreground_vertices.push(vertex);

        // Top right.
        vertex.position[0] = ((x * self.tile_dimensions.0) + metric.width) as f32 + offset.0;
        vertex.position[1] = (y * self.tile_dimensions.1) as f32 + offset.1;
        vertex.tex_coords[0] = ((metric.x + metric.width) as f32) * self.texel_normalize.0;
        vertex.tex_coords[1] = (metric.y as f32) * self.texel_normalize.1;
        self.foreground_vertices.push(vertex);

        // Bottom left.
        vertex.position[0] = ((x * self.tile_dimensions.0) + metric.width) as f32 + offset.0;
        vertex.position[1] = ((y * self.tile_dimensions.1) + metric.height) as f32 + offset.1;
        vertex.tex_coords[0] = ((metric.x + metric.width) as f32) * self.texel_normalize.0;
        vertex.tex_coords[1] = ((metric.y + metric.height) as f32) * self.texel_normalize.1;
        self.foreground_vertices.push(vertex);

        // Bottom right.
        vertex.position[0] = (x * self.tile_dimensions.0) as f32 + offset.0;
        vertex.position[1] = ((y * self.tile_dimensions.1) + metric.height) as f32 + offset.1;
        vertex.tex_coords[0] = (metric.x as f32) * self.texel_normalize.0;
        vertex.tex_coords[1] = ((metric.y + metric.height) as f32) * self.texel_normalize.1;
        self.foreground_vertices.push(vertex);

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Sync the vertex state with the terminal.
    // (should be called once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn sync_with_terminal(&mut self, terminal: &Terminal) -> Result<()> {
        // Clear the vertex vecs.
        self.background_vertices.clear();
        self.foreground_vertices.clear();

        // Determine index for the current vertex buffer and vertex arrays.
        let noncurrent_index = !self.target_backbuffer as usize;

        // Iterate over all tiles, pushing quads for those that are visible.
        //-----------------------------------------------------------------------------------------
        for (coord, tile) in terminal.tiles_iter() {
            // Skip the background quad if it is not visible.
            if tile.background_color != TileColor::TRANSPARENT
                && tile.background_color.0 != self.clear_color
            {
                self.push_background_quad(coord, tile);
            }

            // Skip adding the foreground quads if they are not visible.
            if tile.glyph == ' '
                || tile.foreground_color == TileColor::TRANSPARENT
                || tile.foreground_color == tile.background_color
            {
                continue;
            }
            self.push_foreground_quad(coord, tile, false)
                .context("Failed to push foreground regular quad")?;

            // Skip adding the foreground outline quad if the tile is not outlined.
            if !tile.outlined {
                continue;
            }
            self.push_foreground_quad(coord, tile, true)
                .context("Failed to push foreground outline quad")?;
        }

        // Update the vertex buffer with the new vertex data.
        //-----------------------------------------------------------------------------------------

        // Bind the vertex buffer not currently being rendered.
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffers[noncurrent_index]);
            gl_error_unwrap!("Failed to bind vertex buffer for updating.");

            // Map the buffer into local memory.
            let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
            gl_error_unwrap!("Failed to map vertex buffer.");

            // Determine size of background vertices.
            let background_vertices_size =
                self.background_vertices.len() * mem::size_of::<Vertex>();

            // If background vertices are present, copy them into the buffer.
            if !self.background_vertices.is_empty() {
                ptr::copy_nonoverlapping(
                    // Source pointer.
                    mem::transmute(&self.background_vertices[0]),
                    // Destination pointer.
                    ptr,
                    // Size.
                    background_vertices_size,
                );
            }

            // // If foreground vertices are present, copy them into the buffer.
            if !self.foreground_vertices.is_empty() {
                // Determine the starting offset in the buffer for the foreground.
                let ptr = (ptr as usize) + background_vertices_size;

                ptr::copy_nonoverlapping(
                    // Source pointer.
                    mem::transmute(&self.foreground_vertices[0]),
                    // Destination pointer.
                    ptr as *mut c_void,
                    // Size.
                    self.foreground_vertices.len() * mem::size_of::<Vertex>(),
                );
            }

            // Unmap the buffer (OpenGL will upload the data when it's needed).
            gl::UnmapBuffer(gl::ARRAY_BUFFER);
            gl_error_unwrap!("Failed to unmap vertex buffer.");
        }

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Render a frame and flip the backbuffer.
    // (should be called once per frame (obviously lol)).
    //---------------------------------------------------------------------------------------------
    pub fn render(&mut self) -> Result<()> {
        // Clear the frame.
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Determine index for the current vertex arrays.
        let current_index = self.target_backbuffer as usize;

        // Calculate the index length and offset.
        let background_indices_len =
            (self.background_vertices.len() / VERTICES_PER_QUAD) * INDICES_PER_QUAD;
        let foreground_indices_len =
            (self.foreground_vertices.len() / VERTICES_PER_QUAD) * INDICES_PER_QUAD;
        let foreground_indices_offset = mem::size_of::<GLuint>() * background_indices_len;

        // Draw the background (solid colored quads).
        unsafe {
            // Enable the background shader program and vertex array.
            gl::UseProgram(self.background_program);
            gl_error_unwrap!("Failed to use background program for rendering.");

            gl::BindVertexArray(self.background_vertex_arrays[current_index]);
            gl_error_unwrap!("Failed to enable background vertex array for rendering.");

            // Draw the background quads.
            gl::DrawElements(
                // Mode.
                gl::TRIANGLES,
                // Size.
                background_indices_len as GLint,
                // Type.
                gl::UNSIGNED_INT,
                // Pointer (null because the background starts at the beginning of the VBO).
                ptr::null(),
            );
            gl_error_unwrap!("Failed to draw background elements.");
        }

        // Draw the foreground (regular + outline glyphs).
        unsafe {
            // Enable the foreground shader program and vertex array.
            gl::UseProgram(self.foreground_program);
            gl_error_unwrap!("Failed to use foreground program for rendering.");

            gl::BindVertexArray(self.foreground_vertex_arrays[current_index]);
            gl_error_unwrap!("Failed to enable foreground vertex array for rendering.");

            // Draw the foreground quads.
            gl::DrawElements(
                // Mode.
                gl::TRIANGLES,
                // Size.
                foreground_indices_len as GLint,
                // Type.
                gl::UNSIGNED_INT,
                // Pointer (offset by # of background indices).
                foreground_indices_offset as *const c_void,
            );
            gl_error_unwrap!("Failed to draw foreground elements.");
        }

        // Flip the targeted buffer / vertex arrays.
        self.target_backbuffer = !self.target_backbuffer;

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
// Delete OpenGL objects on drop.
//-------------------------------------------------------------------------------------------------
impl Drop for RendererV2 {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteVertexArrays(2, &self.foreground_vertex_arrays[0]);
            gl::DeleteProgram(self.foreground_program);
            gl::DeleteVertexArrays(2, &self.background_vertex_arrays[0]);
            gl::DeleteProgram(self.background_program);
            gl::DeleteBuffers(2, &self.vertex_buffers[0]);
            gl::DeleteBuffers(1, &self.index_buffer);
        }
    }
}
