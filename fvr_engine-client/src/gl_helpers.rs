//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::ffi::{c_void, CString};
use std::fmt::Display;
use std::path::Path;
use std::ptr;
use std::str;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use anyhow::{anyhow, bail, Context, Result};
use gl::types::*;
use image::DynamicImage;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------

// Number of vertices per quad when using glDrawElements.
pub const VERTICES_PER_QUAD: usize = 4;

// Number of indices per quad when using glDrawElements.
pub const INDICES_PER_QUAD: usize = 6;

//-------------------------------------------------------------------------------------------------
// Checks the current OpenGL error state and returns it as a result.
//-------------------------------------------------------------------------------------------------
#[allow(dead_code)]
pub fn gl_error_unwrap<D>(msg: Option<D>) -> Result<()>
where
    D: Display,
{
    let error = unsafe { gl::GetError() };

    if error != gl::NO_ERROR {
        // Match the error string for the error type.
        let e: String = match error {
            gl::INVALID_ENUM => "[OpenGL] Error: INVALID_ENUM".into(),
            gl::INVALID_VALUE => "[OpenGL] Error: INVALID_VALUE".into(),
            gl::INVALID_OPERATION => "[OpenGL] Error: INVALID_OPERATION".into(),
            gl::INVALID_FRAMEBUFFER_OPERATION => {
                "[OpenGL] Error: INVALID_FRAMEBUFFER_OPERATION".into()
            }
            gl::OUT_OF_MEMORY => "[OpenGL] Error: OUT_OF_MEMORY".into(),
            gl::STACK_UNDERFLOW => "[OpenGL] Error: STACK_UNDERFLOW".into(),
            gl::STACK_OVERFLOW => "[OpenGL] Error: STACK_OVERFLOW".into(),
            _ => format!("[OpenGL] Error: {}", error),
        };

        // Optionally print an error message.
        if let Some(msg) = msg {
            eprintln!("{}", msg);
        }

        bail!(e);
    }

    Ok(())
}

//-------------------------------------------------------------------------------------------------
// Macro so that we can disable calls via attributes instead of runtime checks.
//-------------------------------------------------------------------------------------------------
macro_rules! gl_error_unwrap {
    () => {
        #[cfg(debug_assertions)]
        gl_error_unwrap(None::<String>)?;
    };
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        gl_error_unwrap(Some($msg))?;
    };
}

//-------------------------------------------------------------------------------------------------
// Creates and compiles a new shader from a source string and type.
//-------------------------------------------------------------------------------------------------
pub fn compile_shader(src: &str, shader_type: GLenum) -> Result<GLuint> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes())
            .map_err(|e| anyhow!(e))
            .context("Failed to create cstring from shader source.")?;

        // Compile the shader.
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check the status.
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Return if OK.
        if status == gl::TRUE as GLint {
            return Ok(shader);
        }

        // Else return the error log.
        let mut len = 0_i32;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer = Vec::with_capacity(len as usize);
        buffer.set_len(len as usize - 1);

        gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        let log = String::from_utf8(buffer)
            .map_err(|e| anyhow!(e))
            .context("Failed to create str from shader error log buffer.")?;
        Err(anyhow!(log))
    }
}

//-------------------------------------------------------------------------------------------------
// Creates and links a new program with vertex and fragment shaders.
//-------------------------------------------------------------------------------------------------
pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint> {
    unsafe {
        let program = gl::CreateProgram();

        // Attach the shaders.
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);

        // Link the program.
        gl::LinkProgram(program);

        // Check the status.
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Return if OK.
        if status == gl::TRUE as GLint {
            return Ok(program);
        }

        // Else return the error log.
        let mut len = 0_i32;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buffer = Vec::with_capacity(len as usize);
        buffer.set_len(len as usize - 1);

        gl::GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
        let log = String::from_utf8(buffer)
            .map_err(|e| anyhow!(e))
            .context("Failed to create string from program error log buffer.")?;
        Err(anyhow!(log))
    }
}

//-------------------------------------------------------------------------------------------------
// Creates and links a new program from vertex and fragment shader sources.
//-------------------------------------------------------------------------------------------------
pub fn link_program_from_sources<S>(vertex_source: S, fragment_source: S) -> Result<GLuint>
where
    S: AsRef<str>,
{
    let vertex_shader = compile_shader(vertex_source.as_ref(), gl::VERTEX_SHADER)?;
    let fragment_shader = compile_shader(fragment_source.as_ref(), gl::FRAGMENT_SHADER)?;
    let program = link_program(vertex_shader, fragment_shader)?;

    // Shaders are no longer needed.
    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    Ok(program)
}

//-------------------------------------------------------------------------------------------------
// Returns location of an attrib within a program.
//-------------------------------------------------------------------------------------------------
pub fn get_attrib_location(program: GLuint, name: &str) -> Result<GLint> {
    let c_str = CString::new(name)
        .map_err(|e| anyhow!(e))
        .context("Failed to create cstring from attrib name.")?;
    let location = unsafe { gl::GetAttribLocation(program, c_str.as_ptr()) };

    if location != -1 {
        Ok(location)
    } else {
        Err(anyhow!("OpenGL] Failed to find attrib {}.", name))
    }
}

//-------------------------------------------------------------------------------------------------
// Returns location of a uniform within a program.
//-------------------------------------------------------------------------------------------------
pub fn get_uniform_location(program: GLuint, name: &str) -> Result<GLint> {
    let c_str = CString::new(name)
        .map_err(|e| anyhow!(e))
        .context("Failed to create cstring from uniform name.")?;
    let location = unsafe { gl::GetUniformLocation(program, c_str.as_ptr()) };

    if location != -1 {
        Ok(location)
    } else {
        Err(anyhow!("[OpenGL] Failed to find uniform {}.", name))
    }
}

//-------------------------------------------------------------------------------------------------
// Generates indices for rendering quad elements.
//-------------------------------------------------------------------------------------------------
pub fn generate_indices(num_quads: usize) -> Vec<GLuint> {
    let num_indices = num_quads * INDICES_PER_QUAD;
    let mut indices = vec![0; num_indices];

    let iter = (0..indices.len()).step_by(INDICES_PER_QUAD).enumerate();
    for (i, idx) in iter {
        let i = (i * 4) as GLuint;
        indices[idx] = i;
        indices[idx + 1] = i + 1;
        indices[idx + 2] = i + 2;
        indices[idx + 3] = i;
        indices[idx + 4] = i + 2;
        indices[idx + 5] = i + 3;
    }

    indices
}

// Binds and uploads the data from an image to a texture, returning the size.
pub fn load_texture<P>(path: P, texture: GLuint, active: GLenum) -> Result<(u32, u32)>
where
    P: AsRef<Path>,
{
    // Load the texture image into memory and convert to the proper format.
    //-----------------------------------------------------------------------------------------

    // Load the image data from disk.
    let texture_data = image::open(&path)
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Failed to load file at {}.", path.as_ref().display()))?;

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
        // Set the active texture.
        gl::ActiveTexture(active);
        gl_error_unwrap!("Failed to bind texture.");

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

        // Generate Mipmaps. TODO: Do we need to do this?
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl_error_unwrap!("Failed to generate mipmaps.");
    }

    Ok(texture_dimensions)
}
