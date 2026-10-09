use avian2d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::{app::{Plugin, Update}, ecs::{component::Component, hierarchy::Children, system::Query}, math::{Dir2, Vec2, Vec3}, transform::components::Transform};

use crate::input_system::CharacterIntent;

#[derive(Component)]
pub(crate) struct ShootOffset(pub Vec2);

pub(super) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, shooting_system);
    }
}

fn shooting_system(
    parent_query: Query<(&CharacterIntent, &Children, &ShootOffset, &Transform)>,
    child_query: Query<&Transform>,
    spatial_query: SpatialQuery
) {
    for (intent, children, offset, parent_transform) in &parent_query {
        if intent.is_shooting {
            for &child in children  {
                if let Ok(child_transform) = child_query.get(child) {
                    let pos = parent_transform.translation.truncate();
                    let rotation = child_transform.rotation;

                    let rotated_offset = (rotation * offset.0.extend(0.)).truncate();
                    let ray_origin = pos + rotated_offset;

                    let forward = (rotation * Vec3::X).truncate();
                    
                    if let Ok(ray_dir) = Dir2::new(forward) {
                        if let Some(hit) = spatial_query.cast_ray(ray_origin, ray_dir, 2000., true, &SpatialQueryFilter::default()) {
                            println!("HIT!"); 
                        }
                        else {
                            println!("MISS");
                        }
                    }
                }
            }
        }
    }
}
