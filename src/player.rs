use avian2d::{
    collision::collider::Collider, dynamics::rigid_body::RigidBody
};

use bevy::{
    app::{Plugin, Update}, asset::Assets, ecs::{
        bundle::Bundle, component::Component, query::{With, Without}, system::{Commands, Res, Single}
    }, math::{
        Quat, Vec2, Vec3, primitives::Triangle2d
    }, mesh::{
        Mesh,
        Mesh2d
    }, sprite_render::{
        ColorMaterial,
        MeshMaterial2d
    }, transform::components::Transform
};

use crate::{MouseWorldPosition, Team, character_controller::CharacterController, input_system::{AiControlled, CharacterIntent, Controller, HumanControlled}};

const PLAYER_BODY_TIP: Vec2 = Vec2::new(0.5, 0.);
const PLAYER_BODY_BASE_TOP: Vec2 = Vec2::new(-0.25, 0.4330127);
const PLAYER_BODY_BASE_BOTTOM: Vec2 = Vec2::new(-0.25, -0.4330127);


#[derive(Component)]
#[require(CharacterController, CharacterIntent, Transform)]
pub(super) struct Player(Team);

#[derive(Component)]
#[require(Collider, MeshMaterial2d<ColorMaterial>, Mesh2d, Transform)]
struct PlayerBody;

impl Player {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>, 
        color_materials: &mut Assets<ColorMaterial>,
        controller: Controller,
        position: Vec2,
        team: Team
    ) {
        let team_color = team.get_color();

        let mut entity_cmds = commands.spawn((
            CharacterController,
            Player(team),
            RigidBody::Kinematic,
            Collider::circle(16.),
            Transform::from_translation(position.extend(1.))
        ));

        entity_cmds.with_children(|parent| {
            parent.spawn((
                PlayerBody,
                Collider::triangle_unchecked(
                    PLAYER_BODY_TIP,
                    PLAYER_BODY_BASE_TOP,
                    PLAYER_BODY_BASE_BOTTOM
                ),
                Mesh2d(meshes.add(Triangle2d::new(
                    PLAYER_BODY_TIP,
                    PLAYER_BODY_BASE_TOP,
                    PLAYER_BODY_BASE_BOTTOM
                ))),
                MeshMaterial2d(color_materials.add(team_color)),
                Transform::from_scale(Vec2::splat(32.).extend(1.))
                    .with_translation(Vec3::new(0., 0., 0.1))
            ));
        });

        match controller {
            Controller::Ai(id) => entity_cmds.insert(AiControlled::new(id)),
            Controller::Human => entity_cmds.insert(HumanControlled)
        };
    }
}

pub(super) struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, player_rotation);
    }
}

fn player_rotation(
    mouse_world_position: Res<MouseWorldPosition>,
    intent: Single<&CharacterIntent, (With<Player>, Without<PlayerBody>)>,
    mut player_body_transform: Single<&mut Transform, (With<PlayerBody>, Without<Player>)>
) {
    player_body_transform.rotation = Quat::from_rotation_z(intent.look_angle);
}
