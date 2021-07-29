use bevy::prelude::*;
use bevy_canvas::{
    common_shapes::{RegularPolygon, RegularPolygonFeature},
    Canvas, DrawMode, FillOptions, LineCap, StrokeOptions,
};

const COLORS: [Color; 7] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::GOLD,
    Color::SEA_GREEN,
    Color::SALMON,
    Color::MIDNIGHT_BLUE,
];

struct MyShape {
    draw_mode: DrawMode,
    shape: RegularPolygon,
    color_index: usize,
}

fn main() {
    println!(include_str!("showoff_usage.txt"));

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_canvas::CanvasPlugin)
        .add_startup_system(setup_system)
        .add_system(handle_input_system)
        .add_system(draw_shape_system)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Shapes".to_string(),
            ..Default::default()
        })
        .insert_resource(MyShape {
            draw_mode: DrawMode::Stroke(
                StrokeOptions::default()
                    .with_line_width(5.0)
                    .with_line_join(bevy_canvas::LineJoin::Round)
                    .with_line_cap(LineCap::Round),
            ),
            shape: RegularPolygon {
                center: Vec2::ZERO,
                sides: 6, // hexagons are bestagons.
                feature: RegularPolygonFeature::Radius(100.0),
            },
            color_index: 0,
        })
        .run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn draw_shape_system(mut canvas: ResMut<Canvas>, my_shape: Res<MyShape>) {
    canvas.draw(
        &my_shape.shape,
        my_shape.draw_mode,
        COLORS[my_shape.color_index],
    );
}

fn handle_input_system(keys: Res<Input<KeyCode>>, mut my_shape: ResMut<MyShape>) {
    if let RegularPolygonFeature::Radius(ref mut r) = my_shape.shape.feature {
        if keys.just_pressed(KeyCode::A) {
            *r = (*r - 10.0).clamp(20.0, 200.0);
        }
        if keys.just_pressed(KeyCode::S) {
            *r = (*r + 10.0).clamp(20.0, 200.0);
        }
    }

    match my_shape.draw_mode {
        DrawMode::Fill(ref mut _options) => {}
        DrawMode::Stroke(ref mut options) => {
            if keys.just_pressed(KeyCode::Q) {
                options.line_width = (options.line_width - 1.0).clamp(1.0, 10.0);
            }
            if keys.just_pressed(KeyCode::W) {
                options.line_width = (options.line_width + 1.0).clamp(1.0, 10.0);
            }
        }
    }

    if keys.just_pressed(KeyCode::F) {
        if let DrawMode::Fill(_) = my_shape.draw_mode {
            my_shape.draw_mode = DrawMode::Stroke(
                StrokeOptions::default()
                    .with_line_width(5.0)
                    .with_line_join(bevy_canvas::LineJoin::Round)
                    .with_line_cap(LineCap::Round),
            );
        } else {
            my_shape.draw_mode = DrawMode::Fill(FillOptions::default());
        }
    }

    if keys.just_pressed(KeyCode::C) {
        my_shape.color_index = (my_shape.color_index + 1) % COLORS.len();
    }

    if keys.just_pressed(KeyCode::Z) {
        my_shape.shape.sides = (my_shape.shape.sides - 1).clamp(3, 12);
    }
    if keys.just_pressed(KeyCode::X) {
        my_shape.shape.sides = (my_shape.shape.sides + 1).clamp(3, 12);
    }
}
