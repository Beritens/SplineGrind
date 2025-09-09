use std::collections::HashMap;
use std::f32::consts::PI;
use std::iter::Map;
use std::mem;
use bevy::app::{App, FixedUpdate, Plugin};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::math::ops::atan2;
use bevy::math::Quat;
use bevy::prelude::{Component, Entity, IntoScheduleConfigs, Query, SystemSet, Update, With, Without, World};
use nalgebra::{Normed, Vector2};
use crate::spines_plugin::{point_inside, ControlledBy, FollowMouse, HiddenControlledBy, OldPosition, Position, Spline, SplinePlugin, SplineSet};

pub struct PhysicsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PhySet;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PhySched);
        app.add_systems(FixedUpdate,((update_position, apply_gravity.before(update_position), collide.before(update_position),reset_collisions.before(collide)).in_set(PhySet).after(SplineSet)));
    }
}
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhySched;

fn run_my_schedule(world: &mut World) {
    // Run your schedule multiple times per frame:
    for _ in 0..1 {
        world.run_schedule(PhySched);
    }
}

#[derive(Component)]
struct Mass(f32);
#[derive(Component)]
pub struct VerletObject{
    pub position_old: Vector2<f32>,
    pub acceleration: Vector2<f32>,

}

pub struct SplineColliderInfo {
    pub intersections: i32,
    pub start_sector: i32,
    pub end_sector: i32,
}

#[derive(Component)]
pub struct SplineMemory{
    pub spline_intersections: HashMap<Entity, SplineColliderInfo>,

}


#[derive(Clone)]
pub struct Collision{
    pub other: Entity,
    pub point: Vector2<f32>,
    pub normal: Vector2<f32>,
}
#[derive(Component)]
pub struct Collider{
   pub collisions: Vec<Collision>,
    pub collisions_old: Vec<Collision>,

}
impl Collider {
    pub fn new() -> Self {
        Self { collisions: Vec::new(), collisions_old: Vec::new() }
    }

    pub fn add_collision(&mut self, c: Collision) {
        self.collisions.push(c.clone());
        self.collisions_old.push(c);
    }
}

#[derive(Component)]
pub struct Gravitate();
const gravity: Vector2<f32> = Vector2::<f32>::new(0.0, -10.0);

fn setup(){

}

fn apply_gravity(
   mut query: Query<(&mut VerletObject), With<Gravitate>>
){

    for (mut verlet_object) in &mut query {
        verlet_object.acceleration += gravity;
    }

}

fn reset_collisions(

    mut query: Query<(&mut Collider)>
){
    for (mut collider) in &mut query {
        collider.collisions_old = mem::take(&mut collider.collisions);
    }

}
fn update_position(


    mut query: Query<(&mut VerletObject, &mut Position)>
){

    let dt = 0.016;

    for (mut verlet_object, mut pos) in &mut query {

        let vel = pos.0 - verlet_object.position_old;

        verlet_object.position_old = pos.0;
        pos.0 = pos.0 + vel + verlet_object.acceleration * dt * dt;

        verlet_object.acceleration = Vector2::zeros();

    }
}


fn collide(
    mut query: Query<(&mut Position, &mut VerletObject, &mut Collider, &mut SplineMemory)>,
    spline_query: Query<(&Spline, &ControlledBy, &HiddenControlledBy, Entity)>,
    position_query: Query<(&Position), Without<VerletObject>>,
){

    let line_widht = 5.0;
    // let dt = 0.1;

    let mut temp_buf: [Vector2<f32>; 4] = [Vector2::new(0.0, 0.0); 4];


    for (spline, controlled_by, hidden_controlled_by, entity) in &spline_query {
        let control_points = controlled_by.as_slice();
        let hidden_control_points = hidden_controlled_by.as_slice();



        let positions: Vec<Vector2<f32>> = control_points
            .iter()
            .filter_map(|e| position_query.get(*e).ok())
            .map(|p| p.0)
            .collect();

        let hidden_positions: Vec<Vector2<f32>> = hidden_control_points
            .iter()
            .filter_map(|e| position_query.get(*e).ok())
            .map(|p| p.0)
            .collect();


        for (mut pos, mut verlet, mut collider, mut spline_memory) in &mut query {


            let t = crate::spines_plugin::get_nearest_spline_point(pos.0, &positions);

            let mut v:Vec<f32> = Vec::with_capacity(positions.len() + 4);
            v.extend(std::iter::repeat(0.0).take(4)); // first n zeros
            for i in 1..(positions.len() - 3 ) {
                v.push(i as f32);
            }
            v.extend(std::iter::repeat((positions.len() - 3) as f32).take(4)); // last n zeros


            let l = crate::spines_plugin::find_knot::<4>(t, &v);
            let point = crate::spines_plugin::de_boors::<4>(&positions, t, &v, &mut temp_buf, l);
            let grad = crate::spines_plugin::de_boors_derivative::<4>(&positions, t, &v, &mut temp_buf, l);



            // let t_old = crate::spines_plugin::get_nearest_spline_point(verlet.position_old, &positions_old);
            // let l = crate::spines_plugin::find_knot::<4>(t_old, &v);
            // let point_old = crate::spines_plugin::de_boors::<4>(&positions_old, t_old, &v, &mut temp_buf, l);
            // let grad_old = crate::spines_plugin::de_boors_derivative::<4>(&positions_old, t_old, &v, &mut temp_buf, l);

            let mut normal: Vector2<f32> = (pos.0 - point).normalize();

            // let normal_old: Vector2<f32> = (verlet.position_old - point_old).normalize();

            // let cross = cross2d(normal, grad);
            // let cross_old = cross2d(normal_old, grad_old);

            // let angle = {
            //     let delta_x = normal.x * normal_old.x + normal.y * normal_old.y; // cos(Δθ) = dot(a,b)
            //     let delta_y = normal.y * normal_old.x - normal.x * normal_old.y; // sin(Δθ) = cross(a,b) in 2D
            //     delta_y.atan2(delta_x) // returns Δθ in [-π, π]
            // };

            let mut underground = false;
            let (inside, count) = point_inside(
                pos.0,
                &positions,
                spline_memory
                    .spline_intersections
                    .get(&entity)
            );
            if(inside) {
                underground= true;
                normal = normal * -1.0;

            }
            else{
               spline_memory.spline_intersections.insert(entity, count);
            }

            // if(angle.abs() > (PI/10.0)) {
            //     underground= true;
            //     normal = normal * -1.0;
            // }
            let overground = point + normal *( 60.0 + line_widht);


            let vel = pos.0 - verlet.position_old;
            let old_offset = ((verlet.position_old + vel - overground).transpose() * normal).x + 0.02;

            if old_offset < 0.0{
               verlet.position_old -= normal * old_offset;
            }

            if ((pos.0 - point).transpose() * normal).x < 60.0 + line_widht {

                pos. 0 = overground;

                collider.collisions.push(Collision{other: entity, point: point, normal: normal});

            }
            // if(underground){
            //     println!("underground");
            // }
            // if(underground){
            //
            //     println!("______________");
            //     println!("{}", normal);
            //     println!("{}", normal_old);
            //     println!("{}", angle);
            // }
            // if(underground){
            //     pos.0.y = pos.0.y + 500.0;
            //     verlet.position_old.y = pos.0.y;
            //     verlet.position_old.x = pos.0.x;
            // }

        }

    }
}
