//! # Unofficial Bevy canvas API
//!
//! This crate allows Bevy users to arbitrarily draw 2D shapes without spawning
//! any entity. You will need to spawn a 2D camera.
//!
//! The main goal of this project is to help users draw any geometric shape in
//! the most ergonomical possible way, without the need to spawn entities,
//! unlike [`bevy_prototype_lyon`](https://github.com/Nilirad/bevy_prototype_lyon).
//!
//! ## Setup
//!
//! Don't forget to add the plugin and a 2d camera. Here's an example:
//! ```
//! use bevy::prelude::*;
//! use bevy_canvas::CanvasPlugin;
//!
//! fn main() {
//!     App::build()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(bevy_canvas::CanvasPlugin)
//!         .add_startup_system(setup.system());
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(OrthographicCameraBundle::new_2d());
//! }
//! ```
//!
//! ## Usage
//!
//! For the common usage guide, see the [plugin documentation](CanvasPlugin).

use bevy::app::{AppBuilder, Plugin};

mod canvas;
pub mod common_shapes;
mod path;
mod render;

pub use canvas::{Canvas, DrawMode, Geometry};
pub use path::PathBuilder;
pub use tess::{path::Path, FillOptions, FillRule, LineCap, LineJoin, Orientation, StrokeOptions};

/// A Bevy `Plugin` that gives the ability to directly draw 2D shapes from a
/// system.
///
/// ## Usage
///
/// This plugin provides a [`Canvas`] `Resource`, a rendering context. This
/// context exposes a [`draw`](Canvas::draw) method that can accept any struct
/// that implements the [`Geometry`] trait.
///
/// ### Example
/// ```
/// use bevy::prelude::*;
/// use bevy_canvas::{
///     common_shapes::{RegularPolygon, RegularPolygonFeature},
///     Canvas, CanvasPlugin, DrawMode, StrokeOptions,
/// };
///
/// // This is a Bevy system that runs every frame
/// fn my_system(mut canvas: ResMut<Canvas>) {
///     // RegularPolygon implements the Geometry trait, so it can be accepted
///     // by canvas.draw method.
///     let my_shape = RegularPolygon {
///         center: Vec2::ZERO,
///         sides: 6,
///         feature: RegularPolygonFeature::Radius(100.0),
///     };
///
///     canvas.draw(&my_shape, DrawMode::fill_simple(), Color::MIDNIGHT_BLUE);
/// }
/// ```
#[derive(Default)]
pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Canvas::default());

        render::setup_canvas_node(app.world_mut());
    }
}
