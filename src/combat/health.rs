use bevy::{app::{FixedUpdate, Plugin}, ecs::{component::Component, entity::Entity, message::{Message, MessageReader, MessageWriter}, query::Without, system::Query}};

#[derive(Component, Debug)]
pub(crate) struct DamageTargetReference(Entity);

impl DamageTargetReference {
    pub fn new(target: Entity) -> Self {
        Self(target)
    }

    pub fn target(&self) -> Entity {
        self.0
    }
}

impl Into<Entity> for DamageTargetReference {
    fn into(self) -> Entity {
        self.0
    }
}

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

#[derive(Message, Clone)]
pub(crate) struct DeathMessage(Entity);

impl DeathMessage {
    pub fn new(target: Entity) -> Self {
        Self(target)
    }

    pub fn target(&self) -> Entity {
        self.0
    }
}

#[derive(Component)]
pub(crate) struct Health(u32);

impl Health {
    pub fn new(amount: u32) -> Self {
        Health(amount) 
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Component)]
pub(crate) struct Invencibility;

pub(super) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_message::<DamageMessage>()
            .add_message::<DeathMessage>()
            .add_systems(FixedUpdate, apply_damage_system);
    }
}

fn apply_damage_system(
    damage_target_query: Query<&DamageTargetReference>,
    mut health_query: Query<&mut Health, Without<Invencibility>>,
    mut message_reader: MessageReader<DamageMessage>,
    mut message_writer: MessageWriter<DeathMessage>
) {
    for event in message_reader.read() {
        if let Ok(reference) = damage_target_query.get(event.target) {
            let target = reference.target();
            if let Ok(mut health) = health_query.get_mut(target) {
                health.0 = health.0.saturating_sub(event.amount);

                if health.0 <= 0 {
                    message_writer.write(DeathMessage(target));
                }
            }
        }
    }
}
