use avian2d::{ collision::collider::Collider, dynamics::rigid_body::{LinearVelocity, RigidBody}};
use bevy::{app::{Plugin, Update}, ecs::{component::Component, query::With, system::{Res, Single}}, input::{ButtonInput, keyboard::KeyCode}, math::Vec2, transform::components::Transform};

#[derive(Component, Default)]
#[require(Collider, RigidBody, Transform)]
pub struct CharacterController;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, character_controller);  
    }
}

fn character_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut player_velocity: Single<&mut LinearVelocity, With<CharacterController>>
) {
    let mut x = 0.;
    let mut y = 0.;

    if input.pressed(KeyCode::KeyA) {
        x += -1.;
    } 

    if input.pressed(KeyCode::KeyD) {
       x += 1.;
    }

    if input.pressed(KeyCode::KeyW) {
        y += 1.;
    }

    if input.pressed(KeyCode::KeyS) {
        y += -1.;
    }

    let direction = Vec2::new(x, y).normalize_or_zero();
    
    player_velocity.0 = direction * 100. * if input.pressed(KeyCode::ShiftLeft) { 1.5 } else { 1. };
}
