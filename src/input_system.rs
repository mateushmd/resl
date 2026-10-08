use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        query::With,
        system::{Query, Res, Single},
    },
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    math::Vec2,
    transform::components::Transform,
};

use crate::MouseWorldPosition;

pub(crate) enum Controller {
    Ai(usize),
    Human,
}

#[derive(Component, Default)]
pub(crate) struct HumanControlled;

#[derive(Component, Default)]
pub(crate) struct AiControlled(usize);

impl AiControlled {
    pub fn new(id: usize) -> Self {
        AiControlled(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl Controller {
    pub fn bundle(&self) -> (Option<HumanControlled>, Option<AiControlled>) {
        match self {
            Self::Ai(id) => (None, Some(AiControlled(*id))),
            Self::Human => (Some(HumanControlled), None),
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct CharacterIntent {
    pub move_direction: Vec2,
    pub look_angle: f32,
    pub is_shooting: bool,
}

pub(super) struct InputSystemPlugin;

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
    mut query: Single<(&Transform, &mut CharacterIntent), With<HumanControlled>>,
) {
    let (transform, mut intent) = query.into_inner();

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    intent.move_direction = direction.normalize_or_zero();

    if let Some(target_pos) = mouse_pos.0 {
        let diff = target_pos - transform.translation.truncate();
        intent.look_angle = diff.y.atan2(diff.x);
    }

    intent.is_shooting = mouse.pressed(MouseButton::Left);
}
