use avian2d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    app::{AppExit, FixedUpdate, Plugin, Update},
    color::palettes::css,
    ecs::{
        component::Component,
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        system::Query,
    },
    gizmos::gizmos::Gizmos,
    math::{Dir2, Vec2, Vec3},
    transform::components::{GlobalTransform, Transform},
};

#[derive(Component)]
#[require(Transform)]
pub(crate) struct WeaponMuzzle(Vec2);

impl WeaponMuzzle {
    pub fn new(offset: Vec2) -> Self {
        WeaponMuzzle(offset)
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
            .add_systems(FixedUpdate, shooting_system);
    }
}

fn shooting_system(
    mut fire_messages: MessageReader<FireWeaponMessage>,
    query: Query<(&GlobalTransform, &Transform, &WeaponMuzzle)>,
    spatial_query: SpatialQuery,
    mut message_writter: MessageWriter<AppExit>,
) {
    for event in fire_messages.read() {
        if let Ok((global_transform, transform, muzzle)) = query.get(event.weapon) {
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
