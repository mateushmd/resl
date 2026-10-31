use avian2d::collision::collider::{Collider, CollisionLayers, LayerMask};

use bevy::{
    app::{Plugin, Startup},
    asset::{Assets, Handle},
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        hierarchy::ChildOf,
        relationship::RelatedSpawnerCommands,
        system::{Commands, ResMut},
    },
    math::{Vec2, Vec3, primitives::Triangle2d},
    mesh::{Mesh, Mesh2d},
    sprite_render::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
};

use crate::{
    LayerMasks, Team,
    combat::health::DamageTargetReference,
    player::{PlayerAssets, Rotate},
};

const BASE_SCALE: f32 = 32.;

pub const PLAYER_BODY_TIP: Vec2 = Vec2::new(0.5, 0.);
pub const PLAYER_BODY_BASE_TOP: Vec2 = Vec2::new(-0.25, 0.4330127);
pub const PLAYER_BODY_BASE_BOTTOM: Vec2 = Vec2::new(-0.25, -0.4330127);

const fn scale_vec2(v: Vec2, scale: f32) -> Vec2 {
    Vec2::new(v.x * scale, v.y * scale)
}

pub const PLAYER_BODY_TIP_SCALED: Vec2 = scale_vec2(PLAYER_BODY_TIP, BASE_SCALE);
pub const PLAYER_BODY_BASE_TOP_SCALED: Vec2 = scale_vec2(PLAYER_BODY_BASE_TOP, BASE_SCALE);
pub const PLAYER_BODY_BASE_BOTTOM_SCALED: Vec2 = scale_vec2(PLAYER_BODY_BASE_BOTTOM, BASE_SCALE);

#[derive(Component)]
#[require(Collider, MeshMaterial2d<ColorMaterial>, Mesh2d, Rotate, Transform)]
pub struct Body;

impl Body {
    fn bundle(parent: Entity, mesh: Handle<Mesh>, color: Handle<ColorMaterial>) -> impl Bundle {
        (
            Body,
            Collider::triangle_unchecked(
                PLAYER_BODY_TIP,
                PLAYER_BODY_BASE_TOP,
                PLAYER_BODY_BASE_BOTTOM,
            ),
            CollisionLayers::new(LayerMasks::Player, LayerMask::ALL),
            DamageTargetReference::new(parent),
            Mesh2d(mesh),
            MeshMaterial2d(color),
            Transform::from_scale(Vec2::splat(BASE_SCALE).extend(1.))
                .with_translation(Vec3::new(0., 0., 0.1)),
        )
    }
}

pub(super) trait SpawnBodyExt {
    fn spawn_body(&mut self, parent: Entity, mesh: Handle<Mesh>, color: Handle<ColorMaterial>);
}

impl SpawnBodyExt for RelatedSpawnerCommands<'_, ChildOf> {
    fn spawn_body(&mut self, parent: Entity, mesh: Handle<Mesh>, color: Handle<ColorMaterial>) {
        self.spawn(Body::bundle(parent, mesh, color));
    }
}

pub(super) struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut color_material_assets: ResMut<Assets<ColorMaterial>>,
) {
    let body_mesh = mesh_assets.add(Triangle2d::new(
        PLAYER_BODY_TIP,
        PLAYER_BODY_BASE_TOP,
        PLAYER_BODY_BASE_BOTTOM,
    ));

    // player body color assets
    let attacker_color = color_material_assets.add(Team::Attackers.get_color());
    let defender_color = color_material_assets.add(Team::Defenders.get_color());
    let dummy_color = color_material_assets.add(Color::srgb(0.6, 0.6, 0.6));

    let player_assets = PlayerAssets {
        body_mesh,
        attacker_color,
        defender_color,
        dummy_color,
    };

    commands.insert_resource(player_assets.clone());
}
