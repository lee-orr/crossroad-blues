#![allow(clippy::type_complexity)]
#![feature(iter_map_windows)]
mod app_state;
mod assets;
mod in_game;
mod menus;
mod toon_material;
mod ui;

use std::time::Duration;

use app_state::AppState;
use assets::{MainGameAssetPlugin, MainGameAssets};
use bevy::{
    asset::ChangeWatcher,
    core_pipeline::{clear_color::ClearColorConfig, tonemapping::Tonemapping},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_toggle_active,
    prelude::*,
};

use bevy_inspector_egui::quick::{StateInspectorPlugin, WorldInspectorPlugin};
use bevy_sequential_actions::SequentialActionsPlugin;
use bevy_turborand::prelude::RngPlugin;
use bevy_tweening::TweeningPlugin;
use bevy_vector_shapes::Shape2dPlugin;
use credits::CreditsPlugin;
use dexterous_developer::{hot_bevy_main, InitialPlugins};
use in_game::{InGamePlugin, TrackingCamera};
use loading_state::LoadingScreenPlugin;
use menu::MainMenuPlugin;
use menus::{credits, loading_state, menu};

use ui::{colors::DEFAULT_AMBIENT, UiPlugin};

use crate::app_state::DrawDebugGizmos;

#[hot_bevy_main]
fn bevy_main(initial: impl InitialPlugins) {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins((
            initial
                .initialize::<DefaultPlugins>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_secs_f32(0.5)),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            Shape2dPlugin::default(),
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F1)),
            RngPlugin::default(),
            TweeningPlugin,
            SequentialActionsPlugin,
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(ClearColor(ui::colors::SCREEN_BACKGROUND_COLOR))
        .insert_resource(DEFAULT_AMBIENT)
        .init_resource::<DrawDebugGizmos>()
        .add_plugins((
            LoadingScreenPlugin,
            MainMenuPlugin,
            CreditsPlugin,
            InGamePlugin,
            MainGameAssetPlugin,
            UiPlugin,
        ))
        .add_state::<AppState>()
        .register_type::<AppState>()
        .register_type::<MainGameAssets>()
        .add_plugins(
            StateInspectorPlugin::<AppState>::default()
                .run_if(input_toggle_active(false, KeyCode::F1)),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_gizmos)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(ui::colors::SCREEN_BACKGROUND_COLOR),
            },
            tonemapping: Tonemapping::None,
            projection: OrthographicProjection {
                scale: 0.5,
                ..Default::default()
            },
            ..default()
        },
        TrackingCamera::default(),
    ));
}

fn toggle_gizmos(mut commands: Commands, gizmos: Res<DrawDebugGizmos>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::F2) {
        let gizmos = match gizmos.as_ref() {
            DrawDebugGizmos::None => DrawDebugGizmos::Collision,
            DrawDebugGizmos::Collision => DrawDebugGizmos::None,
        };
        commands.insert_resource(gizmos);
    }
}
