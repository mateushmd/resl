use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run(); 
}

#[derive(Component, Default)]
struct PlayerCollider;

#[derive(Component)]
pub enum Team {
    Attackers,
    Defenders
}

impl Team {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Attackers => Color::srgba(1.0, 0.0, 0.0, 1.0),
            Self::Defenders => Color::srgba(0.0, 0.0, 1.0, 1.0)
        }
    }
}

#[derive(Component)]
#[require(MeshMaterial2d<ColorMaterial>, Mesh2d, PlayerCollider, Transform)]
struct Player(Team);

impl Player {
    pub fn new(
        mut meshes: ResMut<Assets<Mesh>>, 
        mut color_materials: ResMut<Assets<ColorMaterial>>,
        team: Team
    ) -> (Player, MeshMaterial2d<ColorMaterial>, Mesh2d, PlayerCollider, Transform) {
        let team_color = team.get_color();

        (
            Player(team),
            MeshMaterial2d(color_materials.add(team_color)),
            Mesh2d(meshes.add(Triangle2d::default())),
            PlayerCollider,
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                .with_scale(Vec2::splat(32.0).extend(1.0))
        )
    }
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(Camera2d); 
    commands.spawn(Player::new(meshes, color_materials, Team::Attackers));
}
