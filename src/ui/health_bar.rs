use crate::health::Health;
use crate::network::NetworkOwner;
use crate::player::Player;
use bevy::prelude::*;
use bevy_replicon::prelude::{RenetClient, RenetServer};
use bevy_replicon::server::SERVER_ID;

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
pub struct HealthBar;

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
pub struct HealthBarBackground;

pub fn setup_health_bar(mut commands: Commands) {
    commands
        .spawn((
            HealthBarBackground,
            NodeBundle {
                background_color: BackgroundColor::from(Color::rgb_u8(125, 0, 0)),
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
            },
        ))
        .with_children(|cb| {
            cb.spawn((
                HealthBar,
                NodeBundle {
                    background_color: BackgroundColor::from(Color::RED),
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
    query: Query<(&NetworkOwner, &Health, &Player)>,
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
    mut health_bar: Query<(&mut Style, &mut BackgroundColor), With<HealthBar>>,
    mut health_bar_background: Query<
        &mut BackgroundColor,
        (With<HealthBarBackground>, Without<HealthBar>),
    >,
) {
    if let Some(client_id) = server
        .map(|_| SERVER_ID)
        .or_else(|| client.map(|c| c.client_id()))
    {
        if let Some((_, health, player)) = query.iter().find(|(owner, _, _)| owner.0 == client_id) {
            for (mut bar, mut background) in health_bar.iter_mut() {
                bar.size.width = Val::Percent((health.current / health.max) * 100.0);
                background.0 = player.color.color();
            }
            for mut bg in health_bar_background.iter_mut() {
                let color = player.color.color().as_hsla_f32();

                bg.0 = Color::hsl(color[0], color[1] * 0.6, color[2] * 0.3);
            }
        }
    };
}
