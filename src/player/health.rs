use bevy::{app::{FixedUpdate, Plugin}, ecs::{component::Component, entity::Entity, message::{Message, MessageReader, MessageWriter}, query::Without, system::Query}};

use crate::player::DeathMessage;

#[derive(Message, Clone)]
pub(crate) struct DamageMessage {
    pub target: Entity,
    pub amount: u32
}

impl DamageMessage {
    pub fn new(target: Entity, amount: u32) -> Self {
        DamageMessage {
            target,
            amount
        }
    }

    pub fn target(&self) -> Entity {
        self.target
    }
    
    pub fn amount(&self) -> u32 {
        self.amount
    }
}

#[derive(Component)]
pub(super) struct Health(u32);

impl Health {
    pub fn new(amount: u32) -> Self {
        Health(amount) 
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Component)]
pub(super) struct Invencibility;

pub(super) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(FixedUpdate, apply_damage_system);
    }
}

fn apply_damage_system(
    mut health_query: Query<&mut Health, Without<Invencibility>>,
    mut message_reader: MessageReader<DamageMessage>,
    mut message_writer: MessageWriter<DeathMessage>
) {
    for event in message_reader.read() {
        if let Ok(mut health) = health_query.get_mut(event.target) {
            health.0 = health.0.saturating_sub(event.amount);

            if health.0 <= 0 {
                message_writer.write(DeathMessage(event.target));
            }
        }
    }
}
