use bevy::math::ops::{atan2, sin};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};
use bevy::render::render_resource::encase::private::RuntimeSizedArray;
use nalgebra::{Vector, Vector2, Vector3};
use rand::Rng;
use roots::{find_root_newton_raphson, SimpleConvergency};
use crate::physics_plugin::{PhySched, SplineColliderInfo, SplineMemory, VerletObject};

pub struct SplinePlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SplineSet;
impl Plugin for SplinePlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(Update, (render_spline, render_gradient, update_position ));
        app.add_systems(FixedUpdate, (update_old_pos, move_points, go_to_target,  push, ).chain().in_set(SplineSet));
        app.add_systems(PostUpdate, (follow_mouse.after(TransformSystem::TransformPropagate)));
    }
}

#[derive(Component, Debug, Clone)]
pub struct Position(pub Vector2<f32>);

#[derive(Component, Debug, Clone)]
pub struct OldPosition(pub Vector2<f32>);

#[derive(Component, Debug, Clone)]
pub struct Moving();

#[derive(Component)]
pub struct FollowMouse();

#[derive(Component)]
pub struct Target(pub Vector2<f32>);

#[derive(Component)]
pub struct Movable{
    pub default_position: Vector2<f32>
}

#[derive(Component)]
pub struct Pusher();


#[derive(Component)]
pub struct Spline();

#[derive(Component)]
#[relationship(relationship_target = ControlledBy)]
pub struct ControlPoint(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = ControlPoint)]
pub struct ControlledBy(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = HiddenControlledBy)]
pub struct HiddenControlPoint(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = HiddenControlPoint)]
pub struct HiddenControlledBy(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = VisualizedBy)]
pub struct Visualization(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = Visualization)]
pub struct VisualizedBy(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = GradientVisualizedBy)]
pub struct VisualizationGradient(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = VisualizationGradient)]
pub struct GradientVisualizedBy(Vec<Entity>);



fn update_old_pos(

    mut query: Query<(&Position, &mut OldPosition)>
){

    for (new, mut old) in &mut query{
        old.0 = new.0;
    }
}


fn push(
    mut pushed_query: Query<(&mut Position, &Target, &Movable), Without<Pusher>>,
    pusher_query: Query<(&Position, &Pusher)>
){
    for (push_pos, pusher) in &pusher_query{

        for (mut pushed_pos, mut target, mov) in &mut pushed_query{
            let norm = (pushed_pos.0 - push_pos.0).norm();
            if(norm <= 190.0){
                let force = 0.5 * (1.0 - norm/190.0);
                pushed_pos.0 =pushed_pos.0 * (1.0 - force) +  (push_pos.0 + (pushed_pos.0 - push_pos.0).normalize() * 190.0) * (force);
                // pushed_pos.0 =(push_pos.0 + (pushed_pos.0 - push_pos.0).normalize() * 450.0);
            }
            // else{
            //     target.0 = mov.default_position
            // }
        }

    }
}

fn go_to_target(
    mut target_query: Query<(&mut Position, &Target)>,
){

    for (mut pos, target) in &mut target_query{
        pos.0 = pos.0 * 0.98 + target.0 * 0.02;
        // pos.0 =  target.0 ;
    }

}
fn update_position(
    mut query: Query<(&Position, &mut Transform)>
){
    for (pos, mut transform) in query.iter_mut() {
        transform.translation.x = pos.0.x;
        transform.translation.y = pos.0.y;
    }
}

fn follow_mouse(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut follower_query: Query<(&mut Position, &mut FollowMouse)>,
    window: Query<&Window>,
) {
    let (camera, camera_transform) = *camera_query;
    let Ok(window) = window.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (mut pos, mut follower_mouse) in follower_query.iter_mut() {
        pos.0.x = world_pos.x;
        pos.0.y = world_pos.y;
    }

}

fn move_points(
    time: Res<Time>,
    mut position_query: Query<(&mut Position, &mut Transform, &Moving)>,
){
    for (mut pos,  mut trans, mov) in &mut position_query{
        pos.0.y += 0.2 * sin(time.elapsed_secs()+ pos.0.x);
        trans.translation.y = pos.0.y;
    }

}

fn de_casteljau(mut control_points: Vec<Vector2<f32>>, t: f32) -> Vector2<f32> {
    let mut r: i32 = 1;
    while r < control_points.len() as i32 {
        let n = control_points.len() as i32 - r;
        for i in 0..n {
            control_points[i as usize] = t * control_points[i as usize + 1] + (1.0 -t) * control_points[i as usize];
        }
        r += 1;
    }

    return control_points[0];
}

#[inline(always)]
pub fn de_boors<const LEN: usize>(control_points: &Vec<Vector2<f32>>, t: f32, t_vec: &Vec<f32>,temp: &mut [Vector2<f32>; LEN], l: usize) -> Vector2<f32> {
    //assume uniform knots
    let N: usize = LEN - 1;

    let t_scale = t * t_vec[t_vec.len() - 1];


    let base = l - N;

    temp.copy_from_slice(&control_points[base..=l]);


    for r in 1..=N {
        let ir_base = r + base;        // i+r+(l-n) base
        let in1_base = N + 1 + base;   // i+n+1+(l-n) base

        for i in 0..=N - r {
            let t_thing = (t_scale - t_vec[i + ir_base])
                / (t_vec[i + in1_base] - t_vec[i + ir_base]);
            temp[i] = t_thing * temp[i + 1] + (1.0 - t_thing) * temp[i];
        }
    }

    return temp[0];
}
fn cubic_bspline(u: f32, ti: f32, ti1: f32, ti2: f32, ti3: f32, ti4: f32, ti5: f32, ti6:f32,
                 di: Vector2<f32>, di1: Vector2<f32>, di2: Vector2<f32>, di3: Vector2<f32>) -> Vector2<f32> {


    return  (
        ((ti1 - ti4)*(ti2 - ti4)*(ti3 - u)*((ti2 - ti5)*(ti3 - u)*(di2*(-ti6 + u) + di3*(ti3 - u)) + (ti3 - ti6)*(-ti5 + u)*(di1*(-ti5 + u) + di2*(ti2 - u))) + (ti3 - ti5)*(ti3 - ti6)*(-ti4 + u)*((ti1 - ti4)*(ti2 - u)*(di1*(-ti5 + u) + di2*(ti2 - u)) + (ti2 - ti5)*(-ti4 + u)*(di*(-ti4 + u) + di1*(ti1 - u))))/((ti1 - ti4)*(ti2 - ti4)*(ti2 - ti5)*(ti3 - ti4)*(ti3 - ti5)*(ti3 - ti6))
    );

}

#[inline(always)]
pub fn not_de_boors<const LEN: usize>(control_points: &Vec<Vector2<f32>>, t: f32, t_vec: &Vec<f32>,temp: &mut [Vector2<f32>; LEN], l: usize) -> Vector2<f32> {
    //assume uniform knots
    let N: usize = LEN - 1;
    let base = l - N;


    let u = t * t_vec[t_vec.len() - 1];


    let ti = t_vec[base];
    let ti1 = t_vec[base+1];
    let ti2 = t_vec[base+2];
    let ti3 = t_vec[base+3];
    let ti4 = t_vec[base+4];
    let ti5 = t_vec[base+5];
    let ti6 = t_vec[base+6];


    let di = control_points[base];
    let di1 = control_points[base+1];
    let di2 = control_points[base+2];
    let di3 = control_points[base+3];


    return cubic_bspline(u, ti, ti1, ti2, ti3, ti4, ti5, ti6, di, di1, di2, di3);
}

#[inline(always)]
pub fn find_knot<const LEN: usize>(t: f32, t_vec: &Vec<f32>) -> usize {
    //assume uniform knots

    let t_scale = t * t_vec[t_vec.len() - 1];
    let l = t_vec
        .partition_point(|&knot| knot <= t_scale)
        .saturating_sub(1).max(LEN-1). min(t_vec.len() - LEN - 1);
    return l;
}

#[inline(always)] pub fn de_boors_derivative<const LEN: usize>(
    control_points: &Vec<Vector2<f32>>,
    t: f32,
    t_vec: &Vec<f32>,
    temp: &mut [Vector2<f32>; LEN],
    l: usize) -> Vector2<f32> {

    let p = LEN -1;
    let N: usize = LEN - 2;
    let t_scale = t * t_vec[t_vec.len() - 1];
    let base = l - p;
    for i in 0 ..=N{
        let dt = t_vec[base + i + p + 1] - t_vec[base + i + 1];
        temp[i] = (control_points[i + base + 1] - control_points[i + base]) * (p as f32/ dt)
    }

    for r in 1..=N {

        let ir_base = r + base + 1;
        let in1_base = N + 1 + base + 1; // i+n+1+(l-n) base

        for i in 0..=N - r {
            let t_thing = (t_scale - t_vec[i + ir_base]) / (t_vec[i + in1_base] - t_vec[i + ir_base]);
            temp[i] = t_thing * temp[i + 1] + (1.0 - t_thing) * temp[i];
        }
    }
    return temp[0];
}

#[inline(always)]
fn de_boors_second_derivative<const LEN: usize>(control_points: &Vec<Vector2<f32>>, t: f32, t_vec: &Vec<f32>,temp: &mut [Vector2<f32>; LEN], l: usize) -> Vector2<f32> {

    let p = LEN - 1;
    let n1 = LEN - 3; // degree reduced by 2
    let t_scale = t * t_vec[t_vec.len() - 1];
    let base = l - p;


    for i in 0..=n1 {
        let dt1 = t_vec[base + i + p  + 1] - t_vec[base + i + 2];
        let dt0 = t_vec[base + i + p + 1] - t_vec[base + i + 1];
        let cprime_i = (control_points[i + base + 1] - control_points[i + base]) * (p as f32 / dt0);
        let cprime_next = (control_points[i + base + 2] - control_points[i + base + 1]) * (p as f32 / dt0);
        temp[i] = (cprime_next - cprime_i) * ((p - 1) as f32 / dt1);
    }

    for r in 1..=n1 {
        let ir_base = r + base + 2;        // i+r+(l-n) base
        let in1_base = n1 + 1 + base + 2;   // i+n+1+(l-n) base

        for i in 0..=n1 - r {
            let t_thing = (t_scale - t_vec[i + ir_base])
                / (t_vec[i + in1_base] - t_vec[i + ir_base]);
            temp[i] = t_thing * temp[i + 1] + (1.0 - t_thing) * temp[i];
        }
    }

    return temp[0];
}

fn render_spline(query: Query<(&Spline, &ControlledBy, &VisualizedBy)>,
                 position_query: Query<&Position>,
                 mut transforms: Query<&mut Transform>,
){

    let dim = 3;
    let mut temp_buf: [Vector2<f32>; 4] = [Vector2::new(0.0, 0.0); 4];
    for (spline, controlled_by, visualized_by) in &query {
        let control_points = controlled_by.as_slice();

        let positions: Vec<Vector2<f32>> = control_points
            .iter()
            .filter_map(|e| position_query.get(*e).ok())
            .map(|p| p.0)
            .collect();

        let visual_points = visualized_by.as_slice();

        let n = visual_points.len();
        let step = 1.0 / n as f32;
        let mut t = 0.0;


        let mut v:Vec<f32> = Vec::with_capacity(positions.len() + dim + 1);
        v.extend(std::iter::repeat(0.0).take(dim + 1)); // first n zeros
        for i in 1..(positions.len() - dim ) {
            v.push(i as f32);
        }
        v.extend(std::iter::repeat((positions.len() - dim) as f32).take(dim + 1)); // last n zeros





        for i in 0..n {
            // let point = de_casteljau(positions.clone(), t);

            let l = find_knot::<4>(t, &v);
            let point = de_boors::<4>(&positions, t, &v, &mut temp_buf, l);
            if let Ok(mut transform) = transforms.get_mut(visual_points[i as usize]) {
                transform.translation.x = point.x;
                transform.translation.y = point.y;
            }
            t = t+step;


        }

    }
}

fn render_intersections(query: Query<(&Spline, &ControlledBy, &GradientVisualizedBy)>,
                   position_query: Query<&Position, Without<VerletObject>>,
                   mut object_query: Query<(&mut Position), (With<VerletObject>)>,
                   mut transforms: Query<&mut Transform>,
) {

    let dim = 3;
    let mut temp_buf: [Vector2<f32>; 4] = [Vector2::new(0.0, 0.0); 4];

    let Ok(object) = object_query.single() else {
    return;
    };
    for (spline, controlled_by, visualized_by) in &query {
        let control_points = controlled_by.as_slice();

        let positions: Vec<Vector2<f32>> = control_points
        .iter()
        .filter_map(|e| position_query.get(*e).ok())
        .map(|p| p.0)
        .collect();

        let visual_points = visualized_by.as_slice();

        let n = visual_points.len();
        let step = 1.0 / n as f32;


        let mut v:Vec<f32> = Vec::with_capacity(positions.len() + dim + 1);
        v.extend(std::iter::repeat(0.0).take(dim + 1)); // first n zeros
        for i in 1..(positions.len() - dim ) {
        v.push(i as f32);
        }
        v.extend(std::iter::repeat((positions.len() - dim) as f32).take(dim + 1)); // last n zeros



        let mut t = get_nearest_spline_point(object.0, &positions);


        for i in 0..n {
        let l = find_knot::<4>(t, &v);
        let point = de_boors::<4>(&positions, t, &v, &mut temp_buf, l);
        let grad = de_boors_derivative::<4>(&positions, t, &v, &mut temp_buf, l);
        // let dist_grad = (2.0 * (point - mouse.0).transpose() * grad);
        if let Ok(mut transform) = transforms.get_mut(visual_points[i as usize]) {
        transform.translation.x = point.x;
        transform.translation.y = point.y;
        // transform.scale = Vec3::splat(1.0) * (dist_grad.x * 0.001);

        let angle = atan2(grad.y, grad.x);
        transform.rotation = Quat::from_rotation_z(angle);
        }
        // t = t+step;


        }

    }
}

fn render_gradient(query: Query<(&Spline, &ControlledBy, &GradientVisualizedBy)>,
                 position_query: Query<&Position, Without<VerletObject>>,
                   mut object_query: Query<(&mut Position), (With<VerletObject>)>,
                 mut transforms: Query<&mut Transform>,
){

    let dim = 3;
    let mut temp_buf: [Vector2<f32>; 4] = [Vector2::new(0.0, 0.0); 4];

    let Ok(object) = object_query.single() else {
        return;
    };
    for (spline, controlled_by, visualized_by) in &query {
        let control_points = controlled_by.as_slice();

        let positions: Vec<Vector2<f32>> = control_points
            .iter()
            .filter_map(|e| position_query.get(*e).ok())
            .map(|p| p.0)
            .collect();

        let visual_points = visualized_by.as_slice();

        let n = visual_points.len();
        let step = 1.0 / n as f32;


        let mut v:Vec<f32> = Vec::with_capacity(positions.len() + dim + 1);
        v.extend(std::iter::repeat(0.0).take(dim + 1)); // first n zeros
        for i in 1..(positions.len() - dim ) {
            v.push(i as f32);
        }
        v.extend(std::iter::repeat((positions.len() - dim) as f32).take(dim + 1)); // last n zeros



        let mut t = get_nearest_spline_point(object.0, &positions);


        for i in 0..n {
            let l = find_knot::<4>(t, &v);
            let point = de_boors::<4>(&positions, t, &v, &mut temp_buf, l);
            let grad = de_boors_derivative::<4>(&positions, t, &v, &mut temp_buf, l);
            // let dist_grad = (2.0 * (point - mouse.0).transpose() * grad);
            if let Ok(mut transform) = transforms.get_mut(visual_points[i as usize]) {
                transform.translation.x = point.x;
                transform.translation.y = point.y;
                // transform.scale = Vec3::splat(1.0) * (dist_grad.x * 0.001);

                let angle = atan2(grad.y, grad.x);
                transform.rotation = Quat::from_rotation_z(angle);
            }
            // t = t+step;


        }
        //
        // let mut t = get_nearest_spline_point(object.0, &positions);
        // let mut t = 0.0;
        //
        //
        // for i in 0..n {
        //     let l = find_knot::<4>(t, &v);
        //
        //     let f = de_boors::<4>(&positions, t, &v, &mut temp_buf, l);
        //     let _df = de_boors_derivative::<4>(&positions, t, &v, &mut temp_buf, l);
        //     let cached_ddf = de_boors_second_derivative::<4>(&positions, t, &v, &mut temp_buf, l);
        //
        //     // compute gradient and hessian (1D Newton)
        //     let f_val = (f-object.0).norm_squared();
        //     let df_val = (2.0 * (f - object.0).transpose() * _df).x;
        //     let ddf_val = (2.0 * (_df.transpose() * _df + (f - object.0).transpose() * cached_ddf)).x;
        //     // let dist_grad = (2.0 * (point - mouse.0).transpose() * grad);
        //     if let Ok(mut transform) = transforms.get_mut(visual_points[i as usize]) {
        //         transform.translation.x = f.x;
        //         transform.translation.y = f.y;
        //         // transform.scale = Vec3::splat(1.0) * (dist_grad.x * 0.001);
        //
        //         let angle = atan2(_df.y, _df.x);
        //         transform.rotation = Quat::from_rotation_z(angle);
        //     }
        // t = t+step;
        //
        //
        // }

    }
}

pub fn get_nearest_spline_point(
    point: Vector2<f32>,
    positions: &Vec<Vector2<f32>>
) -> f32{
    let dim = 3;
    let mut temp1: [Vector2<f32>; 4] = [Vector2::new(0.0, 0.0); 4];
    let mut temp2 = [Vector2::zeros(); 4];


        let mut v:Vec<f32> = Vec::with_capacity(positions.len() + dim + 1);
        v.extend(std::iter::repeat(0.0).take(dim + 1)); // first n zeros
        for i in 1..(positions.len() - dim ) {
            v.push(i as f32);
        }
        v.extend(std::iter::repeat((positions.len() - dim) as f32).take(dim + 1)); // last n zeros



   let mut l = 0usize;
let mut f = Vector2::zeros();
let mut _df = Vector2::zeros();
let mut cached_ddf = Vector2::zeros();

let mut t :f32= 0.5; // initial guess
let tol = 1e-10;
let max_iter = 10;
    let mut rng = rand::thread_rng();

let mut min_dist = f32::INFINITY;
let mut min_t : f32 = 0.5;
let scale = 1.0 /v[v.len() - 1];
for i in 0..positions.len() {

    // t = (1.0 / 200.0) * i as f32;
    // l = find_knot::<4>(t, &v);
    // f = de_boors::<4>(&positions, t, &v, &mut temp1, l);
    let dist = (positions[i]-point).norm_squared();

    if(dist < min_dist){
        min_dist = dist;
        min_t = v[i + 3] * scale;
    }


}
    let con_t = min_t;

    let con_scale = 1.0 / positions.len() as f32;
    min_dist = f32::INFINITY;

for i in (-20i32)..=20 {

    t = con_t + i as f32 * con_scale / 10.0;
    l = find_knot::<4>(t, &v);
    f = de_boors::<4>(&positions, t, &v, &mut temp1, l);

    let dist = ( f - point).norm_squared();

    if(dist < min_dist){
        min_dist = dist;
        min_t = t;
    }


}
    t = min_t;

for _ in 0..max_iter {
    t = t.min(1.0).max(0.0);
    // compute everything once per t
    l = find_knot::<4>(t, &v);

    f = de_boors::<4>(&positions, t, &v, &mut temp1, l);
    _df = de_boors_derivative::<4>(&positions, t, &v, &mut temp1, l);
    cached_ddf = de_boors_second_derivative::<4>(&positions, t, &v, &mut temp2, l);

    // compute gradient and hessian (1D Newton)
    let df_val = (2.0 * (f - point).transpose() * _df).x;
    let ddf_val = (2.0 * (_df.transpose() * _df + (f - point).transpose() * cached_ddf)).x;
    if ddf_val.abs() < 1e-12 {
        break; // avoid division by zero
    }

    let step = df_val / ddf_val;
    t -= step / v[v.len() - 1];

    if step.abs() < tol {
        if((f - point).norm() > 100.0){
            let t: f32 = rng.gen_range(0.0..1.0);
            continue;
        }
        break;
    }
}
    // fallback if something goes wrong
    if (f-point).norm_squared() > min_dist {
        t = min_t;
    }

    return t.min(1.0).max(0.0);;



}

fn goes_through_line(
    p1: Vector2<f32>,
    p2: Vector2<f32>,
    point: Vector2<f32>,
) -> bool {

    let (bigger, smaller) = if p1.x > p2.x {
        (p1, p2)
    } else {
        (p2, p1)
    };
    if(smaller.x == bigger.x){
        return false;
    }
    let f = (point.x - smaller.x)/ (bigger.x - smaller.x);

    return (((1.0 - f)* smaller.y + f * bigger.y) > point.y) && f>0.0 && f<= 1.0;

}

pub fn point_inside(
    point: Vector2<f32>,
    positions: &Vec<Vector2<f32>>,
    spline_info: Option<&SplineColliderInfo>
) -> (bool, SplineColliderInfo) {

    let mut last: &Vector2<f32> = &positions[0];
    let mut count = 0;

    for pos in &positions[1..] {
       count += goes_through_line(*last, *pos, point) as i32;
        last = pos;
    }

    let start_x = positions[0].x < point.x;
    let start_y = positions[0].y < point.y;
    let end_x = positions[positions.len()-1].x < point.x;
    let end_y = positions[positions.len()-1].y < point.y;

    let splinco = SplineColliderInfo{intersections: count, start_sector: start_x as i32 + start_y as i32 * 2, end_sector: end_x as i32 + end_y as i32 * 2 };


    // for pos in hidden {
    //     count += goes_through_line(*last, *pos, point) as i32;
    //     last = pos;
    // }


    // count += goes_through_line(*last, positions[0], point) as i32;




    if let Some(spl) = spline_info{
        let mut parity_modifier = 0;
        if(spl.start_sector == 0 && splinco.start_sector == 1 || spl.start_sector == 1 && splinco.start_sector == 0){
           parity_modifier += 1;
        }

        if(spl.end_sector == 0 && splinco.end_sector == 1 || spl.end_sector == 1 && splinco.end_sector == 0){
            parity_modifier += 1;
        }

        // println!("{}, {}", spl.start_sector, splinco.start_sector);
        // println!("{}, {}", spl.end_sector, splinco.end_sector);

        let inters_old = spl.intersections;
        let inters_new = splinco.intersections;

        return (inters_old%2 != (parity_modifier + inters_new)%2, splinco);
    }
    return (false, splinco);

}


