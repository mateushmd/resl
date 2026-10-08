use avian2d::{debug_render::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::{
    app::{App, PluginGroup, PreUpdate, Startup},
    utils::default,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use resl::RESLPlugin;

fn main() {
    App::new()
        // default plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Reinforced Learning"),
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        // avian2d plugins
        .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
        // RESL plugin
        .add_plugins(RESLPlugin)
        // custom plugins
        .run();
}
