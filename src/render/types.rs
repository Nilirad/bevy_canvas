use bevy::{
    core::Byteable,
    math::{Vec2, Vec4},
    render::color::Color,
};
use tess::{
    math::{point, vector, Point, Vector},
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor,
};

// TODO: Should a z-component be added?
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub(crate) struct Vertex {
    pub pos: Vec2,
    pub color: Vec4,
}

unsafe impl Byteable for Vertex {}

type IndexType = u32;
pub(crate) type BufferPair = tess::VertexBuffers<Vertex, IndexType>;

pub(crate) struct VertexConstructor {
    pub color: Color,
}

impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            pos: Vec2::new(vertex.position().x, vertex.position().y),
            color: self.color.into(),
        }
    }
}

impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            pos: Vec2::new(vertex.position().x, vertex.position().y),
            color: self.color.into(),
        }
    }
}

pub(crate) trait Conversion {
    fn to_point(self) -> Point;
    fn to_vector(self) -> Vector;
}

impl Conversion for Vec2 {
    fn to_point(self) -> Point {
        point(self.x, self.y)
    }

    fn to_vector(self) -> Vector {
        vector(self.x, self.y)
    }
}
