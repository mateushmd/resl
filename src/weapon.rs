use avian2d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    app::{AppExit, FixedUpdate, Plugin, Update}, color::palettes::css, ecs::{
        component::Component, entity::Entity, hierarchy::ChildOf, message::{Message, MessageReader, MessageWriter}, relationship::RelatedSpawnerCommands, schedule::IntoScheduleConfigs, system::{Commands, EntityCommands, Query, Res}
    }, gizmos::gizmos::Gizmos, math::{Dir2, Quat, Vec2, Vec3}, pbr::ViewFogUniformOffset, time::Time, transform::components::{GlobalTransform, Transform},
};
use rand::RngExt;
use crate::sfx::SpawnSfxExt;

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

#[derive(Component)]
#[require(Transform)]
pub(crate) struct WeaponMuzzle(Vec2);

impl WeaponMuzzle {
    pub fn new(offset: Vec2) -> Self {
        WeaponMuzzle(offset)
    }
}

#[derive(Component)]
struct WeaponRecoil {
    gain: u32,
    recover: u32,
    accumulated: u32,
    max: u32,
    spread_radians_per_unit: f32
}

impl WeaponRecoil {
    fn new(gain: u32, recover: u32, max: u32, spread_radians_per_unit: f32) -> Self {
        WeaponRecoil {
            gain,
            recover,
            accumulated: 0,
            max,
            spread_radians_per_unit
        }
    }
}

pub(crate) trait SpawnWeaponExt {
    fn spawn_waepon(&mut self, offset: Vec2) -> EntityCommands;
}

impl SpawnWeaponExt for RelatedSpawnerCommands<'_, ChildOf> {
    fn spawn_waepon(&mut self, offset: Vec2) -> EntityCommands {
        self.spawn((
            WeaponMuzzle(offset),
            FireInterval::new(8),
            WeaponRecoil::new(20, 1, 100, 0.0025)
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
            .add_systems(FixedUpdate, (update_fire_intervals, recoil_wear_off, shoot_system).chain());
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
    mut commands: Commands,
    mut fire_messages: MessageReader<FireWeaponMessage>,
    mut query: Query<(&mut FireInterval, &GlobalTransform, &Transform, &WeaponMuzzle, &mut WeaponRecoil)>,
    spatial_query: SpatialQuery,
    mut message_writter: MessageWriter<AppExit>,
) {
    for event in fire_messages.read() {
        if let Ok((mut fire_interval, global_transform, transform, muzzle, mut recoil)) = query.get_mut(event.weapon) {
            if fire_interval.accumulated >= fire_interval.ticks {
                fire_interval.accumulated = 0;

                let pos = global_transform.translation();
                let rotation = transform.rotation;

                let rotated_offset = rotation * muzzle.0.extend(0.);

                let ray_pos = (pos + rotated_offset).truncate();
                
                let current_spread = recoil.accumulated as f32 * recoil.spread_radians_per_unit;

                let mut rng = rand::rng();
                let noise_angle = rng.random_range(-current_spread..=current_spread);

                let recoil_rotation = rotation * Quat::from_rotation_z(noise_angle);
                
                let forward = (recoil_rotation * Vec3::X).truncate();

                if let Ok(ray_dir) = Dir2::new(forward) {
                    let (midpoint, distance) = if let Some(hit) = spatial_query.cast_ray(
                        ray_pos,
                        ray_dir,
                        2000.,
                        true,
                        &SpatialQueryFilter::default(),
                    ) {
                        println!("hit");
                        (ray_pos + (ray_dir * (hit.distance / 2.)), hit.distance)
                    } else {
                        (ray_pos + (ray_dir * 1000.), 2000.)
                    };

                    commands.spawn_bullet_beam(midpoint, recoil_rotation, distance);

                    recoil.accumulated = (recoil.accumulated + recoil.gain).min(recoil.max);
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

fn recoil_wear_off(
    query: Query<&mut WeaponRecoil>
) {
    for mut recoil in query {
        recoil.accumulated = recoil.accumulated.saturating_sub(recoil.recover);
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
