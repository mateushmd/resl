use bevy::{app::{Plugin, Update}, color::{Alpha, palettes::css}, ecs::{component::Component, entity::Entity, system::{Commands, Query, Res}}, math::{Quat, Vec2}, sprite::Sprite, time::{Time, Timer, TimerMode}, transform::components::Transform};

pub(super) struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, animate_bullet_beam);
    }
}

pub(crate) trait SpawnSfxExt {
    fn spawn_bullet_beam(&mut self, position: Vec2, rotation: Quat, distance: f32);
}

impl SpawnSfxExt for Commands<'_, '_> {
    fn spawn_bullet_beam(&mut self, position: Vec2, rotation: Quat, distance: f32) {
        self.spawn((
            BulletBeam::new(0.7),
            Sprite {
                color: css::YELLOW.into(),
                custom_size: Some(Vec2::new(distance, 4.)),
                ..Default::default()
            },
            Transform::from_translation(position.extend(0.5))
                .with_rotation(rotation)
        ));
    }
}

#[derive(Component)]
struct BulletBeam {
    timer: Timer
}

impl BulletBeam {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once)
        }
    }
}

fn animate_bullet_beam(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BulletBeam, &mut Sprite)>
) {
    for (entity, mut beam, mut sprite) in &mut query {
        beam.timer.tick(time.delta());

        if beam.timer.is_finished() {
            commands.entity(entity).despawn();
        } else {
            let percent_left = beam.timer.fraction_remaining();
            sprite.color.set_alpha(percent_left);
        }
    }
}
