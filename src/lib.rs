use bevy::{app::{Plugin, PreUpdate, Startup}, asset::Assets, camera::{Camera, Camera2d}, color::Color, ecs::{component::Component, query::With, resource::Resource, schedule::IntoScheduleConfigs, system::{Commands, ResMut, Single}}, input::InputSystems, math::Vec2, mesh::Mesh, sprite_render::ColorMaterial, transform::components::GlobalTransform, window::{PrimaryWindow, Window}};

use crate::{character_controller::CharacterControllerPlugin, player::{Player, PlayerMovementPlugin}};

mod character_controller;
mod player;

#[derive(Resource, Default)]
struct MouseWorldPosition(pub Option<Vec2>);

fn update_mouse_world_position(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut mouse_pos: ResMut<MouseWorldPosition>
) {
    if let Some(cursor_pos) = window.cursor_position() {
        let (cam, cam_transform) = *camera;

        mouse_pos.0 = cam.viewport_to_world_2d(cam_transform, cursor_pos).ok();
    } else {
        mouse_pos.0 = None;
    }
}

#[derive(Component)]
enum Team {
    Attackers,
    Defenders
}

impl Team {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Attackers => Color::srgba(1., 0., 0., 1.),
            Self::Defenders => Color::srgba(0., 0., 1., 1.)
        }
    }
}

pub struct RESLPlugin;

impl Plugin for RESLPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app

            // plugins
            .add_plugins((
                CharacterControllerPlugin,
                PlayerMovementPlugin
            ))

            // systems
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, update_mouse_world_position.after(InputSystems))

            // resources
            .init_resource::<MouseWorldPosition>();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(Camera2d); 
    Player::spawn(&mut commands, &mut meshes, &mut color_materials, Vec2::ZERO, Team::Attackers);
}
