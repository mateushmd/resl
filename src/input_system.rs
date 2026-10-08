use bevy::{app::{Plugin, Update}, ecs::{component::Component, query::With, system::{Query, Res, Single}}, input::{ButtonInput, keyboard::KeyCode, mouse::MouseButton}, math::Vec2, transform::components::Transform};

use crate::MouseWorldPosition;

#[derive(Component, Default)]
struct HumanControlled;

#[derive(Component, Default)]
struct  AiControlled(usize);

#[derive(Component, Default)]
struct CharacterIntent {
    move_direction: Vec2,
    look_angle: f32,
    is_shooting: bool,
}

struct InputSystemPlugin;

impl Plugin for InputSystemPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, human_input_system);     
    }
}

fn ai_input_system(mut query: Query<&mut CharacterIntent, With<AiControlled>>) {
    todo!("ai input system")
}

fn human_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MouseWorldPosition>,
    mut query: Single<(&Transform, &mut CharacterIntent), With<HumanControlled>>
) {
    let (transform, mut intent) = query.into_inner();

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { direction.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { direction.x += 1.0; }

    intent.move_direction = direction.normalize_or_zero();

    if let Some(target_pos) = mouse_pos.0 {
        let diff = target_pos - transform.translation.truncate();
        intent.look_angle = diff.y.atan2(diff.x);
    }

    intent.is_shooting = mouse.pressed(MouseButton::Left);
}
