use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;
use std::str;

use gl::types::*;

use glam::{Mat4, Vec3};

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::terminal::*;

const FONT_ATLAS_PATH: &str = "./resources/font_atlases";
const FONT_NAME: &str = "input_mono";

const VERTICES_PER_QUAD: u32 = 24;
const INDICES_PER_QUAD: u32 = 6;

const TILE_WIDTH: u32 = 24;
const TILE_HEIGHT: u32 = 34;
const TERMINAL_WIDTH: u32 = 103;
const TERMINAL_HEIGHT: u32 = 37;

const VERT_SRC: &str = r#"
#version 150

in vec2 position;
in vec4 color;

out vec4 vColor;

uniform mat4 mvp;

void main()
{
    vColor = color;
    gl_Position = mvp * vec4(position, 1.0, 1.0);
}
"#;

const FRAG_SRC: &str = r#"
#version 150

in vec4 vColor;

out vec4 color;

void main()
{
    color = vColor;
}
"#;

fn compile_shader(src: &str, shader_type: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes()).map_err(|e| e.to_string())?;

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

        gl::GetShaderInfoLog(
            shader,
            len,
            ptr::null_mut(),
            buffer.as_mut_ptr() as *mut GLchar,
        );
        let log = str::from_utf8(&buffer).map_err(|e| e.to_string())?;
        Err(log.into())
    }
}

fn delete_shader(shader: GLuint) {
    unsafe {
        gl::DeleteShader(shader);
    }
}

fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
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

        gl::GetProgramInfoLog(
            program,
            len,
            ptr::null_mut(),
            buffer.as_mut_ptr() as *mut GLchar,
        );
        let log = str::from_utf8(&buffer).map_err(|e| e.to_string())?;
        Err(log.into())
    }
}

fn get_attrib_location(program: GLuint, name: &str) -> Result<GLint, String> {
    let c_str = CString::new(name).map_err(|e| e.to_string())?;
    let location = unsafe { gl::GetAttribLocation(program, c_str.as_ptr()) };

    if location != -1 {
        Ok(location)
    } else {
        Err("Failed to find attrib.".into())
 
    }
}

fn get_uniform_location(program: GLuint, name: &str) -> Result<GLint, String> {
    let c_str = CString::new(name).map_err(|e| e.to_string())?;
    let location = unsafe { gl::GetUniformLocation(program, c_str.as_ptr()) };

    if location != -1 {
        Ok(location)
    } else {
        Err("Failed to find uniform.".into())
    }
}

fn delete_program(program: GLuint) {
    unsafe {
        gl::DeleteProgram(program);
    }
}

fn update_vertices_and_indices_for_coord(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    x_step: f32,
    y_step: f32,
    vertices: &mut [GLfloat],
    indices: &mut [GLuint],
    rng: &mut ThreadRng,
) {
    let r: f32;
    let g: f32;
    let b: f32;

    if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
        r = 1.0;
        g = 1.0;
        b = 1.0;
    } else {
        r = rng.gen();
        g = rng.gen();
        b = rng.gen();
    }

    let vcoord = ((x + (y * width)) * 24) as usize;

    vertices[vcoord] = x_step * x as f32;
    vertices[vcoord + 1] = y_step * y as f32;

    vertices[vcoord + 2] = r as GLfloat;
    vertices[vcoord + 3] = g as GLfloat;
    vertices[vcoord + 4] = b as GLfloat;
    vertices[vcoord + 5] = 1.0 as GLfloat;

    vertices[vcoord + 6] = (x_step * x as f32) + x_step;
    vertices[vcoord + 7] = y_step * y as f32;

    vertices[vcoord + 8] = r as GLfloat;
    vertices[vcoord + 9] = g as GLfloat;
    vertices[vcoord + 10] = b as GLfloat;
    vertices[vcoord + 11] = 1.0 as GLfloat;

    vertices[vcoord + 12] = (x_step * x as f32) + x_step;
    vertices[vcoord + 13] = (y_step * y as f32) + y_step;

    vertices[vcoord + 14] = r as GLfloat;
    vertices[vcoord + 15] = g as GLfloat;
    vertices[vcoord + 16] = b as GLfloat;
    vertices[vcoord + 17] = 1.0 as GLfloat;

    vertices[vcoord + 18] = x_step * x as f32;
    vertices[vcoord + 19] = (y_step * y as f32) + y_step;

    vertices[vcoord + 20] = r as GLfloat;
    vertices[vcoord + 21] = g as GLfloat;
    vertices[vcoord + 22] = b as GLfloat;
    vertices[vcoord + 23] = 1.0 as GLfloat;

    let icoord = ((x + (y * width)) * 6) as usize;
    let index = (x + (y * width)) * 4;

    indices[icoord] = index as GLuint;
    indices[icoord + 1] = index as GLuint + 1;
    indices[icoord + 2] = index as GLuint + 2;
    indices[icoord + 3] = index as GLuint;
    indices[icoord + 4] = index as GLuint + 2;
    indices[icoord + 5] = index as GLuint + 3;
}

fn generate_vertices_and_indices(
    width: u32,
    height: u32,
    tile_width: u32,
    tile_height: u32,
) -> (Vec<GLfloat>, Vec<GLuint>) {
    // 4 (vertices) * (2 (position) + 4 (color)).
    let vertices_len = (width * height * VERTICES_PER_QUAD) as usize;
    let mut vertices = vec![0.0 as GLfloat; vertices_len];
    // 6 incides per quad (2 triangles).
    let indices_len = (width * height * INDICES_PER_QUAD) as usize;
    let mut indices = vec![0 as GLuint; indices_len];

    let mut rng = rand::thread_rng();

    for x in 0..width {
        for y in 0..height {
            update_vertices_and_indices_for_coord(
                x,
                y,
                width,
                height,
                TILE_WIDTH as f32,
                TILE_HEIGHT as f32,
                &mut vertices,
                &mut indices,
                &mut rng,
            );
        }
    }

    (vertices, indices)
}

pub struct Renderer {
    program: GLuint,
    vertices: Vec<GLfloat>,
    indices: Vec<GLuint>,
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    vertex_array: GLuint,
    mvp_location: GLint,
}

impl Renderer {
    pub fn new() -> Result<Self, String> {
        let vertex_shader = compile_shader(VERT_SRC, gl::VERTEX_SHADER)?;
        let fragment_shader = compile_shader(FRAG_SRC, gl::FRAGMENT_SHADER)?;

        let program = link_program(vertex_shader, fragment_shader);

        // The shaders are no longer needed.
        delete_shader(vertex_shader);
        delete_shader(fragment_shader);

        let program = program?;

        let (vertices, indices) = generate_vertices_and_indices(TERMINAL_WIDTH, TERMINAL_HEIGHT, TILE_WIDTH, TILE_HEIGHT);

        let mut vertex_buffer = 0;
        let mut index_buffer = 0;
        let mut vertex_array = 0;

        unsafe {
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::GenBuffers(1, &mut index_buffer);
            gl::GenVertexArrays(1, &mut vertex_array);

            gl::BindVertexArray(vertex_array);

            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&vertices[0]),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&indices[0]),
                gl::STATIC_DRAW,
            );

            let location = get_attrib_location(program, "position")?;
            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (6 * mem::size_of::<GLfloat>()) as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(location as GLuint);

            let location = get_attrib_location(program, "color")?;
            gl::VertexAttribPointer(
                location as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (6 * mem::size_of::<GLfloat>()) as GLsizei,
                (2 * mem::size_of::<GLfloat>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);

            gl::UseProgram(program);

            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
        }

        let mvp_location = get_uniform_location(program, "mvp")?;

        Ok(Self {
            program,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            vertex_array,
            mvp_location,
        })
    }

    pub fn update_viewport(&self, (width, height): (u32, u32)) {
        const EFFECTIVE_WIDTH: f32 = (TILE_WIDTH * TERMINAL_WIDTH) as f32;
        const EFFECTIVE_HEIGHT: f32 = (TILE_HEIGHT * TERMINAL_HEIGHT) as f32;

        let x_ratio = width as f32 / EFFECTIVE_WIDTH;
        let y_ratio = height as f32 / EFFECTIVE_HEIGHT;

        let x_translate;
        let y_translate;
        let scale;

        if x_ratio > y_ratio {
            x_translate = ((width as f32 - (EFFECTIVE_WIDTH * y_ratio)) / 2.0).floor();
            y_translate = 0.0;
            scale = y_ratio;
        } else {
            x_translate = 0.0;
            y_translate = ((height as f32 - (EFFECTIVE_HEIGHT * x_ratio)) / 2.0).floor();
            scale = x_ratio;
        }

        let projection = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, 0.0, 1.0);
        let translate = Mat4::from_translation(Vec3::new(x_translate, y_translate, 0.0));
        let scale = Mat4::from_scale(Vec3::new(scale, scale, 1.0));

        let mvp_data = (projection * translate * scale).to_cols_array();

        unsafe {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
            gl::UniformMatrix4fv(
                self.mvp_location,
                1,
                gl::FALSE as GLboolean,
                mem::transmute(&mvp_data[0])
            );
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as GLint,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.index_buffer);
            gl::DeleteBuffers(1, &self.vertex_buffer);
        }

        delete_program(self.program);
    }
}
