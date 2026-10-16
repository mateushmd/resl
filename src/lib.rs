use avian2d::prelude::PhysicsLayer;
use bevy::{
    app::{Plugin, PreUpdate, Startup},
    camera::{Camera, Camera2d},
    color::Color,
    ecs::{
        component::Component,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, ResMut, Single},
    },
    input::InputSystems,
    math::Vec2,
    transform::components::GlobalTransform,
    window::{PrimaryWindow, Window},
};

use crate::{
    ai::AiPlugin, character_controller::CharacterControllerPlugin, combat::CombatPlugin, input_system::InputSystemPlugin, player::PlayerPlugin, sfx::SfxPlugin,
};

mod ai;
mod character_controller;
mod combat;
mod input_system;
mod player;
mod sfx;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
enum LayerMasks {
    #[default]
    Default,
    Player,
}

#[derive(Clone, Component, Copy, PartialEq)]
#[repr(u8)]
enum Team {
    Attackers,
    Defenders
}

impl Team {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Attackers => Color::srgba(1., 0., 0., 1.),
            Self::Defenders => Color::srgba(0., 0., 1., 1.),
        }
    }

    pub fn get_opposite(&self) -> Self {
        match self {
            Self::Attackers => Self::Defenders,
            Self::Defenders => Self::Attackers
        }
    }
}

#[derive(Resource, Default)]
struct MouseWorldPosition(pub Option<Vec2>);

fn update_mouse_world_position(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut mouse_pos: ResMut<MouseWorldPosition>,
) {
    if let Some(cursor_pos) = window.cursor_position() {
        let (cam, cam_transform) = *camera;

        mouse_pos.0 = cam.viewport_to_world_2d(cam_transform, cursor_pos).ok();
    } else {
        mouse_pos.0 = None;
    }
}

pub struct RESLPlugin;

impl Plugin for RESLPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            // plugins
            .add_plugins((
                AiPlugin,
                CharacterControllerPlugin,
                CombatPlugin,
                InputSystemPlugin,
                PlayerPlugin,
                SfxPlugin,
            ))
            // systems
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, update_mouse_world_position.after(InputSystems))
            // resources
            .init_resource::<MouseWorldPosition>();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
