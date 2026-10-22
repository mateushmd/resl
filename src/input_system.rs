use bevy::{
    app::{Plugin, Update}, ecs::{
        component::Component, query::With, resource::Resource, schedule::{IntoScheduleConfigs, SystemSet}, system::{Query, Res, Single, command::init_resource},
    }, input::{ButtonInput, keyboard::KeyCode, mouse::MouseButton}, math::{EulerRot, Vec2}, transform::components::Transform,
};

use crate::MouseWorldPosition;

const AI_MAX_TURN_RADIANS_PER_TICK: f32 = 0.15;

#[derive(Resource, Default)]
struct AiActionBuffer {
    pub latest_action: [f32; 4]
}

pub enum Controller {
    Ai(usize),
    Human,
}

#[derive(Component, Default)]
pub struct HumanControlled;

#[derive(Component, Default)]
pub struct AiControlled(usize);

impl AiControlled {
    pub fn new(id: usize) -> Self {
        AiControlled(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Component, Default)]
pub struct CharacterIntent {
    pub move_direction: Vec2,
    pub look_angle: f32,
    pub is_shooting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct InputSystems;

pub(super) struct InputSystemPlugin;

impl Plugin for InputSystemPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_systems(Update, (human_input_system, ai_input_system).in_set(InputSystems))
            .init_resource::<AiActionBuffer>();
    }
}

fn ai_input_system(
    action_buffer: Res<AiActionBuffer>,
    query: Query<(&Transform, &mut CharacterIntent), With<AiControlled>>
) {
    let actions = action_buffer.latest_action;

    for (transform, mut intent) in query {
        let mut direction = Vec2::ZERO;

        if actions[0] > 0.3 { direction.x += 1.; }
        else if actions[0] < -0.3 { direction.x -= 1.; }

        if actions[1] > 0.3 { direction.y += 1.; }
        else if actions[1] < -0.3 { direction.y -= 1.; }

        intent.move_direction = direction;

        let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;

        let turn_delta = actions[2] * AI_MAX_TURN_RADIANS_PER_TICK;

        intent.look_angle = current_angle + turn_delta;

        intent.is_shooting = actions[3] > 0.;
    }
}

fn human_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MouseWorldPosition>,
    query: Option<Single<(&Transform, &mut CharacterIntent), With<HumanControlled>>>,
) {
    if let Some(q) = query {
        let (transform, mut intent) = q.into_inner();

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
}
