use avian2d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    ecs::entity::Entity,
    math::{Dir2, Quat, Vec2, Vec3},
};

use crate::{LayerMasks, ai::FOV_RADIANS};

pub const NUM_LIDAR: usize = 17;
const START_ANGLE: f32 = -FOV_RADIANS / 2.;
const ANGLE_STEP: f32 = FOV_RADIANS / (NUM_LIDAR - 1) as f32;

pub fn gather_environment_information(
    spatial_query: &SpatialQuery,
    agent_entity: Entity,
    agent_eye_position: Vec2,
    agent_rotation: Quat,
) -> Vec<f32> {
    let mut lidar_tensor = Vec::with_capacity(NUM_LIDAR);

    let query_filter = SpatialQueryFilter::with_excluded_entities(
        SpatialQueryFilter::from_mask(LayerMasks::default()),
        [agent_entity],
    );

    for i in 0..NUM_LIDAR {
        let raw_angle = START_ANGLE + ANGLE_STEP * i as f32;
        let rotation = agent_rotation * Quat::from_rotation_z(raw_angle);
        let forward = (rotation * Vec3::X).truncate();

        if let Ok(dir) = Dir2::new(forward)
            && let Some(hit) =
                spatial_query.cast_ray(agent_eye_position, dir, 2000., true, &query_filter)
        {
            lidar_tensor.push(hit.distance / 2000.)
        } else {
            lidar_tensor.push(1.);
        }
    }

    lidar_tensor
}
