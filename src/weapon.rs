use avian2d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    app::{AppExit, FixedUpdate, Plugin, Update}, color::palettes::css, ecs::{
        component::Component, entity::Entity, hierarchy::ChildOf, message::{Message, MessageReader, MessageWriter}, relationship::RelatedSpawnerCommands, schedule::IntoScheduleConfigs, system::{EntityCommands, Query, Res}
    }, gizmos::gizmos::Gizmos, math::{Dir2, Vec2, Vec3}, pbr::ViewFogUniformOffset, time::Time, transform::components::{GlobalTransform, Transform},
};

#[derive(Component)]
#[require(Transform)]
pub(crate) struct WeaponMuzzle(Vec2);

#[derive(Component)]
struct FireInterval {
    ticks: u32,
    accumulated: u32
}

impl FireInterval {
    fn new(ticks: u32) -> Self {
        FireInterval {
            ticks,
            accumulated: 0
        }
    }
}

impl WeaponMuzzle {
    pub fn new(offset: Vec2) -> Self {
        WeaponMuzzle(offset)
    }
}

pub(crate) trait SpawnWeaponExt {
    fn spawn_waepon(&mut self, offset: Vec2) -> EntityCommands;
}

impl SpawnWeaponExt for RelatedSpawnerCommands<'_, ChildOf> {
    fn spawn_waepon(&mut self, offset: Vec2) -> EntityCommands {
        self.spawn((
            WeaponMuzzle(offset),
            FireInterval::new(8)
        ))
    }
}

#[derive(Message, Clone)]
pub(crate) struct FireWeaponMessage {
    pub weapon: Entity,
}

pub(super) struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<FireWeaponMessage>()
            .add_systems(Update, draw_gizmos)
            .add_systems(FixedUpdate, (update_fire_intervals, shoot_system).chain());
    }
}

fn update_fire_intervals(
    mut query: Query<&mut FireInterval>
) {
    for mut fire_interval in query {
        if fire_interval.accumulated < fire_interval.ticks {
            fire_interval.accumulated += 1;
        }
    }
}

fn shoot_system(
    mut fire_messages: MessageReader<FireWeaponMessage>,
    mut query: Query<(&GlobalTransform, &Transform, &WeaponMuzzle, &mut FireInterval)>,
    spatial_query: SpatialQuery,
    mut message_writter: MessageWriter<AppExit>,
) {
    for event in fire_messages.read() {
        if let Ok((global_transform, transform, muzzle, mut fire_interval)) = query.get_mut(event.weapon) {
            if fire_interval.accumulated >= fire_interval.ticks {
                fire_interval.accumulated = 0;

                let pos = global_transform.translation();
                let rotation = transform.rotation;

                let rotated_offset = rotation * muzzle.0.extend(0.);

                let ray_pos = (pos + rotated_offset).truncate();
                let forward = (rotation * Vec3::X).truncate();

                if let Ok(ray_dir) = Dir2::new(forward) {
                    if let Some(hit) = spatial_query.cast_ray(
                        ray_pos,
                        ray_dir,
                        2000.,
                        true,
                        &SpatialQueryFilter::default(),
                    ) {
                        println!("hit");
                    }
                }
            }
        } else {
            bevy::log::tracing::error!(
                "the entity from the message payload lacks a component of type WeaponMuzzle"
            );
            message_writter.write(AppExit::error());
        }
    }
}

fn draw_gizmos(query: Query<(&GlobalTransform, &Transform, &WeaponMuzzle)>, mut gizmos: Gizmos) {
    for (global_transform, transform, muzzle) in &query {
        let pos = global_transform.translation();
        let rotation = transform.rotation;

        let rotated_offset = rotation * muzzle.0.extend(0.);

        let ray_pos = (pos + rotated_offset).truncate();
        let forward = (rotation * Vec3::X).truncate();

        gizmos.circle_2d(ray_pos, 4.0, css::RED);
        gizmos.line_2d(ray_pos, ray_pos + (forward * 500.), css::YELLOW);
    }
}
