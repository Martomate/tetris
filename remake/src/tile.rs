#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Tile {
    pub vertices: [Vertex; 6],
}

impl Tile {
    pub fn new(w: f32, h: f32) -> Tile {
        Tile {
            vertices: [
                // upper triangle
                Vertex {
                    position: [0.0, h],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [w, h],
                    tex_coords: [1.0, 0.0],
                },
                // lower triangle
                Vertex {
                    position: [w, h],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [w, 0.0],
                    tex_coords: [1.0, 1.0],
                },
            ],
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        for v in &mut self.vertices {
            v.position[0] += x;
            v.position[1] += y;
        }
        self
    }
}
