use std::{mem, ptr};

use anyhow::Result;
use gl::types::*;

use fvr_engine_core::prelude::*;

use crate::gl_helpers::*;
use crate::quad_grid::*;

// A quad grid with optionally empty quads.
// OpenGL calls are optimized for the dynamic length.
pub struct SparseQuadGrid<V>
where
    V: QuadGridVertex,
{
    vertices: GridMap<Option<[V; 4]>>,
    indices: Vec<GLuint>,
    vertex_data: Vec<[V; 4]>,
    vbo: GLuint,
    ibo: GLuint,
}

impl<V> SparseQuadGrid<V>
where
    V: QuadGridVertex,
{
    // TODO: Move to gl_helpers
    const INDICES_PER_QUAD: u32 = 6;

    pub fn new(width: u32, height: u32) -> Result<Self> {
        let vertices = GridMap::new(width, height);
        let indices = Self::generate_indices(width, height);
        let vertex_data = Vec::new();

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

        Ok(Self { vertices, indices, vertex_data, vbo, ibo })
    }

    pub fn width(&self) -> u32 {
        self.vertices.width()
    }

    pub fn height(&self) -> u32 {
        self.vertices.height()
    }

    pub fn quad(&self, x: u32, y: u32) -> Option<&[V; 4]> {
        self.vertices.get_xy(x, y).as_ref().and_then(|quad| Some(quad))
    }

    pub fn quad_mut(&mut self, x: u32, y: u32) -> Option<&mut [V; 4]> {
        self.vertices.get_xy_mut(x, y).as_mut().and_then(|quad| Some(quad))
    }

    pub fn clear_xy(&mut self, x: u32, y: u32) {
        *self.vertices.get_xy_mut(x, y) = None;
    }

    pub fn reset_xy(&mut self, x: u32, y: u32) {
        *self.vertices.get_xy_mut(x, y) = Some(Default::default());
    }

    pub fn indices_len(&self) -> GLint {
        // Determine indices len based on current used quads.
        (self.vertex_data.len() * Self::INDICES_PER_QUAD as usize) as GLint
    }

    pub fn bind_data(&mut self) -> Result<()> {
        // Clear and repopulate the vertex data.
        self.refresh_vertex_data();

        // Fill vertex data with default values.
        // NOTE: Passing nullptr into glBufferData() does not work.
        for _ in 0..(self.width() * self.height()) {
            self.vertex_data.push([Default::default(); 4]);
        }

        // Bind the buffer data.
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl_error_unwrap!();

            // Set the buffer to the largest possible size for the grid.
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.data().len() * 4 * V::size_of()) as GLsizeiptr,
                mem::transmute(&self.vertex_data[0]),
                gl::DYNAMIC_DRAW,
            );
            gl_error_unwrap!();

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
            gl_error_unwrap!();

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices_len() as usize * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&self.indices[0]),
                gl::STATIC_DRAW,
            );
            gl_error_unwrap!();
        }

        Ok(())
    }

    pub fn rebind_vertices(&mut self) -> Result<()> {
        // Clear and repopulate the vertex data.
        self.refresh_vertex_data();

        // No need to rebind data if no quads are populated.
        if self.vertex_data.is_empty() {
            return Ok(());
        }

        // Rebind the buffer data.
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl_error_unwrap!();

            let ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY);
            gl_error_unwrap!();

            ptr::copy_nonoverlapping(
                mem::transmute(&self.vertex_data[0]),
                ptr,
                self.vertex_data.len() * 4 * V::size_of(),
            );
            gl_error_unwrap!();

            gl::UnmapBuffer(gl::ARRAY_BUFFER);
            gl_error_unwrap!();
        }

        Ok(())
    }

    fn refresh_vertex_data(&mut self) {
        self.vertex_data.clear();

        for vertices in self.vertices.data().iter() {
            if let Some(quad) = vertices {
                self.vertex_data.push(*quad);
            }
        }
    }

    // TODO: Move to gl_helpers
    fn generate_indices(width: u32, height: u32) -> Vec<GLuint> {
        let num_indices = (width * height * Self::INDICES_PER_QUAD) as usize;
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

impl<V> Drop for SparseQuadGrid<V>
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
