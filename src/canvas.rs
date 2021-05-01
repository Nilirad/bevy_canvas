use bevy::{log::error, render::color::Color};
use tess::{
    path::Path, BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
};

use crate::render::types::{BufferPair, VertexConstructor};

// TODO: Consider adding a "passive" field to get a bit of retained mode.
/// A Bevy `Resource` that exposes an immediate mode 2D rendering API.
///
/// ## Usage
///
/// ```
/// use bevy::prelude::*;
/// use bevy_canvas::{
///     common_shapes::{Rectangle, RectangleAnchor},
///     Canvas, DrawMode, StrokeOptions,
/// };
///
/// // In your system, fetch Canvas as a mutable resource:
/// fn my_system(mut canvas: ResMut<Canvas>) {
///     // Create a shape...
///     let square = Rectangle {
///         origin: Vec2::ZERO,
///         extents: Vec2::splat(100.0),
///         anchor_point: RectangleAnchor::TopLeft,
///     };
///
///     // ...then draw it!
///     canvas.draw(&square, DrawMode::fill_simple(), Color::RED);
/// }
/// ```
pub struct Canvas {
    pub(crate) vertex_buffers: BufferPair,
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
}

impl Canvas {
    pub fn draw(
        &mut self,
        geometry: &impl Geometry,
        draw_mode: DrawMode,
        color: Color,
    ) -> &mut Self {
        let path = geometry.generate_path();

        match draw_mode {
            DrawMode::Fill(ref options) => self.fill(&path, options, color),
            DrawMode::Stroke(ref options) => self.stroke(&path, options, color),
        }

        self
    }

    fn fill(&mut self, path: &Path, options: &FillOptions, color: Color) {
        let mut buffers_builder =
            BuffersBuilder::new(&mut self.vertex_buffers, VertexConstructor { color });
        if let Err(e) = self
            .fill_tess
            .tessellate_path(path, options, &mut buffers_builder)
        {
            error!("FillTessellator error: {:?}", e);
        }
    }

    fn stroke(&mut self, path: &Path, options: &StrokeOptions, color: Color) {
        let mut buffers_builder =
            BuffersBuilder::new(&mut self.vertex_buffers, VertexConstructor { color });
        if let Err(e) = self
            .stroke_tess
            .tessellate_path(path, options, &mut buffers_builder)
        {
            error!("StrokeTessellator error: {:?}", e);
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            vertex_buffers: BufferPair::new(),
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
        }
    }
}

/// Determines how a struct can be transformed into a Lyon `Path`.
///
/// ## Usage
/// ```
/// use bevy::math::Vec2;
/// use bevy_canvas::{Geometry, Path, PathBuilder};
///
/// // First, we need to define our struct :3
/// struct UwuMouth;
///
/// impl Geometry for UwuMouth {
///     fn generate_path(&self) -> Path {
///         // we need a custom path
///         let mut b = PathBuilder::new();
///
///         // svg magic starts here...
///
///         // left lip ·ω·
///         b.cubic_bezier_to(
///             Vec2::new(-25.0, -60.0),
///             Vec2::new(-75.0, -60.0),
///             Vec2::new(-100.0, 10.0),
///         );
///
///         // return to monke lol
///         b.move_to(Vec2::ZERO);
///
///         // right lip ·ω·
///         b.cubic_bezier_to(
///             Vec2::new(25.0, -60.0),
///             Vec2::new(75.0, -60.0),
///             Vec2::new(100.0, 10.0),
///         ); // <-- why that sad face...
///
///         // ...drawing is fun! uwu
///         b.build()
///     }
/// }
/// ```
pub trait Geometry {
    fn generate_path(&self) -> Path;
}

/// Determines how a shape is tessellated (i.e. transformed from a parametric
/// representation to a triangle mesh).
#[derive(Clone, Copy)]
pub enum DrawMode {
    /// The shape is tessellated using a fill operation.
    Fill(FillOptions),
    /// The shape is tessellated using a stroke operation.
    Stroke(StrokeOptions),
}

impl DrawMode {
    pub fn stroke_1px() -> Self {
        Self::Stroke(StrokeOptions::default())
    }

    pub fn fill_simple() -> Self {
        Self::Fill(FillOptions::default())
    }
}
