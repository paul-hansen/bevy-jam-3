mod common_systems;
mod confirm_quit;
mod lobby_browser;
mod main_menu;

use crate::game_manager::{GameState, Persist};
use crate::ui::common_systems::{set_display_flex, set_display_none};
use crate::ui::confirm_quit::{confirm_quit_to_menu_update, setup_confirm_quit, ConfirmQuit};
use crate::ui::lobby_browser::{setup_lobby_browser, LobbyBrowser};
use crate::ui::main_menu::{setup_main_menu, MainMenu};
use crate::MainCamera;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::core_2d;
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget, ScalingMode};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::window::{PrimaryWindow, WindowResized};

pub struct UiPlugin;

#[derive(States, Hash, Eq, PartialEq, Copy, Clone, Default, Debug, Reflect)]
pub enum MenuState {
    #[default]
    Hidden,
    Main,
    LobbyBrowser,
    ConfirmQuitToMain,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>();
        app.register_type::<UiCamera>();
        app.register_type::<UiSprite>();
        app.register_type::<MenuUiRoot>();
        app.register_type::<MenuUiContainer>();
        app.register_type::<MainMenu>();
        app.add_system(toggle_menu);

        // Menu root
        app.add_system(set_display_none::<MenuUiRoot>.in_schedule(OnEnter(MenuState::Hidden)));
        app.add_system(set_display_flex::<MenuUiRoot>.in_schedule(OnExit(MenuState::Hidden)));

        // Main menu
        app.add_system(set_display_flex::<MainMenu>.in_schedule(OnEnter(MenuState::Main)));
        app.add_system(set_display_none::<MainMenu>.in_schedule(OnExit(MenuState::Main)));

        // Confirm Quit
        app.add_system(
            set_display_flex::<ConfirmQuit>.in_schedule(OnEnter(MenuState::ConfirmQuitToMain)),
        );
        app.add_system(
            set_display_none::<ConfirmQuit>.in_schedule(OnExit(MenuState::ConfirmQuitToMain)),
        );
        app.add_system(confirm_quit_to_menu_update.in_set(OnUpdate(MenuState::ConfirmQuitToMain)));

        // Lobby browser
        app.add_system(
            set_display_flex::<LobbyBrowser>.in_schedule(OnEnter(MenuState::LobbyBrowser)),
        );
        app.add_system(
            set_display_none::<LobbyBrowser>.in_schedule(OnExit(MenuState::LobbyBrowser)),
        );

        app.add_startup_systems(
            (
                setup_menu_container,
                // Need to ensure MenuUiRoot is spawned
                apply_system_buffers,
                setup_main_menu,
                setup_lobby_browser,
                setup_confirm_quit,
            )
                .chain(),
        );
        app.add_startup_systems((setup_ui_camera, resize_ui));
        app.add_system(resize_ui.run_if(on_event::<WindowResized>()));
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
                        padding: UiRect::all(Val::Px(4.0)),
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
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if key_codes.just_released(KeyCode::Escape) {
        match game_state.0 {
            GameState::PreGame | GameState::Playing | GameState::PostGame => {
                println!("test {menu_state:?}");
                next_menu_state.set(match menu_state.0 {
                    MenuState::Hidden => MenuState::ConfirmQuitToMain,
                    _ => MenuState::Hidden,
                })
            }
            _ => {
                println!("test {:?}", game_state.0);
            }
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
    println!("{} - {width} {height}", viewport_world_size);

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
