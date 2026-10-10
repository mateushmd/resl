use avian2d::{collision::collider::Collider, dynamics::rigid_body::RigidBody};
use bevy::{
    app::{FixedUpdate, Plugin, PostStartup},
    asset::Handle,
    ecs::{
        component::Component,
        hierarchy::Children,
        message::MessageWriter,
        query::With,
        resource::Resource,
        system::{Commands, EntityCommands, Query, Res},
    },
    math::{Quat, Vec2},
    mesh::Mesh,
    sprite_render::ColorMaterial,
    transform::components::Transform,
};

use crate::{
    Team, character_controller::CharacterController, input_system::{AiControlled, CharacterIntent, Controller, HumanControlled}, player::body::{BodyPlugin, SpawnBodyExt}, weapon::{FireWeaponMessage, SpawnWeaponExt, WeaponMuzzle},
};

mod body;

#[derive(Resource, Clone)]
struct PlayerAssets {
    pub body_mesh: Handle<Mesh>,
    pub attacker_color: Handle<ColorMaterial>,
    pub defender_color: Handle<ColorMaterial>,
    pub dummy_color: Handle<ColorMaterial>,
}

#[derive(Component, Default)]
struct Rotate;

#[derive(Component)]
#[require(Transform)]
pub(super) struct Player(Team);

impl Player {
    fn spawn_base<'a>(
        commands: &'a mut Commands,
        player_assets: &PlayerAssets,
        position: Vec2,
        team: Option<Team>,
    ) -> EntityCommands<'a> {
        let mut entity_cmds = commands.spawn((
            RigidBody::Kinematic,
            Collider::circle(16.),
            Transform::from_translation(position.extend(1.)),
        ));

        entity_cmds.with_children(|parent| {
            parent.spawn_body(
                player_assets.body_mesh.clone(),
                match team {
                    Some(Team::Attackers) => player_assets.attacker_color.clone(),
                    Some(Team::Defenders) => player_assets.defender_color.clone(),
                    None => player_assets.dummy_color.clone(),
                },
            );
        });

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

        entity_cmds.insert((CharacterController, Player(team)));

        entity_cmds.with_children(|parent| {
            parent.spawn_waepon(Vec2::new(25., 0.)).insert(Rotate);
        });

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

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BodyPlugin)
            .add_systems(PostStartup, setup)
            .add_systems(FixedUpdate, (children_rotation, process_shoot_intents));
    }
}

fn setup(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    Player::spawn_player(
        &mut commands,
        &player_assets,
        Controller::Human,
        Vec2::ZERO,
        Team::Attackers,
    );

    Player::spawn_dummy(&mut commands, &player_assets, Vec2::new(100., 0.), None);
}

fn children_rotation(
    parent_query: Query<(&CharacterIntent, &Children), With<Player>>,
    mut child_query: Query<&mut Transform, With<Rotate>>,
) {
    for (intent, children) in &parent_query {
        for &child in children {
            if let Ok(mut transform) = child_query.get_mut(child) {
                transform.rotation = Quat::from_rotation_z(intent.look_angle);
            }
        }
    }
}

fn process_shoot_intents(
    parent_query: Query<(&CharacterIntent, &Children), With<Player>>,
    child_query: Query<&WeaponMuzzle>,
    mut writer: MessageWriter<FireWeaponMessage>,
) {
    for (intent, children) in &parent_query {
        if intent.is_shooting {
            for &child in children {
                if child_query.contains(child) {
                    writer.write(FireWeaponMessage { weapon: child });
                }
            }
        }
    }
}
