use avian2d::{
    collision::collider::Collider, dynamics::rigid_body::RigidBody
};

use bevy::{
    app::{Plugin, Update}, asset::Assets, ecs::{
        component::Component, query::{With, Without}, system::{Commands, Res, Single}
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

use crate::{MouseWorldPosition, Team, character_controller::CharacterController};

#[derive(Component)]
#[require(CharacterController, Transform)]
pub struct Player(Team);

const PLAYER_BODY_TIP: Vec2 = Vec2::new(0.5, 0.);
const PLAYER_BODY_BASE_TOP: Vec2 = Vec2::new(-0.25, 0.4330127);
const PLAYER_BODY_BASE_BOTTOM: Vec2 = Vec2::new(-0.25, -0.4330127);

#[derive(Component)]
#[require(Collider, MeshMaterial2d<ColorMaterial>, Mesh2d, Transform)]
struct PlayerBody;

impl Player {
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>, 
        color_materials: &mut Assets<ColorMaterial>,
        position: Vec2,
        team: Team
    ) {
        let team_color = team.get_color();

        commands.spawn((
            CharacterController,
            Player(team),
            RigidBody::Kinematic,
            Collider::circle(16.),
            Transform::from_translation(position.extend(1.))
        )).with_children(|parent| {
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
    }
}

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, player_rotation);
    }
}

fn player_rotation(
    mouse_world_position: Res<MouseWorldPosition>,
    player_transform: Single<&Transform, (With<Player>, Without<PlayerBody>)>,
    mut player_body_transform: Single<&mut Transform, (With<PlayerBody>, Without<Player>)>
) {
    if let Some(position) = mouse_world_position.0 {
        let diff = position - player_transform.translation.truncate();        

        let angle = diff.y.atan2(diff.x);

        player_body_transform.rotation = Quat::from_rotation_z(angle);
    }
}
