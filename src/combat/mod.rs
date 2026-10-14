use bevy::{app::Plugin, ecs::{component::Component, entity::Entity, message::Message}};

use crate::combat::{health::HealthPlugin, weapon::WeaponPlugin};

pub(crate) mod health;
pub(crate) mod weapon;


pub(super) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            HealthPlugin,
            WeaponPlugin
        ));
    }
}
