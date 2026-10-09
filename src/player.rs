use avian2d::{collision::collider::Collider, dynamics::rigid_body::RigidBody};

use bevy::{
    app::{FixedUpdate, Plugin, Startup, Update}, asset::{Assets, Handle}, color::{Color, palettes::css}, ecs::{
        component::Component, hierarchy::{ChildOf, Children}, query::With, relationship::RelatedSpawnerCommands, resource::Resource, system::{Commands, EntityCommands, Query, ResMut},
    }, gizmos::gizmos::Gizmos, math::{Quat, Vec2, Vec3, primitives::Triangle2d}, mesh::{Mesh, Mesh2d}, sprite_render::{ColorMaterial, MeshMaterial2d}, transform::components::Transform,
};

use crate::{
    Team, character_controller::CharacterController, combat::ShootOffset, input_system::{AiControlled, CharacterIntent, Controller, HumanControlled}
};

const PLAYER_BODY_TIP: Vec2 = Vec2::new(0.5, 0.);
const PLAYER_BODY_BASE_TOP: Vec2 = Vec2::new(-0.25, 0.4330127);
const PLAYER_BODY_BASE_BOTTOM: Vec2 = Vec2::new(-0.25, -0.4330127);

#[derive(Resource, Clone)]
struct PlayerAssets {
    pub body_mesh: Handle<Mesh>,
    pub attacker_color: Handle<ColorMaterial>,
    pub defender_color: Handle<ColorMaterial>,
    pub dummy_color: Handle<ColorMaterial>,
}

#[derive(Component)]
#[require(Transform)]
pub(super) struct Player(Team);

impl Player {
    fn spawn_body(
        parent: &mut RelatedSpawnerCommands<ChildOf>,
        mesh: Handle<Mesh>,
        color: Handle<ColorMaterial>
    ) {
        parent.spawn((
            PlayerBody,
            Collider::triangle_unchecked(
                PLAYER_BODY_TIP,
                PLAYER_BODY_BASE_TOP,
                PLAYER_BODY_BASE_BOTTOM,
            ),
            Mesh2d(mesh),
            MeshMaterial2d(color),
            Transform::from_scale(Vec2::splat(32.).extend(1.))
                .with_translation(Vec3::new(0., 0., 0.1))
        ));
    }

    fn spawn_base<'a>(
        commands: &'a mut Commands,
        player_assets: &PlayerAssets,
        position: Vec2,
        team: Option<Team>
    ) -> EntityCommands<'a> {

        let mut entity_cmds = commands.spawn((
            RigidBody::Kinematic,
            Collider::circle(16.),
            Transform::from_translation(position.extend(1.)),
        ));

        entity_cmds.with_children(|parent| Self::spawn_body(
            parent, 
            player_assets.body_mesh.clone(),
            match team {
                Some(Team::Attackers) => player_assets.attacker_color.clone(),
                Some(Team::Defenders) => player_assets.defender_color.clone(),
                None => player_assets.dummy_color.clone()
            }
        ));

        entity_cmds
    }

    fn spawn_player(
        commands: &mut Commands,
        player_assets: &PlayerAssets,
        controller: Controller,
        position: Vec2,
        team: Team,
    ) {
        let mut entity_cmds = Self::spawn_base(commands, player_assets, position, Some(team));

        entity_cmds.insert((
            CharacterController,
            Player(team),
            ShootOffset(Vec2::new(25., 0.))
        ));

        match controller {
            Controller::Ai(id) => entity_cmds.insert(AiControlled::new(id)),
            Controller::Human => entity_cmds.insert(HumanControlled),
        };
    }

    fn spawn_dummy(
        commands: &mut Commands,
        player_assets: &PlayerAssets,
        position: Vec2,
        team: Option<Team>,
    ) {
        Self::spawn_base(commands, player_assets, position, team);
    }
}

#[derive(Component)]
#[require(Collider, MeshMaterial2d<ColorMaterial>, Mesh2d, Transform)]
struct PlayerBody;

pub(super) struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, draw_gizmos)
            .add_systems(FixedUpdate, body_rotation);
    }
}

fn setup(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut color_material_assets: ResMut<Assets<ColorMaterial>>
) {
    // player body asset
    let body_mesh = mesh_assets.add(Triangle2d::new(
        PLAYER_BODY_TIP,
        PLAYER_BODY_BASE_TOP,
        PLAYER_BODY_BASE_BOTTOM
    ));

    // player body color assets
    let attacker_color = color_material_assets.add(Team::Attackers.get_color());
    let defender_color = color_material_assets.add(Team::Defenders.get_color());
    let dummy_color = color_material_assets.add(Color::srgb(0.6, 0.6, 0.6));

    let player_assets = PlayerAssets {
        body_mesh,
        attacker_color,
        defender_color,
        dummy_color
    };

    commands.insert_resource(player_assets.clone());

    Player::spawn_player(
        &mut commands,
        &player_assets, 
        Controller::Human,
        Vec2::ZERO,
        Team::Attackers,
    );

    Player::spawn_dummy(
        &mut commands,
        &player_assets,
        Vec2::new(100., 0.),
        None
    );
}

fn body_rotation(
    parent_query: Query<(&CharacterIntent, &Children)>,
    mut child_query: Query<&mut Transform, With<PlayerBody>>,
) {
    for (intent, children) in &parent_query {
        for &child in children {
            if let Ok(mut body_transform) = child_query.get_mut(child) {
                body_transform.rotation = Quat::from_rotation_z(intent.look_angle);
            }
        }
    }
}

fn draw_gizmos(
    parent_query: Query<(&Children, &ShootOffset, &Transform), With<CharacterIntent>>,
    child_query: Query<&Transform>,
    mut gizmos: Gizmos
) {
    for (children, offset, parent_transform,) in &parent_query {
        for &child in children {
            if let Ok(child_transform) = child_query.get(child) {
                let pos = parent_transform.translation.truncate();
                let rotated_offset = (child_transform.rotation * offset.0.extend(0.)).truncate();
                let ray_origin = pos + rotated_offset;

                let forward = (child_transform.rotation * Vec3::X).truncate();

                gizmos.circle_2d(ray_origin, 4.0, css::RED);
                gizmos.line_2d(ray_origin, ray_origin + (forward * 500.), css::YELLOW);
            }
        }
    }
}
