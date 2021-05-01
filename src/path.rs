use bevy::math::Vec2;
use tess::{
    math::Angle,
    path::{builder::WithSvg, path::Builder, EndpointId, Path},
};

use crate::render::types::Conversion;

// FIXME: Name conflicts with tess::path::traits::PathBuilder. Either use
// namespacing or choose another name.

/// Builder for a Lyon `Path` structure.
///
/// This type can be used standalone to just generate `Path`s, but its intended
/// use is to facilitate the implementation of complex geometries. Wiew the
/// [`Geometry`](crate::Geometry) trait documentation to learn more.
///
/// ## Usage
/// ```
/// use bevy::math::Vec2;
/// use bevy_canvas::PathBuilder;
///
/// let mut b = PathBuilder::new();
/// b.line_to(Vec2::new(100.0, 200.0));
/// b.quadratic_bezier_to(Vec2::new(150.0, 300.0), Vec2::splat(200.0));
///
/// let path = b.build();
/// ```
pub struct PathBuilder(WithSvg<Builder>);

impl PathBuilder {
    pub fn new() -> Self {
        Self(Builder::new().with_svg())
    }

    pub fn build(self) -> Path {
        self.0.build()
    }

    pub fn move_to(&mut self, to: Vec2) -> EndpointId {
        self.0.move_to(to.to_point())
    }

    pub fn line_to(&mut self, to: Vec2) -> EndpointId {
        self.0.line_to(to.to_point())
    }

    pub fn close(&mut self) {
        self.0.close();
    }

    pub fn quadratic_bezier_to(&mut self, ctrl: Vec2, to: Vec2) -> EndpointId {
        self.0.quadratic_bezier_to(ctrl.to_point(), to.to_point())
    }

    pub fn cubic_bezier_to(&mut self, ctrl1: Vec2, ctrl2: Vec2, to: Vec2) -> EndpointId {
        self.0
            .cubic_bezier_to(ctrl1.to_point(), ctrl2.to_point(), to.to_point())
    }

    pub fn arc(&mut self, center: Vec2, radii: Vec2, sweep_angle: f32, x_rotation: f32) {
        self.0.arc(
            center.to_point(),
            radii.to_vector(),
            Angle::radians(sweep_angle),
            Angle::radians(x_rotation),
        );
    }

    pub fn current_position(&self) -> Vec2 {
        let p = self.0.current_position();
        Vec2::new(p.x, p.y)
    }
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}
