use avian2d::collision::collider::Collider;

use bevy::{
    app::{Plugin, Startup}, asset::{Assets, Handle}, color::Color, ecs::{
        bundle::Bundle, component::Component, event::Trigger, hierarchy::ChildOf, lifecycle::Add, observer::On, relationship::RelatedSpawnerCommands, system::{Commands, ResMut},
    }, math::{Vec2, Vec3, primitives::Triangle2d}, mesh::{Mesh, Mesh2d}, sprite_render::{ColorMaterial, MeshMaterial2d}, transform::components::Transform,
};

use crate::{
    player::{PlayerAssets, Rotate},
    Team,
};

const PLAYER_BODY_TIP: Vec2 = Vec2::new(0.5, 0.);
const PLAYER_BODY_BASE_TOP: Vec2 = Vec2::new(-0.25, 0.4330127);
const PLAYER_BODY_BASE_BOTTOM: Vec2 = Vec2::new(-0.25, -0.4330127);

#[derive(Component)]
#[require(Collider, MeshMaterial2d<ColorMaterial>, Mesh2d, Rotate, Transform)]
pub(super) struct Body;

impl Body {
    fn bundle(mesh: Handle<Mesh>, color: Handle<ColorMaterial>) -> impl Bundle {
        (
            Body,
            Collider::triangle_unchecked(
                PLAYER_BODY_TIP,
                PLAYER_BODY_BASE_TOP,
                PLAYER_BODY_BASE_BOTTOM,
            ),
            Mesh2d(mesh),
            MeshMaterial2d(color),
            Transform::from_scale(Vec2::splat(32.).extend(1.))
                .with_translation(Vec3::new(0., 0., 0.1)),
        )
    }
}

pub(super) trait SpawnBodyExt {
    fn spawn_body(&mut self, mesh: Handle<Mesh>, color: Handle<ColorMaterial>);
}

impl SpawnBodyExt for RelatedSpawnerCommands<'_, ChildOf> {
    fn spawn_body(&mut self, mesh: Handle<Mesh>, color: Handle<ColorMaterial>) {
        self.spawn(Body::bundle(mesh, color));
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
