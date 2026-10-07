use avian2d::{
    PhysicsPlugins, 
    collision::collider::Collider, 
    debug_render::PhysicsDebugPlugin, 
    dynamics::rigid_body::{
        RigidBody,
        LinearVelocity
    }
};

use bevy::{
    DefaultPlugins, app::{
        App, Startup, Update
    }, asset::Assets, camera::Camera2d, color::Color, ecs::{
        bundle::Bundle, component::Component, query::With, system::{
            Commands, Query, Res, ResMut, Single
        }
    }, input::{ButtonInput, keyboard::KeyCode}, math::{
        Vec2, 
        primitives::Triangle2d
    }, mesh::{
        Mesh, Mesh2d
    }, sprite_render::{
        ColorMaterial, 
        MeshMaterial2d
    }, time::Time, transform::components::Transform
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default()
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, player_controller)
        .run(); 
}

#[derive(Component)]
pub enum Team {
    Attackers,
    Defenders
}

impl Team {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Attackers => Color::srgba(1., 0., 0., 1.),
            Self::Defenders => Color::srgba(0., 0., 1., 1.)
        }
    }
}

#[derive(Component)]
#[require(Collider, MeshMaterial2d<ColorMaterial>, Mesh2d, RigidBody, Transform)]
struct Player(Team);

impl Player {
    pub fn new(
        meshes: &mut Assets<Mesh>, 
        color_materials: &mut Assets<ColorMaterial>,
        position: Vec2,
        team: Team
    ) -> impl Bundle {
        let team_color = team.get_color();

        (
            Player(team),
            RigidBody::Kinematic,
            Collider::triangle_unchecked(
                Vec2::new(0., 0.5),
                Vec2::new(-0.5, -0.5), 
                Vec2::new(0.5, -0.5)
            ),
            MeshMaterial2d(color_materials.add(team_color)),
            Mesh2d(meshes.add(Triangle2d::default())),
            Transform::from_translation(position.extend(1.))
                .with_scale(Vec2::splat(32.).extend(1.))
        )
    }
}

fn player_controller(
    input: Res<ButtonInput<KeyCode>>,
    mut player_velocity: Single<&mut LinearVelocity, With<Player>>
) {
    let mut x = 0.;
    let mut y = 0.;

    if input.pressed(KeyCode::KeyA) {
        x += -1.;
    } 

    if input.pressed(KeyCode::KeyD) {
       x += 1.;
    }

    if input.pressed(KeyCode::KeyW) {
        y += 1.;
    }

    if input.pressed(KeyCode::KeyS) {
        y += -1.;
    }

    let direction = Vec2::new(x, y).normalize_or_zero();
    
    player_velocity.0 = direction * 100. * if input.pressed(KeyCode::ShiftLeft) { 1.5 } else { 1. };
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(Camera2d); 
    commands.spawn(Player::new(&mut meshes, &mut color_materials, Vec2::ZERO, Team::Attackers));
}
