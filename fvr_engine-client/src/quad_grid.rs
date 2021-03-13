use std::{mem, ptr};

use anyhow::Result;
use gl::types::*;

use fvr_engine_core::prelude::*;

use crate::gl_helpers::*;

pub trait Vertex {
    fn size_of() -> usize;
    fn enable_attribs(program: GLuint) -> Result<()>;
}

pub trait QuadGridVertex: Copy + Default + Vertex {}

pub struct QuadGrid<V>
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

    pub fn new(width: u32, height: u32) -> Result<Self> {
        let vertices = GridMap::new(width, height);
        let indices = Self::generate_indices(width, height);

        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        gl_error_unwrap!();

        let mut ibo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ibo);
        }
        gl_error_unwrap!();

        Ok(Self { vertices, indices, vbo, ibo })
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

    pub fn bind_data(&self) -> Result<()> {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl_error_unwrap!();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.data().len() * 4 * V::size_of()) as GLsizeiptr,
                mem::transmute(&self.vertices.data()[0]),
                gl::DYNAMIC_DRAW,
            );
            gl_error_unwrap!();

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl_error_unwrap!();

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&self.indices[0]),
                gl::STATIC_DRAW,
            );
            gl_error_unwrap!();
        }

        Ok(())
    }

    pub fn rebind_vertices(&self) -> Result<()> {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl_error_unwrap!();

            let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
            gl_error_unwrap!();

            ptr::copy_nonoverlapping(
                mem::transmute(&self.vertices.data()[0]),
                ptr,
                self.vertices.data().len() * 4 * V::size_of(),
            );
            gl_error_unwrap!();

            gl::UnmapBuffer(gl::ARRAY_BUFFER);
            gl_error_unwrap!();
        }

        Ok(())
    }

    fn generate_indices(width: u32, height: u32) -> Vec<GLuint> {
        let num_indices = (width * height * Self::INDICES_PER_QUAD * 2) as usize;
        let mut indices = vec![0; num_indices];

        let iter = (0..indices.len()).step_by(Self::INDICES_PER_QUAD as usize).enumerate();
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