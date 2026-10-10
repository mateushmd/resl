use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LinearVelocity, RigidBody},
};
use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{component::Component, query::With, system::Single},
    transform::components::Transform,
};

use crate::input_system::CharacterIntent;

#[derive(Component, Default)]
#[require(CharacterIntent, Collider, RigidBody, Transform)]
pub struct CharacterController;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(FixedUpdate, character_controller);
    }
}

fn character_controller(
    query: Single<(&CharacterIntent, &mut LinearVelocity), With<CharacterController>>,
) {
    let (intent, mut player_velocity) = query.into_inner();
    player_velocity.0 = intent.move_direction * 150.;
}
