// This example is messy but I'm too lazy to refactor it. Sorry...

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_canvas::{Canvas, DrawMode, Geometry, PathBuilder, StrokeOptions};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_canvas::CanvasPlugin)
        .add_startup_stage_after(
            StartupStage::Startup,
            "uwu:stage:camera_position",
            SystemStage::single(move_camera.system()),
        )
        .add_startup_system(setup.system())
        .add_system(shapes.system())
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "UwU".to_string(),
            ..Default::default()
        })
        .run();
}

struct Eye {
    center: Vec2,
    oification: f32,
}

impl Geometry for Eye {
    fn generate_path(&self) -> tess::path::Path {
        let mut b = PathBuilder::new();
        let arc_start_left = Vec2::new(self.center.x - 100.0, self.center.y);
        let arc_start_right = Vec2::new(self.center.x + 100.0, self.center.y);

        // lower eyelid
        b.move_to(arc_start_left);
        b.arc(self.center, Vec2::new(100.0, 100.0), PI, 0.0);

        // upper eyelid
        if self.oification < 0.5 {
            // eyelid is down
            b.move_to(arc_start_left);
            b.arc(
                self.center,
                Vec2::new(100.0, 100.0 * (1.0 - self.oification * 2.0)),
                PI,
                0.0,
            );
        } else {
            // eyelid is up
            b.move_to(arc_start_right);
            b.arc(
                self.center,
                Vec2::new(100.0, 100.0 * (self.oification - 0.5) * 2.0),
                PI,
                0.0,
            );
        }

        b.build()
    }
}

struct Mouth {
    wification: f32,
}

impl Geometry for Mouth {
    fn generate_path(&self) -> tess::path::Path {
        let y = -(60.0 + 140.0 * self.wification);
        let lip_sides_y = 10.0 + 30.0 * self.wification;
        let lip_center_y = -30.0 * self.wification;
        let mut b = PathBuilder::new();
        b.move_to(Vec2::new(-80.0, lip_sides_y));
        b.cubic_bezier_to(
            Vec2::new(-60.0, y),
            Vec2::new(-20.0, y),
            Vec2::new(0.0, lip_center_y),
        );
        b.cubic_bezier_to(
            Vec2::new(20.0, y),
            Vec2::new(60.0, y),
            Vec2::new(80.0, lip_sides_y),
        );

        b.build()
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn move_camera(mut query: Query<(&mut Transform,)>) {
    if let Some(mut camera_transform) = query.iter_mut().next() {
        camera_transform.0.translation.y += 100.0;
    }
}

fn shapes(mut canvas: ResMut<Canvas>, time: Res<Time>) {
    let t = time.time_since_startup().as_secs_f32();
    let owo_factor = (3.0 * t).cos() / 2.0 + 0.5;
    let color_a_factor = t.cos() / 2.0 + 0.5;
    let color_b_factor = (t - PI).cos() / 2.0 + 0.5;

    let left_eye = Eye {
        center: Vec2::new(-200.0, 200.0),
        oification: owo_factor,
    };
    let right_eye = Eye {
        center: Vec2::new(200.0, 200.0),
        oification: owo_factor,
    };
    let mouth = Mouth {
        wification: owo_factor,
    };

    let color_a = Color::rgb(color_a_factor, 1.0, 0.0);
    let color_b = Color::rgb(1.0, color_b_factor, 0.0);

    let options = StrokeOptions::default()
        .with_line_width(2.0 + 7.0 * owo_factor)
        .with_line_cap(tess::LineCap::Round)
        .with_line_join(tess::LineJoin::Round);
    canvas.draw(&left_eye, DrawMode::Stroke(options), color_a);
    canvas.draw(&right_eye, DrawMode::Stroke(options), color_a);
    canvas.draw(&mouth, DrawMode::Stroke(options), color_b);
}
