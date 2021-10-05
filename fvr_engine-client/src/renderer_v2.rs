//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use fnv::FnvHashMap;
use std::ffi::c_void;
use std::path::Path;
use std::{mem, ptr};

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{Context, Result};
use gl::types::*;
use glam::{Mat4, Vec3};

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

// Relative path to the fonts directory.
const FONTS_PATH: &str = "./resources/fonts/";

// Whether to alternate drawing/updating between two sets of array buffers and vertex arrays.
const USE_DOUBLE_BUFFERS: bool = true;

// Whether to use signed distance field font rendering.
const USE_SDF_FONTS: bool = false;

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
    // Index of the texture to sample.
    tex_index: GLfloat,
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
// buffer for both draw calls and avoid switching bindings.
//-------------------------------------------------------------------------------------------------
pub struct RendererV2 {
    // Dimensions of each tile in the terminal in # of pixels.
    tile_dimensions: ICoord,
    // Dimensions of the terminal in # of tiles.
    terminal_dimensions: ICoord,
    // Frame clear color.
    clear_color: SdlColor,
    // Cached current size of the viewport.
    viewport: [GLint; 4],
    // Inverse projection matrix for converting screen coords to world coords.
    inverse_projection: Mat4,
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
    // Cached count of background indices for use when drawing.
    background_indices_len: [GLsizei; 2],
    // Shader program used for rendering the foreground.
    foreground_program: GLuint,
    // Vertex Arrays for storing foreground vertex attributes.
    foreground_vertex_arrays: [GLuint; 2],
    // Vec for collecting foreground quads each frame.
    foreground_vertices: Vec<Vertex>,
    // Location of the projection matrix in the foreground shader program.
    foreground_projection_location: GLint,
    // Cached count of foreground indices for use when drawing.
    foreground_indices_len: [GLsizei; 2],
    // Shader program used for rendering the vignette.
    vignette_program: GLuint,
    // A blank vertex array used when rendering the vignette.
    vignette_vertex_array: GLuint,
    // Array of font textures for every tile style.
    // The first half of the array will contain the non-outlined textures.
    // The second half of the array will contain the outlined textures.
    textures: [GLuint; TILE_STYLE_COUNT * 2],
    // Normalization values for texel in pixels to texel in OpenGL space for every font texture.
    // The first half of the array will contain the non-outlined texture normalization values.
    // The second half of the array will contain the outlined texture normalization values.
    texel_normalize: [(f32, f32); TILE_STYLE_COUNT * 2],
    // Vec of maps of u32 codepoint to corresponding glyph metrics for every font texture.
    // Length will equal TILE_STYLE_COUNT * 2.
    // The first half of the vec will contain maps for the non-outlined metrics.
    // The second half of the vec will contain maps for the outlined metrics.
    metrics: Vec<FnvHashMap<i32, GlyphMetric>>,
}

impl RendererV2 {
    //---------------------------------------------------------------------------------------------
    // Creates a new renderer.
    // (there should only ever be one)
    //---------------------------------------------------------------------------------------------
    pub fn new<S>(
        tile_dimensions: ICoord,
        terminal_dimensions: ICoord,
        font_name: S,
    ) -> Result<Self>
    where
        S: AsRef<str>,
    {
        // Default clear color (this will change).
        let clear_color = SdlColor::RGB(15, 25, 35);

        // Viewport will be set the first time the viewport is updated.
        let viewport = [GLint::default(); 4];

        // Inverse projection will be set the first time the viewport is updated.
        let inverse_projection = Mat4::IDENTITY;

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
        let foreground_program = if USE_SDF_FONTS {
            link_program_from_sources(
                FOREGROUND_VERTEX_SHADER_SOURCE,
                FOREGROUND_FRAGMENT_SHADER_SDF_SOURCE,
            )
        } else {
            link_program_from_sources(
                FOREGROUND_VERTEX_SHADER_SOURCE,
                FOREGROUND_FRAGMENT_SHADER_SOURCE,
            )
        }?;

        // Generate the foreground vertex array.
        let mut foreground_vertex_arrays: [GLuint; 2] = [0; 2];
        unsafe {
            gl::GenVertexArrays(2, &mut foreground_vertex_arrays[0]);
        }
        gl_error_unwrap!("Failed to generate foreground vertex arrays.");

        // Generate the vignette program (compile shaders and link).
        let vignette_program = link_program_from_sources(
            FULL_FRAME_VERTEX_SHADER_SOURCE,
            VIGNETTE_FRAGMENT_SHADER_SOURCE,
        )?;

        // Generate the vignette vertex array.
        let mut vignette_vertex_array = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vignette_vertex_array);
        }
        gl_error_unwrap!("Failed to generate vignette vertex array.");

        // Generate the style textures.
        let mut textures = [0; TILE_STYLE_COUNT * 2];
        unsafe {
            gl::GenTextures((TILE_STYLE_COUNT * 2) as GLint, &mut textures[0]);
        }
        gl_error_unwrap!("Failed to generate texture.");

        // Find the location of the projection matrix uniforms.
        //-----------------------------------------------------------------------------------------
        let background_projection_location =
            get_uniform_location(background_program, "projection")
                .context("Failed to obtain background projection matrix uniform location.")?;

        let foreground_projection_location =
            get_uniform_location(foreground_program, "projection")
                .context("Failed to obtain foreground projection matrix uniform location.")?;

        // Indices len will be updated whenever the vertex data is updated.
        //-----------------------------------------------------------------------------------------
        let background_indices_len = [Default::default(); 2];
        let foreground_indices_len = [Default::default(); 2];

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
        for buffer in vertex_buffers {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
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
                gl_error_unwrap!(
                    "Failed to set color attrib pointer for background vertex array."
                );

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
                gl_error_unwrap!(
                    "Failed to set color attrib pointer for foreground vertex array."
                );

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
                gl_error_unwrap!(
                    "Failed to enable tex_coords attrib for foreground vertex array."
                );
            }

            let location = get_attrib_location(foreground_program, "tex_index")
                .context("Failed to get foreground tex_index attrib location.")?;

            unsafe {
                gl::VertexAttribPointer(
                    // Attribute location.
                    location as GLuint,
                    // Size.
                    1,
                    // Type.
                    gl::FLOAT,
                    // Normalized.
                    gl::FALSE as GLboolean,
                    // Stride.
                    mem::size_of::<Vertex>() as GLsizei,
                    // Offset.
                    (mem::size_of::<GLfloat>() * 8) as *const c_void,
                );
                gl_error_unwrap!(
                    "Failed to set tex_index attrib pointer for foreground vertex array."
                );

                gl::EnableVertexAttribArray(location as GLuint);
                gl_error_unwrap!("Failed to enable tex_index attrib for foreground vertex array.");
            }
        }

        // Load and bind the style textures.
        //-----------------------------------------------------------------------------------------

        // Double length to account for outline versions.
        let mut texel_normalize = [Default::default(); TILE_STYLE_COUNT * 2];

        // Make sure the foreground program is in use before updating uniforms.
        unsafe {
            gl::UseProgram(foreground_program);
            gl_error_unwrap!("Failed to use foreground program when binding textures.");
        }

        // Bind and upload the non-outlined textures.
        for i in 0..TILE_STYLE_COUNT {
            // Get the texture path string.
            let extension = if USE_SDF_FONTS { "_sdf.png" } else { ".png" };
            let path_string =
                [FONTS_PATH, font_name.as_ref(), "/", TILE_STYLE_NAMES[i], extension].concat();

            let dimensions =
                load_texture(Path::new(&path_string), textures[i], gl::TEXTURE0 + i as GLuint)?;
            texel_normalize[i] = (1.0 / dimensions.0 as f32, 1.0 / dimensions.1 as f32);

            let location = get_uniform_location(foreground_program, TILE_STYLE_NAMES[i])?;
            unsafe {
                gl::Uniform1i(location, i as GLint);
                gl_error_unwrap!("Failed to set non-outlined sampler2D uniform value.");
            }
        }

        // Bind and upload the outlined textures.
        #[allow(clippy::needless_range_loop)]
        for i in 0..TILE_STYLE_COUNT {
            // Get the outline texture path string.
            let extension = if USE_SDF_FONTS { "_outline_sdf.png" } else { "_outline.png" };
            let path_string =
                [FONTS_PATH, font_name.as_ref(), "/", TILE_STYLE_NAMES[i], extension].concat();

            // Offset the index for outlined textures.
            let index = i + TILE_STYLE_COUNT;

            let dimensions = load_texture(
                Path::new(&path_string),
                textures[index],
                gl::TEXTURE0 + index as GLuint,
            )?;
            texel_normalize[index] = (1.0 / dimensions.0 as f32, 1.0 / dimensions.1 as f32);

            let location = get_uniform_location(
                foreground_program,
                &format!("{}_outline", TILE_STYLE_NAMES[i]),
            )?;
            unsafe {
                gl::Uniform1i(location, index as GLint);
                gl_error_unwrap!("Failed to set outlined sampler2D uniform value.");
            }
        }

        // Misc. OpenGL settings.
        //-----------------------------------------------------------------------------------------
        unsafe {
            gl::BlendEquation(gl::FUNC_ADD);
            gl_error_unwrap!("Failed to set blend equation.");

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl_error_unwrap!("Failed to set blend func.");

            // Ensure depth testing is enabled.
            gl::Enable(gl::DEPTH_TEST);
            gl_error_unwrap!("Failed to enable depth testing.");

            gl::DepthFunc(gl::ALWAYS);
            gl_error_unwrap!("Failed to set depth func.");

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

        let mut metrics = vec![FnvHashMap::default(); TILE_STYLE_COUNT * 2];

        // Load the non-outlined metrics.
        for i in 0..TILE_STYLE_COUNT {
            // Get the path string for the font metrics.
            let path_string =
                [FONTS_PATH, font_name.as_ref(), "/", TILE_STYLE_NAMES[i], ".toml"].concat();
            let path = Path::new(&path_string);

            // Read in the data from the metrics file and parse it as TOML.
            let metrics_toml = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read contents of file {}.", path.display()))?;

            let font_metrics: FontMetricsV2 =
                toml::from_str(&metrics_toml).context("Failed to parse font metrics TOML.")?;

            // Populate hash maps with non-outlined metrics for easy access.
            for metric in font_metrics.metrics {
                metrics[i].insert(metric.codepoint, metric);
            }
        }

        // Load the outlined metrics.
        for i in 0..TILE_STYLE_COUNT {
            // Get the path string for the outline font metrics.
            let path_string =
                [FONTS_PATH, font_name.as_ref(), "/", TILE_STYLE_NAMES[i], "_outline.toml"]
                    .concat();
            let path = Path::new(&path_string);

            // Read in the data from the metrics file and parse it as TOML.
            let metrics_toml = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read contents of file {}.", path.display()))?;

            let font_metrics: FontMetricsV2 =
                toml::from_str(&metrics_toml).context("Failed to parse font metrics TOML.")?;

            // Populate hash maps with outlined metrics for easy access.
            // (remembering to offset the index for outlined metrics)
            for metric in font_metrics.metrics {
                metrics[i + TILE_STYLE_COUNT].insert(metric.codepoint, metric);
            }
        }

        // ...and that's it!
        //-----------------------------------------------------------------------------------------
        Ok(Self {
            tile_dimensions,
            terminal_dimensions,
            clear_color,
            viewport,
            inverse_projection,
            target_backbuffer,
            index_buffer,
            vertex_buffers,
            background_program,
            background_vertex_arrays,
            background_vertices,
            background_projection_location,
            background_indices_len,
            foreground_program,
            foreground_vertex_arrays,
            foreground_vertices,
            foreground_projection_location,
            foreground_indices_len,
            vignette_program,
            vignette_vertex_array,
            textures,
            texel_normalize,
            metrics,
        })
    }

    //---------------------------------------------------------------------------------------------
    // Update the OpenGL viewport and projection matrices for a new window size.
    // (should be called whenever the window size changes and no more than once per frame)
    //---------------------------------------------------------------------------------------------
    pub fn update_viewport(&mut self, (width, height): ICoord) -> Result<()> {
        // Update the OpenGL viewport and query and save the new size.
        unsafe {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
            gl_error_unwrap!();

            gl::GetIntegerv(gl::VIEWPORT, &mut self.viewport[0]);
            gl_error_unwrap!();
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
        let projection =
            Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -100.0, 100.0);
        let translate = Mat4::from_translation(Vec3::new(x_translate, y_translate, 0.0));
        let scale = Mat4::from_scale(Vec3::new(scale, scale, 1.0));
        let combined = projection * translate * scale;

        let uniform_data = combined.to_cols_array();

        // Upload the new uniform data to both the background and foreground shader programs.
        unsafe {
            gl::UseProgram(self.background_program);
            gl_error_unwrap!("Failed to use background program for updating projection.");

            gl::UniformMatrix4fv(
                self.background_projection_location,
                1,
                gl::FALSE as GLboolean,
                &uniform_data as *const f32,
            );
            gl_error_unwrap!("Failed to update background projection matrix.");

            gl::UseProgram(self.foreground_program);
            gl_error_unwrap!("Failed to use foreground program for updating projection.");

            gl::UniformMatrix4fv(
                self.foreground_projection_location,
                1,
                gl::FALSE as GLboolean,
                &uniform_data as *const f32,
            );
            gl_error_unwrap!("Failed to update foreground projection matrix.");
        }

        // Save the inverse projection matrix for converting screen coords to world coords.
        self.inverse_projection = combined.inverse();

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Convert a coord in screen space to the corresponding coord in world space.
    //---------------------------------------------------------------------------------------------
    pub fn screen_to_world_coords(&self, (x, y): ICoord) -> Option<ICoord> {
        // Convert the screen coords to [-1, 1]
        let normalized_x = -1.0 + 2.0 * x as f32 / self.viewport[2] as f32;
        let normalized_y = 1.0 - 2.0 * y as f32 / self.viewport[3] as f32;

        // Apply the inverse projection matrix to convert to world coords.
        let projected = self.inverse_projection.mul_vec4(glam::Vec4::new(
            normalized_x,
            normalized_y,
            1.0,
            1.0,
        ));

        // Cast the coords to int.
        let x = projected.x as i32;
        let y = projected.y as i32;

        // Return the world coords if they are in bounds of the faux terminal.
        if x >= 0
            && x < self.terminal_dimensions.0 * self.tile_dimensions.0
            && y >= 0
            && y < self.terminal_dimensions.1 * self.tile_dimensions.1
        {
            Some((x, y))
        } else {
            None
        }
    }

    //---------------------------------------------------------------------------------------------
    // Convert a coord in screen space to the corresponding tile coord in the faux terminal.
    //---------------------------------------------------------------------------------------------
    pub fn screen_to_terminal_coords(&self, (x, y): ICoord) -> Option<ICoord> {
        let world = self.screen_to_world_coords((x, y))?;

        Some((world.0 / self.tile_dimensions.0, world.1 / self.tile_dimensions.1))
    }

    //---------------------------------------------------------------------------------------------
    // Push a colored quad onto the background vertices, based on a tile.
    //---------------------------------------------------------------------------------------------
    fn push_background_quad(&mut self, (x, y): ICoord, tile: &Tile, opacity: GLfloat) {
        let mut vertex = Vertex::default();

        // Each vertex of the quad shares the same color values (for now).
        vertex.color[0] = tile.background_color.0.r as GLfloat
            * COLOR_NORMALIZE_8BIT
            * opacity
            * tile.background_opacity;
        vertex.color[1] = tile.background_color.0.g as GLfloat
            * COLOR_NORMALIZE_8BIT
            * opacity
            * tile.background_opacity;
        vertex.color[2] = tile.background_color.0.b as GLfloat
            * COLOR_NORMALIZE_8BIT
            * opacity
            * tile.background_opacity;

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
    // TODO: Which produces fewer scaling artifacts - floor() or round()?
    //---------------------------------------------------------------------------------------------
    fn calculate_glyph_offset(&self, metric: &GlyphMetric, layout: TileLayout) -> (f32, f32) {
        match layout {
            // Center the glyph.
            TileLayout::Center => (
                ((self.tile_dimensions.0 - metric.width) as f32 / 2.0).floor(),
                ((self.tile_dimensions.1 - metric.height) as f32 / 2.0).floor(),
            ),
            // Center the glyph horizontally but align with the base of the quad vertically.
            TileLayout::Floor => (
                ((self.tile_dimensions.0 - metric.width) as f32 / 2.0).floor(),
                (self.tile_dimensions.1 - metric.height) as f32,
            ),
            // Adjust the glyph based on font metrics.
            TileLayout::Text => (metric.x_offset as f32, metric.y_offset as f32),
            // Adjust the glyph from the center position by an exact offset.
            TileLayout::Exact((x, y)) => (
                (((self.tile_dimensions.0 - metric.width) as f32 / 2.0) + x as f32).floor(),
                (((self.tile_dimensions.1 - metric.height) as f32 / 2.0) + y as f32).floor(),
            ),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Push a colored and textured quad onto the foreground vertices, based on a tile.
    //---------------------------------------------------------------------------------------------
    fn push_foreground_quad(
        &mut self,
        (x, y): ICoord,
        tile: &Tile,
        outline_quad: bool,
        opacity: GLfloat,
    ) -> Result<()> {
        let mut vertex = Vertex::default();

        // Find and set the texture/metric index.
        let index = if outline_quad {
            tile.style as usize + TILE_STYLE_COUNT
        } else {
            tile.style as usize
        };

        vertex.tex_index = index as GLfloat;

        // Retrieve the metrics for the tile's glyph and style.
        let metric = self.metrics[index]
            .get(&(tile.glyph as i32))
            .with_context(|| format!("Failed to load outline metric for glyph {}.", tile.glyph))?;

        // Use either the foreground or outline color from the tile.
        let color = if outline_quad { tile.outline_color } else { tile.foreground_color };

        // Calculate the glyph offset for the tile's layout.
        let offset = self.calculate_glyph_offset(metric, tile.layout);

        // Get the texel normalize values.
        let texel_normalize = &self.texel_normalize[index];

        // Each vertex of the quad shares the same color values (for now).
        if outline_quad {
            vertex.color[0] = color.0.r as GLfloat * COLOR_NORMALIZE_8BIT;
            vertex.color[1] = color.0.g as GLfloat * COLOR_NORMALIZE_8BIT;
            vertex.color[2] = color.0.b as GLfloat * COLOR_NORMALIZE_8BIT;
            vertex.color[3] = opacity as GLfloat * tile.outline_opacity;
        } else {
            vertex.color[0] = color.0.r as GLfloat * COLOR_NORMALIZE_8BIT;
            vertex.color[1] = color.0.g as GLfloat * COLOR_NORMALIZE_8BIT;
            vertex.color[2] = color.0.b as GLfloat * COLOR_NORMALIZE_8BIT;
            vertex.color[3] = opacity as GLfloat * tile.foreground_opacity;
        }

        // Top left.
        vertex.position[0] = (x * self.tile_dimensions.0) as f32 + offset.0;
        vertex.position[1] = (y * self.tile_dimensions.1) as f32 + offset.1;
        vertex.tex_coords[0] = (metric.x as f32) * texel_normalize.0;
        vertex.tex_coords[1] = (metric.y as f32) * texel_normalize.1;
        self.foreground_vertices.push(vertex);

        // Top right.
        vertex.position[0] = ((x * self.tile_dimensions.0) + metric.width) as f32 + offset.0;
        vertex.position[1] = (y * self.tile_dimensions.1) as f32 + offset.1;
        vertex.tex_coords[0] = ((metric.x + metric.width) as f32) * texel_normalize.0;
        vertex.tex_coords[1] = (metric.y as f32) * texel_normalize.1;
        self.foreground_vertices.push(vertex);

        // Bottom left.
        vertex.position[0] = ((x * self.tile_dimensions.0) + metric.width) as f32 + offset.0;
        vertex.position[1] = ((y * self.tile_dimensions.1) + metric.height) as f32 + offset.1;
        vertex.tex_coords[0] = ((metric.x + metric.width) as f32) * texel_normalize.0;
        vertex.tex_coords[1] = ((metric.y + metric.height) as f32) * texel_normalize.1;
        self.foreground_vertices.push(vertex);

        // Bottom right.
        vertex.position[0] = (x * self.tile_dimensions.0) as f32 + offset.0;
        vertex.position[1] = ((y * self.tile_dimensions.1) + metric.height) as f32 + offset.1;
        vertex.tex_coords[0] = (metric.x as f32) * texel_normalize.0;
        vertex.tex_coords[1] = ((metric.y + metric.height) as f32) * texel_normalize.1;
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

        // Get the opacity modifier for the entire terminal.
        let opacity = terminal.opacity();

        // Iterate over all tiles, pushing quads for those that are visible.
        //-----------------------------------------------------------------------------------------
        for (coord, tile) in terminal.coords_and_tiles_iter() {
            // Skip the background if it would not be visible.
            if tile.background_color.0.a != 0
                && tile.background_opacity > 0.0
                && tile.background_color.0 != self.clear_color
            {
                self.push_background_quad(coord, tile, opacity);
            }

            // Skip the foreground if it would not be visible
            if tile.glyph != ' ' && tile.foreground_color.0.a != 0 && tile.foreground_opacity > 0.0
            // TODO: Is this check worth fixing, performance wise? It is currently broken.
            // && tile.foreground_color != tile.background_color
            {
                self.push_foreground_quad(coord, tile, false, opacity)
                    .context("Failed to push foreground regular quad")?;
            }

            // Skip the foreground outline if it is not enabled or would not be visible.
            if tile.outlined && tile.outline_color.0.a != 0 && tile.outline_opacity > 0.0 {
                self.push_foreground_quad(coord, tile, true, opacity)
                    .context("Failed to push foreground outline quad")?;
            }
        }

        // Update the vertex buffer with the new vertex data.
        //-----------------------------------------------------------------------------------------

        // Determine index for the current vertex buffer and vertex arrays.
        let noncurrent_index =
            if USE_DOUBLE_BUFFERS { !self.target_backbuffer } else { self.target_backbuffer }
                as usize;

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

        // Calculate and cache the indices counts.
        self.background_indices_len[noncurrent_index] =
            ((self.background_vertices.len() / VERTICES_PER_QUAD) * INDICES_PER_QUAD) as GLsizei;

        self.foreground_indices_len[noncurrent_index] =
            ((self.foreground_vertices.len() / VERTICES_PER_QUAD) * INDICES_PER_QUAD) as GLsizei;

        Ok(())
    }

    //---------------------------------------------------------------------------------------------
    // Render a frame and flip the backbuffer.
    // (should be called once per frame (obviously lol)).
    //---------------------------------------------------------------------------------------------
    pub fn render(&mut self) -> Result<()> {
        // Clear the frame.
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Determine index for the current vertex arrays.
        let current_index = self.target_backbuffer as usize;

        // Draw the background (solid colored quads).
        unsafe {
            // Disable blending.
            gl::Disable(gl::BLEND);
            gl_error_unwrap!("Failed to disable blending.");

            // Disable depth testing.
            gl::DepthMask(gl::FALSE);
            gl_error_unwrap!("Failed to disable depth testing.");

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
                self.background_indices_len[current_index],
                // Type.
                gl::UNSIGNED_INT,
                // Pointer (null because the background starts at the beginning of the VBO).
                ptr::null(),
            );
            gl_error_unwrap!("Failed to draw background elements.");
        }

        // Calculate the foreground offset.
        let foreground_indices_offset =
            mem::size_of::<GLuint>() * self.background_indices_len[current_index] as usize;

        // Draw the foreground (regular + outline glyphs).
        unsafe {
            // Enable blending.
            gl::Enable(gl::BLEND);
            gl_error_unwrap!("Failed to enable blending.");

            // Enable depth testing.
            gl::DepthMask(gl::TRUE);
            gl_error_unwrap!("Failed to enable depth testing.");

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
                self.foreground_indices_len[current_index],
                // Type.
                gl::UNSIGNED_INT,
                // Pointer (offset by # of background indices).
                foreground_indices_offset as *const c_void,
            );
            gl_error_unwrap!("Failed to draw foreground elements.");
        }

        // Draw the vignette.
        unsafe {
            // Enable the vignette shader program and vertex array.
            gl::UseProgram(self.vignette_program);
            gl_error_unwrap!("Failed to use vignette program for rendering.");

            gl::BindVertexArray(self.vignette_vertex_array);
            gl_error_unwrap!("Failed to enable vignette vertex array for rendering.");

            // Draw the single vignette quad (generated by the vertex shader).
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            gl_error_unwrap!("Failed to draw vignette.");
        }

        // Flip the targeted buffer / vertex arrays.
        if USE_DOUBLE_BUFFERS {
            self.target_backbuffer = !self.target_backbuffer;
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
// Delete OpenGL objects on drop.
//-------------------------------------------------------------------------------------------------
impl Drop for RendererV2 {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures((TILE_STYLE_COUNT * 2) as GLint, &self.textures[0]);
            gl::DeleteVertexArrays(1, &self.vignette_vertex_array);
            gl::DeleteVertexArrays(2, &self.foreground_vertex_arrays[0]);
            gl::DeleteProgram(self.foreground_program);
            gl::DeleteVertexArrays(2, &self.background_vertex_arrays[0]);
            gl::DeleteProgram(self.background_program);
            gl::DeleteBuffers(2, &self.vertex_buffers[0]);
            gl::DeleteBuffers(1, &self.index_buffer);
        }
    }
}
