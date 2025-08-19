use bevy::app::{App, FixedUpdate, Plugin};
use bevy::prelude::{Component, Deref, Entity, Query, RelationshipTarget, Transform, Update, Without};
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
    query: Query<(&Transform, &Followed)>,
    mut position_query: Query<&mut Transform, Without<Followed>>,
){

    for (transform, followed) in &query {


        for entity in followed.iter() {
            if let Ok(mut pos) = position_query.get_mut(entity) {
                pos.translation.x = pos.translation.x * 0.99 + transform.translation.x * 0.01;
                pos.translation.y = pos.translation.y * 0.99+ transform.translation.y * 0.01;
            }
        }
    }

}
