use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;
use std::str;

use anyhow::Result;

use gl::types::*;

use glam::{Mat4, Vec3};

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::terminal::*;

use fvr_engine_core::prelude::*;

const FONT_ATLAS_PATH: &str = "./resources/font_atlases";
const FONT_NAME: &str = "input_mono";

const TILE_WIDTH: u32 = 24;
const TILE_HEIGHT: u32 = 34;
const TERMINAL_WIDTH: u32 = 103;
const TERMINAL_HEIGHT: u32 = 37;

const BACKGROUND_VERTEX_SHADER_SOURCE: &str = r#"
#version 150

in vec2 position;
in vec3 color;

out vec4 v_color;

uniform mat4 mvp;

void main()
{
    v_color = vec4(color, 1.0);
    gl_Position = mvp * vec4(position, 1.0, 1.0);
}
"#;

const BACKGROUND_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 150

precision lowp float;

in vec4 v_color;

out vec4 color;

void main()
{
    color = v_color;
}
"#;

const FOREGROUND_VERTEX_SHADER_SOURCE: &str = r#"
#version 150

in vec2 position;
in vec4 color;
in vec2 tex_coords;

out vec4 v_color;
out vec2 v_tex_coords;

uniform mat4 mvp;

void main()
{
    v_color = color;
    v_tex_coords = tex_coords;
    gl_Position = mvp * vec4(position, 1.0, 1.0);
}
"#;

const FOREGROUND_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 150

precision lowp float;

in vec4 v_color;
in vec2 v_tex_coords;

out vec4 color;

uniform sampler2D texture;

void main()
{
    color = texture2D(texture, v_tex_coords) * v_color;
}
"#;

fn gl_error_unwrap() -> Result<(), String> {
    let error = unsafe { gl::GetError() };

    if error != gl::NO_ERROR {
        return match error {
            gl::INVALID_ENUM => Err("[OpenGL] Error: INVALID_ENUM".into()),
            gl::INVALID_VALUE => Err("[OpenGL] Error: INVALID_VALUE".into()),
            gl::INVALID_OPERATION => Err("[OpenGL] Error: INVALID_OPERATION".into()),
            gl::INVALID_FRAMEBUFFER_OPERATION => {
                Err("[OpenGL] Error: INVALID_FRAMEBUFFER_OPERATION".into())
            }
            gl::OUT_OF_MEMORY => Err("[OpenGL] Error: OUT_OF_MEMORY".into()),
            gl::STACK_UNDERFLOW => Err("[OpenGL] Error: STACK_UNDERFLOW".into()),
            gl::STACK_OVERFLOW => Err("[OpenGL] Error: STACK_OVERFLOW".into()),
            _ => Err(format!("[OpenGL] Error: {}", error)),
        };
    }

    Ok(())
}

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
        Err(format!("[OpenGL] Failed to find attrib {}.", name))
    }
}

fn get_uniform_location(program: GLuint, name: &str) -> Result<GLint, String> {
    let c_str = CString::new(name).map_err(|e| e.to_string())?;
    let location = unsafe { gl::GetUniformLocation(program, c_str.as_ptr()) };

    if location != -1 {
        Ok(location)
    } else {
        Err(format!("[OpenGL] Failed to find uniform {}.", name))
    }
}

trait Vertex {
    fn size_of() -> usize;
    fn enable_attribs(program: GLuint) -> Result<(), String>;
}

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

    fn enable_attribs(program: GLuint) -> Result<(), String> {
        unsafe {
            let location = get_attrib_location(program, "position")?;
            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(location as GLuint);

            gl_error_unwrap()?;

            let location = get_attrib_location(program, "color")?;
            gl::VertexAttribPointer(
                location as GLuint,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                (mem::size_of::<GLfloat>() * 2) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);

            gl_error_unwrap()?;
        }

        Ok(())
    }
}

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

    fn enable_attribs(program: GLuint) -> Result<(), String> {
        unsafe {
            let location = get_attrib_location(program, "position")?;
            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(location as GLuint);

            gl_error_unwrap()?;

            let location = get_attrib_location(program, "color")?;
            gl::VertexAttribPointer(
                location as GLuint,
                4,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                (mem::size_of::<GLfloat>() * 2) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);

            gl_error_unwrap()?;

            let location = get_attrib_location(program, "tex_coords")?;
            gl::VertexAttribPointer(
                location as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                Self::size_of() as GLsizei,
                (mem::size_of::<GLfloat>() * 6) as *const c_void,
            );
            gl::EnableVertexAttribArray(location as GLuint);

            gl_error_unwrap()?;
        }

        Ok(())
    }
}

trait QuadGridVertex: Copy + Default + Vertex {}

impl QuadGridVertex for BackgroundVertex {}
impl QuadGridVertex for ForegroundVertex {}

struct QuadGrid<V>
where
    V: QuadGridVertex,
{
    vertices: GridMap<[V; 4]>,
    indices: Vec<GLuint>,
    vbo: GLuint,
    ibo: GLuint,
}

impl<V> QuadGrid<V>
where
    V: QuadGridVertex,
{
    const INDICES_PER_QUAD: u32 = 6;

    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let vertices = GridMap::new(width, height);
        let indices = Self::generate_indices(width, height);

        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        gl_error_unwrap()?;

        let mut ibo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ibo);
        }

        gl_error_unwrap()?;

        Ok(Self {
            vertices,
            indices,
            vbo,
            ibo,
        })
    }

    pub fn width(&self) -> u32 {
        self.vertices.width()
    }

    pub fn height(&self) -> u32 {
        self.vertices.height()
    }

    pub fn quad(&self, x: u32, y: u32) -> &[V; 4] {
        self.vertices.get_xy(x, y)
    }

    pub fn quad_mut(&mut self, x: u32, y: u32) -> &mut [V; 4] {
        self.vertices.get_xy_mut(x, y)
    }

    pub fn indices_len(&self) -> GLint {
        self.indices.len() as GLint
    }

    pub fn bind_data(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.data().len() * 4 * V::size_of()) as GLsizeiptr,
                mem::transmute(&self.vertices.data()[0]),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&self.indices[0]),
                gl::STATIC_DRAW,
            );
        }
    }

    fn generate_indices(width: u32, height: u32) -> Vec<GLuint> {
        let num_indices = (width * height * Self::INDICES_PER_QUAD * 2) as usize;
        let mut indices = vec![0; num_indices];

        let iter = (0..indices.len())
            .step_by(Self::INDICES_PER_QUAD as usize)
            .enumerate();
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
}

impl<V> Drop for QuadGrid<V>
where
    V: QuadGridVertex,
{
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
        }
    }
}

pub struct Renderer {
    background_program: GLuint,
    foreground_program: GLuint,
    background_vao: GLuint,
    foreground_vao: GLuint,
    background_quad_grid: QuadGrid<BackgroundVertex>,
    foreground_quad_grid: QuadGrid<ForegroundVertex>,
    background_mvp_location: GLint,
    foreground_mvp_location: GLint,
}

impl Renderer {
    pub fn new() -> Result<Self, String> {
        // Setup background VAO.
        let mut background_vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut background_vao);
        }

        gl_error_unwrap()?;

        // Link background shader program.
        let vertex_shader = compile_shader(BACKGROUND_VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER)?;
        let fragment_shader =
            compile_shader(BACKGROUND_FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER)?;
        let program = link_program(vertex_shader, fragment_shader);

        // The shaders are no longer needed.
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        let background_program = program?;

        unsafe {
            gl::BindVertexArray(background_vao);
        }

        gl_error_unwrap()?;

        // Bind buffer data and enable attribs.
        let mut background_quad_grid =
            QuadGrid::<BackgroundVertex>::new(TERMINAL_WIDTH, TERMINAL_HEIGHT)?;

        // TODO: REMOVE
        let mut rng = rand::thread_rng();

        for x in 0..background_quad_grid.width() {
            for y in 0..background_quad_grid.height() {
                let mut quad = background_quad_grid.quad_mut(x, y);
                let r = rng.gen();
                let g = rng.gen();
                let b = rng.gen();

                quad[0].position[0] = (x * TILE_WIDTH) as GLfloat;
                quad[0].position[1] = (y * TILE_HEIGHT) as GLfloat;
                quad[0].color[0] = r;
                quad[0].color[1] = g;
                quad[0].color[2] = b;

                quad[1].position[0] = ((x * TILE_WIDTH) + TILE_WIDTH) as GLfloat;
                quad[1].position[1] = (y * TILE_HEIGHT) as GLfloat;
                quad[1].color[0] = r;
                quad[1].color[1] = g;
                quad[1].color[2] = b;

                quad[2].position[0] = ((x * TILE_WIDTH) + TILE_WIDTH) as GLfloat;
                quad[2].position[1] = ((y * TILE_HEIGHT) + TILE_HEIGHT) as GLfloat;
                quad[2].color[0] = r;
                quad[2].color[1] = g;
                quad[2].color[2] = b;

                quad[3].position[0] = (x * TILE_WIDTH) as GLfloat;
                quad[3].position[1] = ((y * TILE_HEIGHT) + TILE_HEIGHT) as GLfloat;
                quad[3].color[0] = r;
                quad[3].color[1] = g;
                quad[3].color[2] = b;
            }
        }
        // TODO: REMOVE

        background_quad_grid.bind_data();

        BackgroundVertex::enable_attribs(background_program)?;

        let background_mvp_location = get_uniform_location(background_program, "mvp")?;

        // Setup foreground VAO.
        let mut foreground_vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut foreground_vao);
        }

        gl_error_unwrap()?;

        // Link background shader program.
        let vertex_shader = compile_shader(FOREGROUND_VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER)?;
        let fragment_shader =
            compile_shader(FOREGROUND_FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER)?;
        let program = link_program(vertex_shader, fragment_shader);

        // The shaders are no longer needed.
        unsafe {
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }

        let foreground_program = program?;

        unsafe {
            gl::BindVertexArray(foreground_vao);
        }

        gl_error_unwrap()?;

        // Bind buffer data and enable attribs.
        let foreground_quad_grid =
            QuadGrid::<ForegroundVertex>::new(TERMINAL_WIDTH, TERMINAL_HEIGHT)?;
        foreground_quad_grid.bind_data();

        ForegroundVertex::enable_attribs(foreground_program)?;

        let foreground_mvp_location = get_uniform_location(foreground_program, "mvp")?;

        // Misc. OpenGL settings.
        unsafe {
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);

            // Optimized blending for opaque background.
            // https://apoorvaj.io/alpha-compositing-opengl-blending-and-premultiplied-alpha/
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        gl_error_unwrap()?;

        Ok(Self {
            background_program,
            foreground_program,
            background_vao,
            foreground_vao,
            background_quad_grid,
            foreground_quad_grid,
            background_mvp_location,
            foreground_mvp_location,
        })
    }

    pub fn update_viewport(&self, (width, height): (u32, u32)) -> Result<(), String> {
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
            gl::UniformMatrix4fv(
                self.background_mvp_location,
                1,
                gl::FALSE as GLboolean,
                mem::transmute(&mvp_data[0]),
            );

            gl_error_unwrap()?;

            gl::UseProgram(self.foreground_program);
            gl::UniformMatrix4fv(
                self.foreground_mvp_location,
                1,
                gl::FALSE as GLboolean,
                mem::transmute(&mvp_data[0]),
            );

            gl_error_unwrap()?;
        }

        Ok(())
    }

    pub fn render(&self) -> Result<(), String> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw background.
            gl::BindVertexArray(self.background_vao);
            gl::UseProgram(self.background_program);
            gl::DrawElements(
                gl::TRIANGLES,
                self.background_quad_grid.indices_len(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );

            gl_error_unwrap()?;

            // Draw foreground.
            // gl::BindVertexArray(self.foreground_vao);
            // gl::UseProgram(self.foreground_program);
            // gl::DrawElements(
            //     gl::TRIANGLES,
            //     self.foreground_quad_grid.indices_len() as GLint,
            //     gl::UNSIGNED_INT,
            //     ptr::null(),
            // );

            Ok(())
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.foreground_program);
            gl::DeleteVertexArrays(1, &self.foreground_vao);
            gl::DeleteProgram(self.background_program);
            gl::DeleteVertexArrays(1, &self.background_vao);
        }
    }
}
