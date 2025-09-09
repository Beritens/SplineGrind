use bevy::app::{App, FixedUpdate, Plugin};
use bevy::prelude::{Component, Deref, Entity, Query, RelationshipTarget, Res, Time, Transform, Update, Without};
use crate::physics_plugin::PhySched;
use crate::spines_plugin::{ControlledBy, Position, Spline, VisualizedBy};

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, follow_object);
    }
}


#[derive(Component)]
#[relationship(relationship_target = Followed)]
pub struct Follower(pub Entity);

#[derive(Component, Deref)]
#[relationship_target(relationship = Follower)]
pub struct Followed(Vec<Entity>);

fn follow_object(
    time: Res<Time>,
    query: Query<(&Transform, &Followed)>,
    mut position_query: Query<&mut Transform, Without<Followed>>,
) {
    let delta = time.delta_secs();
    let max_distance = 400.0; // maximum lag allowed
    let catchup_speed = 2.0;  // smoothing factor (higher = snappier)

    for (target_transform, followed) in &query {
        for entity in followed.iter() {
            if let Ok(mut pos) = position_query.get_mut(entity) {
                let target = target_transform.translation;
                let mut current = pos.translation;

                // Smooth follow with exponential decay
                let alpha = 1.0 - (-catchup_speed * delta).exp();
                current = current.lerp(target, alpha);

                // Clamp max lag
                let diff = target - current;
                let dist = diff.length();
                if dist > max_distance {
                    current = target - diff.normalize() * max_distance;
                }

                pos.translation.x = current.x;
                pos.translation.y = current.y;
            }
        }
    }
}
