use std::f32::consts::PI;
use bevy::app::{App, FixedUpdate, Plugin};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::math::ops::atan2;
use bevy::math::Quat;
use bevy::prelude::{Component, IntoScheduleConfigs, Query, Update, With, Without, World};
use nalgebra::{Normed, Vector2};
use crate::spines_plugin::{ControlledBy, FollowMouse, OldPosition, Position, Spline, SplinePlugin, SplineSet};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(PhySched);
        app.add_systems(PhySched,((update_position, apply_gravity.before(update_position), collide.before(update_position)).after(SplineSet)));
        app.add_systems(FixedUpdate,run_my_schedule);
    }
}
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhySched;

fn run_my_schedule(world: &mut World) {
    // Run your schedule multiple times per frame:
    for _ in 0..8 {
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
fn cross2d(a: Vector2<f32>, b: Vector2<f32>) -> f32 {
    a.x * b.y - a.y * b.x
}


fn collide(
    mut query: Query<(&mut Position, &mut VerletObject), (With<VerletObject>)>,
    spline_query: Query<(&Spline, &ControlledBy)>,
    position_query: Query<(&Position, &OldPosition), Without<VerletObject>>,
){

    let line_widht = 5.0;
    // let dt = 0.1;

    let mut temp_buf: [Vector2<f32>; 4] = [Vector2::new(0.0, 0.0); 4];


    for (spline, controlled_by) in &spline_query {
        let control_points = controlled_by.as_slice();



        let positions_new_old: Vec<(Vector2<f32>, Vector2<f32>)> = control_points
            .iter()
            .filter_map(|e| position_query.get(*e).ok())
            .map(|(p, po)| (p.0, po.0))
            .collect();
        let positions = positions_new_old.iter().map(|e| e.0.clone()).collect();
        // let positions_old = positions_new_old.iter().map(|e| e.1.clone()).collect();


        for (mut pos, mut verlet) in &mut query {


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

            let cross = cross2d(normal, grad);
            // let cross_old = cross2d(normal_old, grad_old);

            // let angle = {
            //     let delta_x = normal.x * normal_old.x + normal.y * normal_old.y; // cos(Δθ) = dot(a,b)
            //     let delta_y = normal.y * normal_old.x - normal.x * normal_old.y; // sin(Δθ) = cross(a,b) in 2D
            //     delta_y.atan2(delta_x) // returns Δθ in [-π, π]
            // };

            let mut underground = false;

            if(cross >= 0.0) {
                underground= true;
                normal = normal * -1.0;
            }

            // if(angle.abs() > (PI/10.0)) {
            //     underground= true;
            //     normal = normal * -1.0;
            // }
            let overground = point + normal *( 30.0 + line_widht);


            let vel = pos.0 - verlet.position_old;
            let old_offset = ((verlet.position_old + vel - overground).transpose() * normal).x + 0.02;

            if old_offset < 0.0{
               verlet.position_old -= normal * old_offset;
            }

            if ((pos.0 - point).transpose() * normal).x < 30.0 + line_widht {

                pos. 0 = overground;

            }
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
