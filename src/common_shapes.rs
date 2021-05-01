// TODO: Add rotation to these structs.
// TODO: Add rectangle with rounded borders.

//! A convenient collection of shapes.
//!
//! These shapes are provided to help users draw basic shapes without needing to
//! implement the [`Geometry`](crate::Geometry) trait themselves.

use bevy::math::Vec2;
use tess::{
    math::{point, Angle, Point, Rect, Size},
    path::{path::Builder, traits::PathBuilder, Polygon as LyonPolygon, Winding},
};

use crate::{render::types::Conversion, Geometry};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RectangleAnchor {
    Center,
    BottomLeft,
    BottomRight,
    TopRight,
    TopLeft,
}

impl Default for RectangleAnchor {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    /// Reference point of the shape.
    pub origin: Vec2,
    /// Width and height.
    pub extents: Vec2,
    /// Part of the shape located at the
    /// `origin`.
    pub anchor_point: RectangleAnchor,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            extents: Vec2::ONE,
            anchor_point: RectangleAnchor::default(),
        }
    }
}

impl Geometry for Rectangle {
    fn generate_path(&self) -> tess::path::Path {
        let mut b = Builder::new();
        let mut origin = match self.anchor_point {
            RectangleAnchor::Center => Point::new(-self.extents.x / 2.0, -self.extents.y / 2.0),
            RectangleAnchor::BottomLeft => Point::new(0.0, 0.0),
            RectangleAnchor::BottomRight => Point::new(-self.extents.x, 0.0),
            RectangleAnchor::TopRight => Point::new(-self.extents.x, -self.extents.y),
            RectangleAnchor::TopLeft => Point::new(0.0, -self.extents.y),
        };
        origin.x += self.origin.x;
        origin.y += self.origin.y;

        b.add_rectangle(
            &Rect::new(origin, Size::new(self.extents.x, self.extents.y)),
            Winding::Positive,
        );
        b.build()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            radius: 1.0,
        }
    }
}

impl Geometry for Circle {
    fn generate_path(&self) -> tess::path::Path {
        let mut b = Builder::new();
        b.add_circle(self.center.to_point(), self.radius, Winding::Positive);
        b.build()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse {
    pub center: Vec2,
    pub radii: Vec2,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            radii: Vec2::ONE,
        }
    }
}

impl Geometry for Ellipse {
    fn generate_path(&self) -> tess::path::Path {
        let mut b = Builder::new();
        b.add_ellipse(
            self.center.to_point(),
            self.radii.to_vector(),
            Angle::zero(),
            Winding::Positive,
        );
        b.build()
    }
}

/// Describes a polygon or a polyline.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    /// Reference point of the shape.
    pub origin: Vec2,
    /// The positions of the vertices of the polygon.
    pub points: Vec<Vec2>,
    /// Determines if the shape will be closed (a polygon) or remain open (a
    /// polyline).
    pub closed: bool,
}

impl Default for Polygon {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            points: Vec::new(),
            closed: true,
        }
    }
}

impl Geometry for Polygon {
    fn generate_path(&self) -> tess::path::Path {
        let mut b = Builder::new();
        let points = self
            .points
            .iter()
            .map(|p| (*p + self.origin).to_point())
            .collect::<Vec<Point>>();
        let polygon: LyonPolygon<Point> = LyonPolygon {
            points: points.as_slice(),
            closed: self.closed,
        };

        b.add_polygon(polygon);
        b.build()
    }
}

/// The base feature of a regular polygon that will be used to determine its
/// size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegularPolygonFeature {
    /// The radius of the polygon's circumcircle.
    Radius(f32),
    /// The radius of the polygon's incircle.
    Apothem(f32),
    /// The length of the polygon's side.
    SideLength(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegularPolygon {
    pub center: Vec2,
    /// The number of sides.
    pub sides: usize,
    /// The radius, apothem or side length of the polygon.
    pub feature: RegularPolygonFeature,
}

impl RegularPolygon {
    /// Calculates the radius of a regular polygon from its shape features.
    fn radius(&self) -> f32 {
        let ratio = std::f32::consts::PI / self.sides as f32;

        match self.feature {
            RegularPolygonFeature::Radius(r) => r,
            RegularPolygonFeature::Apothem(a) => a * ratio.tan() / ratio.sin(),
            RegularPolygonFeature::SideLength(s) => s / (2.0 * ratio.sin()),
        }
    }
}

impl Default for RegularPolygon {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            sides: 3,
            feature: RegularPolygonFeature::Radius(1.0),
        }
    }
}

impl Geometry for RegularPolygon {
    fn generate_path(&self) -> tess::path::Path {
        // -- Implementation details **PLEASE KEEP UPDATED** --
        // - `step`: angle between two vertices.
        // - `internal`: internal angle of the polygon.
        // - `offset`: bias to make the shape lay flat on a line parallel to the x-axis.

        use std::f32::consts::PI;
        assert!(self.sides > 2, "Polygons must have at least 3 sides");
        let mut b = Builder::new();

        let n = self.sides as f32;
        let radius = self.radius();
        let internal = (n - 2.0) * PI / n;
        let offset = -internal / 2.0;

        let mut points = Vec::with_capacity(self.sides);
        let step = 2.0 * PI / n;
        for i in 0..self.sides {
            let cur_angle = (i as f32).mul_add(step, offset);
            let x = radius.mul_add(cur_angle.cos(), self.center.x);
            let y = radius.mul_add(cur_angle.sin(), self.center.y);
            points.push(point(x, y));
        }

        let polygon = LyonPolygon {
            points: points.as_slice(),
            closed: true,
        };

        b.add_polygon(polygon);
        b.build()
    }
}

/// A line segment described by its endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub Vec2, pub Vec2);

impl Geometry for Line {
    fn generate_path(&self) -> tess::path::Path {
        let mut b = Builder::new();
        b.add_polygon(LyonPolygon {
            points: &[self.0.to_point(), self.1.to_point()],
            closed: false,
        });
        b.build()
    }
}
