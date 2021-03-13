use std::ffi::CString;
use std::ptr;
use std::str;

use anyhow::{anyhow, bail, Context, Result};

use gl::types::*;

// Checks the current OpenGL error state as returns it as a result.
pub fn gl_error_unwrap() -> Result<()> {
    let error = unsafe { gl::GetError() };

    if error != gl::NO_ERROR {
        let msg: String = match error {
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

        bail!(msg);
    }

    Ok(())
}

// Macro so that we can disable calls via attributes instead of runtime checks.
macro_rules! gl_error_unwrap {
    () => {
        #[cfg(debug_assertions)]
        gl_error_unwrap()?;
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
