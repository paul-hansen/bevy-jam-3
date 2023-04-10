mod confirm_quit;
mod create_game;
mod focus;
mod join_by_ip;
mod lobby_browser;
mod main_menu;
mod pre_game;

use crate::game_manager::{GameState, Persist};
use crate::network::commands::Disconnect;
use crate::network::matchmaking::{MatchmakingState, ServerList};
use crate::network::{is_client, is_server};
use crate::ui::confirm_quit::{confirm_quit_to_menu_update, setup_confirm_quit};
use crate::ui::create_game::draw_create_game;
use crate::ui::focus::ui_focus_system;
use crate::ui::join_by_ip::draw_join_by_ip;
use crate::ui::lobby_browser::{handle_join_game_click, setup_lobby_browser, update_lobby_browser};
use crate::ui::main_menu::setup_main_menu;
use crate::ui::pre_game::setup_pre_game;
use crate::MainCamera;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::core_2d;
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget, ScalingMode};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::ui::UiSystem;
use bevy::window::{PrimaryWindow, WindowResized};

pub struct UiPlugin;

/// This is used as a state to determine the currently open menu, and a component to identify the
/// root node of each menu
#[derive(States, Hash, Eq, PartialEq, Copy, Clone, Default, Debug, Reflect, Component)]
pub enum Menu {
    #[default]
    Hidden,
    Main,
    CreateGame,
    JoinByIP,
    LobbyBrowser,
    PreGame,
    ConfirmQuitToMain,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<Menu>();
        app.register_type::<UiCamera>();
        app.register_type::<UiSprite>();
        app.register_type::<MenuUiRoot>();
        app.register_type::<MenuUiContainer>();
        app.register_type::<Menu>();
        app.register_type::<ChangeStateOnClick<Menu>>();
        app.add_system(change_state_on_click::<Menu>);
        app.register_type::<ChangeStateOnClick<GameState>>();
        app.add_system(change_state_on_click::<GameState>);
        app.add_system(toggle_menu);
        app.add_system(command_on_click::<Disconnect>);
        app.add_system(
            ui_focus_system
                .in_set(UiSystem::Focus)
                .after(bevy::ui::ui_focus_system),
        );
        app.add_system(open_main_menu.in_schedule(OnEnter(GameState::MainMenu)));

        // Show the pregame ui for servers
        app.add_system(
            open_pregame
                .run_if(is_server())
                .in_schedule(OnEnter(GameState::PreGame)),
        );
        // Clients pregame step doesn't currently progress, hide the menus for now
        app.add_system(
            hide_menu
                .run_if(is_client())
                .in_schedule(OnExit(GameState::MainMenu)),
        );

        app.add_system(hide_menu.in_schedule(OnExit(GameState::PreGame)));

        app.add_system(update_menu_display);
        app.add_systems((
            handle_join_game_click,
            update_lobby_browser.run_if(run_once()),
            update_lobby_browser
                .run_if(resource_changed::<ServerList>())
                .in_set(OnUpdate(Menu::LobbyBrowser)),
        ));

        app.add_systems((draw_create_game.in_set(OnUpdate(Menu::CreateGame)),));
        app.add_systems((draw_join_by_ip.in_set(OnUpdate(Menu::JoinByIP)),));

        app.add_system(
            confirm_quit_to_menu_update
                .run_if(resource_changed::<MatchmakingState>())
                .in_set(OnUpdate(Menu::ConfirmQuitToMain)),
        );

        app.add_startup_systems(
            (
                setup_menu_container,
                // Need to ensure MenuUiRoot is spawned
                apply_system_buffers,
                setup_main_menu,
                setup_lobby_browser,
                setup_confirm_quit,
                setup_pre_game,
            )
                .chain(),
        );
        app.add_startup_systems((setup_ui_camera, resize_ui));
        app.add_system(resize_ui.run_if(on_event::<WindowResized>()));
        app.add_system(button_hover);
    }
}

fn open_main_menu(mut menu_state: ResMut<NextState<Menu>>) {
    menu_state.set(Menu::Main);
}

fn open_pregame(mut menu_state: ResMut<NextState<Menu>>) {
    menu_state.set(Menu::PreGame);
}

fn hide_menu(mut menu_state: ResMut<NextState<Menu>>) {
    menu_state.set(Menu::Hidden);
}

fn update_menu_display(
    menu_state: Res<State<Menu>>,
    mut menus: Query<(&Menu, &mut Style), Without<MenuUiRoot>>,
    mut root: Query<&mut Style, With<MenuUiRoot>>,
) {
    if let Ok(mut root_style) = root.get_single_mut() {
        root_style.display = if menu_state.0 == Menu::Hidden {
            Display::None
        } else {
            Display::Flex
        }
    }

    for (menu, mut style) in menus.iter_mut() {
        style.display = if *menu == menu_state.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// The topmost part of the menu ui.
/// You might want [`MenuUiContainer`] instead if you want to add things to the UI.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MenuUiRoot;

/// This is inside the menu's borders and is where you should add things to the UI.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MenuUiContainer;

fn setup_menu_container(mut commands: Commands) {
    commands
        .spawn((
            Name::new("MenuUiRoot"),
            MenuUiRoot,
            Persist,
            NodeBundle {
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                style: Style {
                    display: Display::None,
                    size: Size::all(Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|child_builder| {
            child_builder
                .spawn((NodeBundle {
                    background_color: BackgroundColor::from(Color::GREEN),
                    style: Style {
                        padding: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|child_builder| {
                    child_builder.spawn((
                        Name::new("MenuUiContainer"),
                        MenuUiContainer,
                        NodeBundle {
                            background_color: BackgroundColor::from(Color::BLACK),
                            style: Style {
                                padding: UiRect::all(Val::Px(6.0)),
                                size: Size::all(Val::Percent(100.0)),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                });
        });
}

fn toggle_menu(
    key_codes: Res<Input<KeyCode>>,
    game_state: Res<State<GameState>>,
    menu_state: Res<State<Menu>>,
    mut next_menu_state: ResMut<NextState<Menu>>,
) {
    if key_codes.just_released(KeyCode::Escape) {
        match game_state.0 {
            GameState::PreGame | GameState::Playing | GameState::PostGame => {
                next_menu_state.set(match menu_state.0 {
                    Menu::Hidden => Menu::ConfirmQuitToMain,
                    _ => Menu::Hidden,
                })
            }
            _ => {}
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct UiCamera;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct UiSprite;

fn setup_ui_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);
    commands.spawn((
        UiSprite,
        Persist,
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 999.0),
            texture: image_handle.clone(),
            ..default()
        },
    ));

    // Uncomment to show
    // Useful for debugging UI viewport size

    // commands.spawn(NodeBundle {
    //     background_color: BackgroundColor(Color::rgba_u8(255, 0, 0, 60)),
    //     style: Style {
    //         size: Size::all(Val::Percent(99.0)),
    //         ..default()
    //     },
    //     ..default()
    // });

    commands.spawn((
        UiCamera,
        Persist,
        Camera2dBundle {
            // position the UI camera far away so it can't see itself or other sprites.
            transform: Transform::from_xyz(9999.0, 9999.0, 9999.0),
            camera_render_graph: CameraRenderGraph::new(core_2d::graph::NAME),
            camera: Camera {
                target: RenderTarget::Image(image_handle),
                ..default()
            },
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::WindowSize(1.0),
                ..Default::default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::NONE),
            },
            ..default()
        },
        UiCameraConfig { show_ui: true },
    ));
}

fn resize_ui(
    ui_camera_query: Query<&Camera, With<UiCamera>>,
    mut ui_sprite: Query<&mut Sprite, With<UiSprite>>,
    mut images: ResMut<Assets<Image>>,
    mut asset_events: EventWriter<AssetEvent<Image>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    main_camera: Query<&Camera, (With<MainCamera>, Without<UiCamera>)>,
) {
    let window = primary_window.single();
    let width = window.width();
    let height = window.height();
    let Ok(main_camera) = main_camera.get_single() else {
        return;
    };

    let viewport_world_size = main_camera
        .viewport_to_world_2d(&GlobalTransform::default(), Vec2::ZERO)
        .map(|d| Vec2::new(-d.x, -d.y) * 2.0)
        .unwrap_or(Vec2::splat(10.0));

    for camera in ui_camera_query.iter() {
        if let RenderTarget::Image(handle) = &camera.target {
            if let Some(image) = images.get_mut(handle) {
                image.resize(Extent3d {
                    width: width as u32,
                    height: height as u32,
                    ..default()
                });
                asset_events.send(AssetEvent::Modified {
                    handle: handle.clone(),
                });

                for mut sprite in ui_sprite.iter_mut() {
                    // Make the ui sprite fullscreen.
                    // Gets the top left corner of the viewport in world space, since the camera is
                    // centered, this is half the camera size in world space.
                    // Probably an easier way to do this but this is the first way I found that worked.
                    sprite.custom_size = Some(viewport_world_size);
                }
            }
        }
    }
}

pub fn button_hover(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    children: Query<&Children>,
    mut texts: Query<&mut Text>,
) {
    for (entity, interaction) in query.iter() {
        match interaction {
            Interaction::Clicked => {}
            Interaction::Hovered => {
                change_button_text_color(entity, &children, &mut texts, Color::ORANGE);
            }
            Interaction::None => {
                change_button_text_color(entity, &children, &mut texts, Color::RED);
            }
        }
    }
}

pub fn change_button_text_color(
    button: Entity,
    children: &Query<&Children>,
    texts: &mut Query<&mut Text>,
    color: Color,
) {
    children.iter_descendants(button).for_each(|e| {
        if let Ok(mut text) = texts.get_mut(e) {
            text.sections
                .iter_mut()
                .for_each(|section| section.style.color = color);
        }
    });
}

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
pub struct ChangeStateOnClick<S: States> {
    pub state: S,
}

fn change_state_on_click<S: States>(
    query: Query<(&Interaction, &ChangeStateOnClick<S>), Changed<Interaction>>,
    mut next_state: ResMut<NextState<S>>,
) {
    for (interaction, change_state) in query.iter() {
        if interaction == &Interaction::Clicked {
            next_state.set(change_state.state.clone());
        }
    }
}

#[derive(Component)]
pub struct CommandOnClick<C: Command + Send + Sync + Clone> {
    pub command: C,
}

fn command_on_click<C: Command + Send + Sync + Clone>(
    mut commands: Commands,
    query: Query<(&Interaction, &CommandOnClick<C>), Changed<Interaction>>,
) {
    for (interaction, change_state) in query.iter() {
        if interaction == &Interaction::Clicked {
            commands.add(change_state.command.clone())
        }
    }
}
