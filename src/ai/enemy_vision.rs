use avian2d::{
    parry::math::VectorExt,
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::{
    color::palettes::css,
    ecs::{entity::Entity, hierarchy::Children},
    gizmos::gizmos::Gizmos,
    math::{Dir2, Vec2, Vec3},
    transform::components::Transform,
};

use crate::{
    LayerMasks,
    ai::FOV_RADIANS,
    player::{Player, body},
};

pub const MAX_VISIBLE_ENEMIES: usize = 2;
pub const ENEMY_POINTS: usize = 4;
pub const ENEMY_POINT_DATA_COUNT: usize = 3;

#[derive(Clone, Default)]
pub struct VisibleEnemyPoint {
    angle: f32,
    distance: f32,
    visible: f32,
}

impl VisibleEnemyPoint {
    pub fn new(angle: f32, distance: f32, visible: f32) -> Self {
        VisibleEnemyPoint {
            angle,
            distance,
            visible,
        }
    }

    pub fn flatten(&self) -> Vec<f32> {
        vec![self.angle, self.distance, self.visible]
    }
}

pub fn gather_visible_enemy_information(
    spatial_query: &SpatialQuery,
    agent_entity: Entity,
    agent_eye_position: Vec2,
    agent_forward: Vec2,
    enemies: &Vec<(Entity, &Transform)>,
) -> Vec<VisibleEnemyPoint> {
    let mut visible_enemies = Vec::<(f32, Vec<VisibleEnemyPoint>)>::new();

    let mut iter = enemies.iter();

    while visible_enemies.len() < MAX_VISIBLE_ENEMIES
        && let Some((enemy_entity, enemy_transform)) = iter.next()
    {
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
        let mut enemy_points_data = Vec::<VisibleEnemyPoint>::with_capacity(ENEMY_POINTS);
        let mut center_distance = f32::MAX;

        let query_filter = &SpatialQueryFilter::with_excluded_entities(
            SpatialQueryFilter::from_mask(LayerMasks::default()),
            [agent_entity, *enemy_entity],
        );

        for (i, point) in points_to_check.iter().enumerate() {
            let to_target = point - agent_eye_position;
            let distance = to_target.length();
            let dir_to_target = to_target.normalize_or_zero();
            let angle_to_target = agent_forward.angle(dir_to_target);

            if i == 0 {
                center_distance = distance
            };

            is_visible = if angle_to_target.abs() <= (FOV_RADIANS / 2.)
                && let Ok(dir) = Dir2::new(dir_to_target)
            {
                !spatial_query
                    .cast_ray(agent_eye_position, dir, distance, true, query_filter)
                    .is_some()
            } else {
                false
            };

            enemy_points_data.push(VisibleEnemyPoint::new(
                angle_to_target / (FOV_RADIANS / 2.),
                (distance / 2000.).clamp(0., 1.),
                if is_visible { 1. } else { 0. },
            ));
        }

        if is_visible {
            visible_enemies.push((center_distance, enemy_points_data));
        }
    }

    visible_enemies.sort_by(|(d_a, _), (d_b, _)| d_a.partial_cmp(&d_b).unwrap());

    visible_enemies.into_iter().flat_map(|v| v.1).collect()
}

pub fn draw_gizmos(
    agent_entity: Entity,
    agent_position: Vec2,
    agent_forward: Vec2,
    enemies: &Vec<(Entity, &Transform)>,
    spatial_query: &SpatialQuery,
    mut gizmos: Gizmos,
) {
    let mut visible_enemies = Vec::<(f32, Vec2)>::new();

    for (enemy_entity, enemy_transform) in enemies {
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
            [agent_entity, *enemy_entity],
        );

        for point in points_to_check.iter() {
            let to_target = point - agent_position;
            let distance = to_target.length();
            let dir_to_target = to_target.normalize_or_zero();
            let angle_to_target = agent_forward.angle(dir_to_target);

            if angle_to_target.abs() <= (FOV_RADIANS / 2.) {
                if let Ok(dir) = Dir2::new(dir_to_target) {
                    let is_blocked = spatial_query
                        .cast_ray(agent_position, dir, distance, true, query_filter)
                        .is_some();

                    if !is_blocked {
                        gizmos.line_2d(agent_position, *point, css::GREEN);
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
