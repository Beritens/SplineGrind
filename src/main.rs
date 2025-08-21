mod spines_plugin;
mod physics_plugin;
mod controls_plugin;

use bevy::math::ops::sin;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};

use nalgebra::{Vector, Vector2};
use rand::Rng;
use crate::controls_plugin::{ControlsPlugin, Follower};
use crate::physics_plugin::{Gravitate, PhysicsPlugin, VerletObject};
use crate::spines_plugin::{initBezierControlPoints, OldPosition, Position, SplinePlugin};

struct OverlayColor;

impl OverlayColor {
    const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}
fn main() {
    let mut app = App::new();
    app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // provide the ID selector string here
                    canvas: Some("#bevy".into()),
                    // ... any other window properties ...
                    ..default()
                }),
                ..default()
            }),
            SplinePlugin,
            PhysicsPlugin,
            ControlsPlugin,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 12.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: OverlayColor::GREEN,
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
        ));
    app.insert_resource(Time::<Fixed>::from_seconds(0.01666666));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {

    let circle = meshes.add(Circle::new(30.0));
    let color = Color::WHITE;
    let material = materials.add(color);
    let player = commands.spawn((Position(Vector2::new(100.0, 300.0)),
                    Transform::from_xyz(
                        0.0,
                        0.0,
                        0.0,
                    ),
                    Mesh2d(circle.clone()),
                    MeshMaterial2d(material.clone()),
                    Gravitate(),
                    VerletObject{position_old: Vector2::new(100.0,300.0), acceleration: Vector2::new(0.0,0.0)}
    )).id();
    let cameraWidth = 2400.0;
    commands.spawn((Camera2d,
                    Projection::from(OrthographicProjection {
                        scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal {
                            viewport_width: cameraWidth
                        },
                        ..OrthographicProjection::default_2d()
                    }),
                    Follower(player)));

    let mid_circle = meshes.add(Circle::new(5.0));
    let small_circle = meshes.add(Circle::new(5.0));
    let rec = meshes.add(Rectangle::new(2.0, 15.0));

    // spline
    let mut splines:Vec<Entity> = Vec::with_capacity(100);

    for i in 0..1{

        splines.push(commands.spawn((crate::spines_plugin::Spline())).id());
    }
    let mut rng = rand::thread_rng();

    commands.spawn((Position(Vector2::new(0.0, 1000.0)),
                    crate::spines_plugin::Pusher(),
                    crate::spines_plugin::FollowMouse(),
                    // Transform::from_xyz(
                    //     0.0,
                    //     0.0,
                    //     0.0,
                    // ),
                    // Mesh2d(circle.clone()),
                    // MeshMaterial2d(material.clone()),
    ));

    for j in 0..1{
        initBezierControlPoints(&mut commands, 500, splines[j], &mid_circle, &material);
        for i in 0..500{
        // let x_rand: f32 = rng.gen_range(-10.0..10.0);
            let x_rand: f32 = 0.0;
            let x =  -1000.0 + i as f32 * 40.0 + x_rand;
            // let y: f32 = rng.gen_range(-5.0..5.0);
            let y: f32 = sin(x*0.01)*x * 0.01 + sin(x*0.0085)*x * 0.005 +  sin(x*0.0185)*x * 0.0076 - 200.0;
            // let y: f32 = 0.3*x*x * 0.01 - 200.0;

            let _x = x - j as f32 * 1000.0;
            let _y = y+(j as f32*300.0);

            commands.spawn((Position(Vector2::new(x,y)),
                            crate::spines_plugin::Target(Vector2::new(_x, _y)),
                            OldPosition(Vector2::new(_x, _y)),
                            crate::spines_plugin::Movable {default_position: Vector2::new(x, y)},
                            // Transform::from_xyz(
                            //     x,
                            //     y,
                            //     0.0,
                            // ),
                            // Mesh2d(mid_circle.clone()),
                            // MeshMaterial2d(material.clone()),
                            crate::spines_plugin::ControlPoint(splines[j]),

            ));


        }

    }


    for i in 0..5000 {

        for j in 0..1{

            commands.spawn((Position(Vector2::new(5.0 * i as f32,0.0)),
                            Transform::from_xyz(
                                0.0,
                                0.0,
                                0.0,
                            ),
                            Mesh2d(small_circle.clone()),
                            MeshMaterial2d(material.clone()),
                            crate::spines_plugin::Visualization(splines[j])
            ));


        }


    }

    for i in 0..1 {

        for j in 0..1{

            commands.spawn((Position(Vector2::new(5.0 * i as f32,0.0)),
                            Transform::from_xyz(
                                0.0,
                                0.0,
                                0.0,
                            ),
                            Mesh2d(rec.clone()),
                            MeshMaterial2d(material.clone()),
                            crate::spines_plugin::VisualizationGradient(splines[j])
            ));


        }


    }


}
