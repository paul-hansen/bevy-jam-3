use bevy::prelude::*;

pub fn spawn_bundle_default_on_added<Add: Component, Bun: Bundle + Default>(
    mut commands: Commands,
    query: Query<Entity, Added<Add>>,
) {
    for entity in query.iter() {
        let Some(mut entcmds) = commands.get_entity(entity) else{
            warn!("Could not find entity to insert bundle into");
            return;
        };

        entcmds.insert(Bun::default());
    }
}
