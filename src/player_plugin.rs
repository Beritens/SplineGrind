use std::collections::HashMap;
use std::f64::consts::PI;
use bevy::asset::ron::Map;
use bevy::math::ops::atan2;
use bevy::prelude::*;
use bevy::transform;
use bevy_asset_loader::prelude::*;
use nalgebra::{Normed, Vector2};
use crate::assets_plugin::{GameState, PlayerAssets};
use crate::controls_plugin::Follower;
use crate::physics_plugin::{Collider, Gravitate, SplineMemory, VerletObject};
use crate::spines_plugin::Position;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_player);
        app.add_systems(Update, anime_player);

    }
}

fn spawn_player(mut commands: Commands, mut player_assets: Res<PlayerAssets>) {

    let player = commands.spawn((Position(Vector2::new(100.0, 300.0)),
                                 Transform::from_scale(
                                     Vec3::splat(0.3)
                                 ),
                                 Sprite::from_atlas_image(
                                     player_assets.sprite.clone(),
                                     TextureAtlas {
                                         layout: player_assets.layout.clone(),
                                         index: 0,
                                     },
                                 ),
                                 SplineMemory{spline_intersections: HashMap::new()},
                                 Gravitate(),
                                 Collider::new(),
                                 VerletObject { position_old: Vector2::new(100.0, 300.0), acceleration: Vector2::new(0.0, 0.0) }
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

}

fn cross2d(a: Vector2<f32>, b: Vector2<f32>) -> f32 {
    a.x * b.y - a.y * b.x
}

fn anime_player(
    mut query: Query<(&Position, &VerletObject, &mut Sprite, &mut Transform, &Collider)>
){

    for (position, verlet_object, mut sprite ,mut transform, collider) in &mut query{

        if(!collider.collisions_old.is_empty()){
            let normal = collider.collisions_old[collider.collisions_old.len()-1].normal;
            let angle = atan2(normal.y, normal.x);
            let target = Quat::from_rotation_z(angle-PI as f32/2.0);

            transform.rotation = transform.rotation.slerp(target, 0.1);


        }

        let speed: Vector2<f32> = position.0 - verlet_object.position_old;

        let bevy_v = Vec3::new(0.0, 1.0, 0.0);

        // Rotate using Bevy
        let rotated = transform.rotation * bevy_v;

        // Convert back Bevy Vec3 -> nalgebra Vector3
        let normal: Vector2<f32> = Vector2::new(rotated.x, rotated.y);


        let hor_speed = speed - normal * (speed.transpose() * normal);

        let cross = -1.0 * cross2d(normal, hor_speed);
        if(collider.collisions_old.is_empty()){
            let angle = if(hor_speed.norm_squared() > 0.1) {atan2(speed.y, speed.x) + cross.signum() *  PI as f32/ 2.0} else {PI as f32/2.0};
            let target = Quat::from_rotation_z(angle-PI as f32/2.0);
            transform.rotation = transform.rotation.slerp(target, 0.1);
        }


        let val = hor_speed.norm_squared();
        if let Some(atlas) = &mut sprite.texture_atlas {

            if(val >  0.2){

                atlas.index = 1;
            }
            else if(val <0.1){

                atlas.index = 0;
            }
        }


        if(val > 0.2){
            transform.scale.x = transform.scale.x.abs() * cross.signum();
        }
    }

}
