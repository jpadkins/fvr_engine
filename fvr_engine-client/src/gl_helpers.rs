use std::ffi::CString;
use std::fmt::Display;
use std::ptr;
use std::str;

use anyhow::{anyhow, bail, Context, Result};

use gl::types::*;

// Number of vertices per quad when using glDrawElements.
pub const VERTICES_PER_QUAD: usize = 4;

// Number of indices per quad when using glDrawElements.
pub const INDICES_PER_QUAD: usize = 6;

// Checks the current OpenGL error state and returns it as a result.
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

// Macro so that we can disable calls via attributes instead of runtime checks.
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

// Creates and compiles a new shader from a source string anad type.
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
        let mut len = 0 as GLint;
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

// Creates and links a new program with vertex and fragment shaders.
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
        let mut len = 0 as GLint;
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

// Creates and links a new program from vertex and fragment shader sources.
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

// Returns location of an attrib within a program.
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

// Returns location of a uniform within a program.
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
