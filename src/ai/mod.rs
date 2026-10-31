use std::time::Duration;

use avian2d::{dynamics::rigid_body::LinearVelocity, spatial_query::SpatialQuery};
use bevy::{
    app::{FixedUpdate, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::Children,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Query, Single},
    },
    gizmos::gizmos::Gizmos,
    math::Vec3,
    time::common_conditions::on_timer,
    transform::components::Transform,
};

use crate::{
    ai::{
        enemy_vision::{
            ENEMY_POINT_DATA_COUNT, ENEMY_POINTS, MAX_VISIBLE_ENEMIES,
            gather_visible_enemy_information,
        },
        environment_vision::{NUM_LIDAR, gather_environment_information},
    },
    character_controller,
    combat::{
        health::{self, Health},
        weapon::WeaponRecoil,
    },
    player::{
        Player,
        body::{self, Body},
    },
};

mod enemy_vision;
mod environment_vision;

const FOV_RADIANS: f32 = 1.57;
const SELF_DATA_COUNT: usize = 3;

#[derive(Component, Default)]
pub struct AiObservation {
    tensor: Vec<f32>,
}

impl AiObservation {
    pub fn tensor(&self) -> &Vec<f32> {
        &self.tensor
    }
}

#[derive(Component)]
pub struct GizmosTarget;

pub(super) struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            (gather_observation_system)
                .before(crate::input_system::InputSystems)
                .run_if(on_timer(Duration::from_secs_f32(1. / 16.))),
        )
        .add_systems(Update, draw_gizmos);
    }
}

fn gather_observation_system(
    mut agent_query: Query<(
        &mut AiObservation,
        &Children,
        Entity,
        &Transform,
        &Health,
        &WeaponRecoil,
        &LinearVelocity,
        &Player,
    )>,
    body_query: Query<&Transform, With<Body>>,
    player_query: Query<(Entity, &Player, &Transform)>,
    spatial_query: SpatialQuery,
) {
    for (
        mut agent_observation,
        agent_children,
        agent_entity,
        agent_transform,
        agent_health,
        agent_recoil,
        agent_velocity,
        agent_player,
    ) in agent_query.iter_mut()
    {
        if let Some(agent_body_transform) = agent_children
            .iter()
            .find_map(|&child| body_query.get(child).ok())
        {
            let mut tensor = Vec::with_capacity(
                SELF_DATA_COUNT
                    + ENEMY_POINTS * ENEMY_POINT_DATA_COUNT * MAX_VISIBLE_ENEMIES
                    + NUM_LIDAR,
            );

            tensor.push(agent_health.current() as f32 / health::MAX_HEALTH as f32);
            tensor.push(agent_recoil.accumulated() as f32 / agent_recoil.max() as f32);
            tensor
                .push((agent_velocity.length() / character_controller::MAX_VELOCITY).clamp(0., 1.));

            let forward_dir = (agent_body_transform.rotation * Vec3::X).truncate();
            let pos = agent_transform.translation.truncate()
                + (agent_body_transform.rotation * body::PLAYER_BODY_TIP_SCALED.extend(0.))
                    .truncate();

            let enemies = player_query
                .iter()
                .filter(|(_, p, _)| p.team() != agent_player.team())
                .map(|(e, _, t)| (e, t))
                .collect();

            let mut enemy_information = gather_visible_enemy_information(
                &spatial_query,
                agent_entity,
                pos,
                forward_dir,
                &enemies,
            );

            for i in 0..MAX_VISIBLE_ENEMIES {
                if i < enemy_information.len() {
                    let data = &mut enemy_information[i];
                    tensor.append(&mut data.flatten());
                } else {
                    for _ in 0..(ENEMY_POINTS * ENEMY_POINT_DATA_COUNT) {
                        tensor.push(0.);
                    }
                }
            }

            tensor.extend(gather_environment_information(
                &spatial_query,
                agent_entity,
                pos,
                agent_body_transform.rotation,
            ));

            agent_observation.tensor = tensor;
        }
    }
}

fn draw_gizmos(
    agent_query: Single<(Entity, &Transform, &Player, &Children), With<GizmosTarget>>,
    body_query: Query<&Transform, With<Body>>,
    player_query: Query<(Entity, &Player, &Transform)>,
    spatial_query: SpatialQuery,
    gizmos: Gizmos,
) {
    let (agent_entity, agent_transform, agent_player, children) = agent_query.into_inner();

    if let Some(agent_body_transform) = children
        .iter()
        .find_map(|&child| body_query.get(child).ok())
    {
        let enemies = player_query
            .iter()
            .filter(|(_, p, _)| p.team() != agent_player.team())
            .map(|(e, _, t)| (e, t))
            .collect();

        let forward_dir = (agent_body_transform.rotation * Vec3::X).truncate();
        let pos = agent_transform.translation.truncate()
            + (agent_body_transform.rotation * body::PLAYER_BODY_TIP_SCALED.extend(0.)).truncate();

        enemy_vision::draw_gizmos(
            agent_entity,
            pos,
            forward_dir,
            &enemies,
            &spatial_query,
            gizmos,
        );
    }
}
