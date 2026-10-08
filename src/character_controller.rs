use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LinearVelocity, RigidBody},
};
use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        query::With,
        system::{Res, Single},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::Vec2,
    transform::components::Transform,
};

use crate::input_system::CharacterIntent;

#[derive(Component, Default)]
#[require(CharacterIntent, Collider, RigidBody, Transform)]
pub struct CharacterController;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, character_controller);
    }
}

fn character_controller(
    input: Res<ButtonInput<KeyCode>>,
    query: Single<(&CharacterIntent, &mut LinearVelocity), With<CharacterController>>,
) {
    let (intent, mut player_velocity) = query.into_inner();
    player_velocity.0 = intent.move_direction
        * 100.
        * if input.pressed(KeyCode::ShiftLeft) {
            1.5
        } else {
            1.
        };
}
