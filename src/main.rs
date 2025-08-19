mod spines_plugin;
mod physics_plugin;

use bevy::math::ops::sin;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};

use nalgebra::{Vector, Vector2};
use rand::Rng;
use crate::physics_plugin::{Gravitate, PhysicsPlugin, VerletObject};
use crate::spines_plugin::{Position, SplinePlugin};

struct OverlayColor;

impl OverlayColor {
    const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}
fn main() {
    let mut app = App::new();
    app.add_plugins((
            DefaultPlugins,
            SplinePlugin,
            PhysicsPlugin,
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
    commands.spawn(Camera2d);

    let circle = meshes.add(Circle::new(30.0));
    let mid_circle = meshes.add(Circle::new(5.0));
    let small_circle = meshes.add(Circle::new(1.0));
    let rec = meshes.add(Rectangle::new(1.0, 10.0));
    let color = Color::WHITE;
    let material = materials.add(color);

    // spline
    let mut splines:Vec<Entity> = Vec::with_capacity(100);

    for i in 0..2{

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

    for i in 0..50{
        // let x_rand: f32 = rng.gen_range(-10.0..10.0);
        let x_rand: f32 = 0.0;
        let x =  -1000.0 + i as f32 * 40.0 + x_rand;
        // let y: f32 = rng.gen_range(-5.0..5.0);
        let y: f32 = sin(x*0.01)*0.0 - 200.0;

        for j in 0..2{

            commands.spawn((Position(Vector2::new(x,y)),
                            crate::spines_plugin::Target(Vector2::new(x - j as f32 * 1000.0, y+(j as f32*300.0))),
                            crate::spines_plugin::Movable {default_position: Vector2::new(x, y)},
                            Transform::from_xyz(
                                x,
                                y,
                                0.0,
                            ),
                            Mesh2d(mid_circle.clone()),
                            MeshMaterial2d(material.clone()),
                            crate::spines_plugin::ControlPoint(splines[j]),

            ));


        }

    }


    for i in 0..2000 {

        for j in 0..2{

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

        for j in 0..2{

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

    commands.spawn((Position(Vector2::new(100.0, 300.0)),
                    Transform::from_xyz(
                        0.0,
                        0.0,
                        0.0,
                    ),
                    Mesh2d(circle.clone()),
                    MeshMaterial2d(material.clone()),
                    Gravitate(),
                    VerletObject{position_old: Vector2::new(100.0,300.0), acceleration: Vector2::new(0.0,0.0)}
    ));

}
