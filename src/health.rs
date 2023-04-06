use crate::asteroid::Asteroid;
use crate::player::weapons::DamagedEvent;
use crate::player::Player;
use bevy::prelude::*;
use bevy_replicon::prelude::AppReplicationExt;
use bevy_replicon::server::ServerSet;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>();
        app.add_event::<DeathEvent>();
        // Replicating health so the client can show UI related to their health.
        app.replicate::<Health>();
        app.add_system(update_health_on_damage.in_set(ServerSet::Authority));
        app.add_system(despawn_on_death::<Asteroid>.in_set(ServerSet::Authority));
        app.add_system(despawn_on_death::<Player>.in_set(ServerSet::Authority));
    }
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug)]
pub struct DeathEvent {
    pub entity: Entity,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

fn update_health_on_damage(
    mut query: Query<&mut Health>,
    mut damage_events: EventReader<DamagedEvent>,
    mut death_events: EventWriter<DeathEvent>,
) {
    for event in damage_events.iter() {
        if let Ok(mut health) = query.get_mut(event.entity) {
            health.current -= event.amount;
            health.current = health.current.max(0.0);

            if health.current <= 0.0 {
                death_events.send(DeathEvent {
                    entity: event.entity,
                });
            }
        }
    }
}

fn despawn_on_death<C: Component>(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    query: Query<&C>,
) {
    for event in death_events.iter() {
        if query.contains(event.entity) {
            commands.entity(event.entity).despawn_recursive();
        }
    }
}
