use bevy::prelude::*;

pub fn spawn_bundle_default_on_added<Add: Component, Bun: Bundle + Default>(
    mut commands: Commands,
    query: Query<Entity, Added<Add>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Bun::default());
    }
}
