use std::time::Duration;

use avian2d::{
    dynamics::rigid_body::LinearVelocity,
    parry::math::VectorExt,
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::{
    app::{FixedUpdate, Plugin, Update},
    color::palettes::css,
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::Children,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Query, Single},
    },
    gizmos::gizmos::Gizmos,
    math::{Dir2, EulerRot, Vec2, Vec3},
    time::common_conditions::on_timer,
    transform::components::{GlobalTransform, Transform},
};

use crate::{
    character_controller,
    combat::{
        health::{self, Health},
        weapon::WeaponRecoil,
    },
    input_system::AiControlled,
    player::{
        body::{self, Body, PLAYER_BODY_TIP_SCALED},
        Player,
    },
    LayerMasks, Team,
};

mod enemy_vision;

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
    agent_query: Query<(
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
    let (attackers, defenders): (Vec<_>, Vec<_>) = player_query
        .iter()
        .partition(|(_, p, _)| p.team() == Team::Attackers);

    for (
        mut agent_observation,
        agent_children,
        agent_entity,
        agent_transform,
        agent_health,
        agent_recoil,
        agent_velocity,
        agent_player,
    ) in agent_query
    {
        if let Some(agent_body_transform) = agent_children
            .iter()
            .find_map(|&child| body_query.get(child).ok())
        {
            let enemies = match agent_player.team() {
                Team::Attackers => &defenders,
                Team::Defenders => &attackers,
            };

            let mut tensor = Vec::with_capacity(
                SELF_DATA_COUNT + ENEMY_POINTS * ENEMY_POINT_DATA_COUNT * MAX_VISIBLE_ENEMIES,
            );

            tensor.push(agent_health.current() as f32 / health::MAX_HEALTH as f32);
            tensor.push(agent_recoil.accumulated() as f32 / agent_recoil.max() as f32);
            tensor
                .push((agent_velocity.length() / character_controller::MAX_VELOCITY).clamp(0., 1.));

            let forward_dir = (agent_body_transform.rotation * Vec3::X).truncate();
            let pos = agent_transform.translation.truncate()
                + (agent_body_transform.rotation * body::PLAYER_BODY_TIP_SCALED.extend(0.))
                    .truncate();

            let mut visible_enemies = Vec::<(f32, Vec<f32>)>::new();

            for (enemy_entity, _, enemy_transform) in enemies {
                let center_world = enemy_transform.translation.truncate();

                let tip_world = enemy_transform
                    .transform_point(body::PLAYER_BODY_TIP_SCALED.extend(0.))
                    .truncate();
                let base_top_world = enemy_transform
                    .transform_point(body::PLAYER_BODY_BASE_TOP_SCALED.extend(0.))
                    .truncate();
                let base_bottom_world = enemy_transform
                    .transform_point(body::PLAYER_BODY_BASE_BOTTOM_SCALED.extend(0.))
                    .truncate();

                let points_to_check = [center_world, tip_world, base_top_world, base_bottom_world];

                let mut is_visible = false;
                let mut enemy_points_data =
                    Vec::<f32>::with_capacity(ENEMY_POINTS * ENEMY_POINT_DATA_COUNT);
                let mut center_distance = f32::MAX;

                let query_filter = &SpatialQueryFilter::with_excluded_entities(
                    SpatialQueryFilter::from_mask(LayerMasks::default()),
                    [agent_entity, enemy_entity.to_owned()],
                );

                for (i, point) in points_to_check.iter().enumerate() {
                    let to_target = point - pos;
                    let distance = to_target.length();
                    let dir_to_target = to_target.normalize_or_zero();
                    let angle_to_target = forward_dir.angle(dir_to_target);

                    if i == 0 {
                        center_distance = distance
                    };

                    if angle_to_target.abs() <= (FOV_RADIANS / 2.) {
                        if let Ok(dir) = Dir2::new(dir_to_target) {
                            let is_blocked = spatial_query
                                .cast_ray(pos, dir, distance, true, query_filter)
                                .is_some();

                            is_visible = !is_blocked;
                            enemy_points_data.push(angle_to_target / (FOV_RADIANS / 2.));
                            enemy_points_data.push((distance / 2000.).clamp(0., 1.));
                            enemy_points_data.push(if is_blocked { 0. } else { 1. });
                        } else {
                            enemy_points_data.extend_from_slice(&[0., 0., 0.]);
                        }
                    } else {
                        enemy_points_data.push(angle_to_target / (FOV_RADIANS / 2.));
                        enemy_points_data.push((distance / 2000.).clamp(0., 1.));
                        enemy_points_data.push(0.);
                    }
                }

                if is_visible {
                    visible_enemies.push((center_distance, enemy_points_data));
                }
            }

            visible_enemies.sort_by(|(d_a, _), (d_b, _)| d_a.partial_cmp(&d_b).unwrap());

            for i in 0..MAX_VISIBLE_ENEMIES {
                if i < visible_enemies.len() {
                    let (_, data) = &mut visible_enemies[i];
                    tensor.append(data);
                } else {
                    for _ in 0..(ENEMY_POINTS * ENEMY_POINT_DATA_COUNT) {
                        tensor.push(0.);
                    }
                }
            }

            agent_observation.tensor = tensor;
        }
    }
}

fn draw_gizmos(
    agent_query: Single<(Entity, &Transform, &Player, &Children), With<GizmosTarget>>,
    body_query: Query<&Transform, With<Body>>,
    player_query: Query<(Entity, &Player, &Transform)>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
) {
    let (attackers, defenders): (Vec<_>, Vec<_>) = player_query
        .iter()
        .partition(|(_, p, _)| p.team() == Team::Attackers);

    let (agent_entity, agent_transform, agent_player, children) = agent_query.into_inner();

    let mut visible_enemies = Vec::<(f32, Vec2)>::new();

    if let Some(agent_body_transform) = children
        .iter()
        .find_map(|&child| body_query.get(child).ok())
    {
        let enemies = match agent_player.team() {
            Team::Attackers => &defenders,
            Team::Defenders => &attackers,
        };

        let forward_dir = (agent_body_transform.rotation * Vec3::X).truncate();
        let pos = agent_transform.translation.truncate()
            + (agent_body_transform.rotation * body::PLAYER_BODY_TIP_SCALED.extend(0.)).truncate();

        for (enemy_entity, _, enemy_transform) in enemies {
            let center_world = enemy_transform.translation.truncate();

            let tip_world = enemy_transform
                .transform_point(body::PLAYER_BODY_TIP_SCALED.extend(0.))
                .truncate();
            let base_top_world = enemy_transform
                .transform_point(body::PLAYER_BODY_BASE_TOP_SCALED.extend(0.))
                .truncate();
            let base_bottom_world = enemy_transform
                .transform_point(body::PLAYER_BODY_BASE_BOTTOM_SCALED.extend(0.))
                .truncate();

            let points_to_check = [center_world, tip_world, base_top_world, base_bottom_world];

            let query_filter = &SpatialQueryFilter::with_excluded_entities(
                SpatialQueryFilter::from_mask(LayerMasks::default()),
                [agent_entity, enemy_entity.to_owned()],
            );

            for point in points_to_check.iter() {
                let to_target = point - pos;
                let distance = to_target.length();
                let dir_to_target = to_target.normalize_or_zero();
                let angle_to_target = forward_dir.angle(dir_to_target);

                if angle_to_target.abs() <= (FOV_RADIANS / 2.) {
                    if let Ok(dir) = Dir2::new(dir_to_target) {
                        let is_blocked = spatial_query
                            .cast_ray(pos, dir, distance, true, query_filter)
                            .is_some();

                        if !is_blocked {
                            gizmos.line_2d(pos, *point, css::GREEN);
                        }
                    }
                }
            }
        }
    }

    visible_enemies.sort_by(|(d_a, _), (d_b, _)| d_a.partial_cmp(&d_b).unwrap());

    for i in 0..MAX_VISIBLE_ENEMIES {
        if i < visible_enemies.len() {
            let (_, pos) = &mut visible_enemies[i];
            gizmos.circle_2d(*pos, 4., css::GREEN);
        }
    }
}
