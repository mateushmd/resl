use avian2d::{collision::collider::Collider, dynamics::rigid_body::RigidBody};
use bevy::{
    app::{FixedUpdate, Plugin, PostStartup},
    asset::Handle,
    ecs::{
        component::Component,
        entity,
        hierarchy::Children,
        message::{MessageReader, MessageWriter},
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, EntityCommands, Query, Res},
    },
    math::{Quat, Vec2},
    mesh::Mesh,
    sprite_render::ColorMaterial,
    transform::components::Transform,
};

use crate::{
    Team,
    character_controller::CharacterController,
    combat::{
        health::{DeathMessage, Health},
        weapon::{FireWeaponMessage, SpawnWeaponExt, WeaponMuzzle},
    },
    input_system::{AiControlled, CharacterIntent, Controller, HumanControlled},
    player::body::{BodyPlugin, SpawnBodyExt},
};

pub mod body;

#[derive(Resource, Clone)]
pub(super) struct PlayerAssets {
    pub body_mesh: Handle<Mesh>,
    pub attacker_color: Handle<ColorMaterial>,
    pub defender_color: Handle<ColorMaterial>,
    pub dummy_color: Handle<ColorMaterial>,
}

#[derive(Component, Default)]
struct Rotate;

#[derive(Component)]
#[require(Transform)]
pub struct Player(Team);

impl Player {
    fn spawn_base<'a>(
        commands: &'a mut Commands,
        player_assets: &PlayerAssets,
        position: Vec2,
        team: Team,
    ) -> EntityCommands<'a> {
        let mut entity_cmds = commands.spawn((
            RigidBody::Kinematic,
            Collider::circle(16.),
            Health::new(),
            Player(team),
            Transform::from_translation(position.extend(1.)),
        ));

        let id = entity_cmds.id();

        entity_cmds.with_children(|parent| {
            parent.spawn_body(
                id,
                player_assets.body_mesh.clone(),
                match team {
                    Team::Attackers => player_assets.attacker_color.clone(),
                    Team::Defenders => player_assets.defender_color.clone(),
                },
            );
        });

        entity_cmds
    }

    pub(super) fn spawn_player<'a>(
        commands: &'a mut Commands,
        player_assets: &PlayerAssets,
        controller: Controller,
        position: Vec2,
        team: Team,
    ) -> EntityCommands<'a> {
        let mut entity_cmds = Self::spawn_base(commands, player_assets, position, team);

        entity_cmds.insert(CharacterController);

        entity_cmds.with_children(|parent| {
            parent.spawn_weapon(Vec2::new(25., 0.)).insert(Rotate);
        });

        match controller {
            Controller::Ai(id) => entity_cmds.insert(AiControlled::new(id)),
            Controller::Human => entity_cmds.insert(HumanControlled),
        };

        entity_cmds
    }

    pub(super) fn spawn_dummy<'a>(
        commands: &'a mut Commands,
        player_assets: &PlayerAssets,
        position: Vec2,
        team: Team,
    ) -> EntityCommands<'a> {
        Self::spawn_base(commands, player_assets, position, team)
    }

    pub fn team(&self) -> Team {
        self.0
    }
}

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BodyPlugin)
            .add_systems(PostStartup, setup)
            .add_systems(
                FixedUpdate,
                (
                    children_rotation.after(crate::input_system::InputSystems),
                    process_shoot_intents,
                    process_death_messages,
                ),
            );
    }
}

fn setup(mut commands: Commands, player_assets: Res<PlayerAssets>) {}

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
                    writer.write(FireWeaponMessage::new(child));
                }
            }
        }
    }
}

fn process_death_messages(
    mut commands: Commands,
    query: Query<&Player>,
    mut message_reader: MessageReader<DeathMessage>,
) {
    for message in message_reader.read() {
        println!("{}", query.contains(message.target()));
        if query.contains(message.target()) {
            commands.entity(message.target()).despawn();
        }
    }
}
