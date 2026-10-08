use avian2d::{PhysicsPlugins, debug_render::PhysicsDebugPlugin};
use bevy::{DefaultPlugins, app::{App, PluginGroup, PreUpdate, Startup}, utils::default, window::{Window, WindowPlugin}};
use resl::RESLPlugin;

fn main() {
    App::new()
        
        // default plugins
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Reinforced Learning"),
                    ..default()
                }),
                ..default()
            }).build(),
        )

        // avian2d plugins
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default()
        ))

        // RESL plugin
        .add_plugins(RESLPlugin)

        // custom plugins

        .run(); 
}
