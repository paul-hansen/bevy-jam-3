use crate::health::Health;
use crate::network::NetworkOwner;
use bevy::prelude::*;
use bevy_replicon::prelude::{RenetClient, RenetServer};
use bevy_replicon::server::SERVER_ID;

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
pub struct HealthBar;

pub fn setup_health_bar(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            background_color: BackgroundColor::from(Color::RED),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(40.0),
                    right: Val::Auto,
                    top: Val::Px(30.0),
                    bottom: Default::default(),
                },

                size: Size {
                    width: Val::Percent(20.0),
                    height: Val::Percent(1.0),
                },
                ..default()
            },
            ..default()
        })
        .with_children(|cb| {
            cb.spawn((
                HealthBar,
                NodeBundle {
                    background_color: BackgroundColor::from(Color::GREEN),
                    style: Style {
                        size: Size::all(Val::Percent(100.0)),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

pub fn update_health_bar(
    query: Query<(&NetworkOwner, &Health)>,
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
    mut health_bar: Query<&mut Style, With<HealthBar>>,
) {
    if let Some(client_id) = server
        .map(|_| SERVER_ID)
        .or_else(|| client.map(|c| c.client_id()))
    {
        if let Some((_, health)) = query.iter().find(|(owner, _)| owner.0 == client_id) {
            for mut bar in health_bar.iter_mut() {
                bar.size.width = Val::Percent((health.current / health.max) * 100.0);
            }
        }
    };
}
