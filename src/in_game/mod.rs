mod actions;
mod game_completed;
mod game_over;
mod game_state;
mod movement;
mod pause_screen;
mod player;
mod schedule;
mod shadow;
mod souls;
mod teleport;

use bevy::{
    audio::{Volume, VolumeLevel},
    input::common_conditions::input_toggle_active,
    prelude::*,
    window::CursorGrabMode,
};
use bevy_inspector_egui::quick::StateInspectorPlugin;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};

use big_brain::BigBrainPlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::{
    app_state::AppState,
    assets::MainGameAssets,
    in_game::souls::{souls_plugin, sun_sensitivity, take_damage},
    ui::colors::{DEFAULT_AMBIENT, DEFAULT_CLEAR},
};

use self::{
    actions::PlayerAction,
    game_completed::GameCompletedPlugin,
    game_over::GameOverPlugin,
    game_state::{GameState, PauseState},
    movement::*,
    pause_screen::PausePlugin,
    player::*,
    schedule::*,
    shadow::*,
    souls::{Damage, Death},
    teleport::*,
};
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<PlayerAction>::default(),
            BigBrainPlugin::new(InGamePreUpdate),
            seldom_state::StateMachinePlugin,
        ))
        .add_plugins((PausePlugin, GameOverPlugin, GameCompletedPlugin))
        .add_state::<GameState>()
        .register_type::<GameState>()
        .add_event::<Damage>()
        .add_event::<Death>()
        .add_plugins(
            StateInspectorPlugin::<GameState>::default()
                .run_if(input_toggle_active(false, KeyCode::F1)),
        )
        .add_systems(OnExit(AppState::InGame), (exit, clear_audio))
        .add_systems(Update, (enable_audio).run_if(in_state(AppState::InGame)))
        .setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(in_game)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<InGame, _, _>(AppState::InGame, setup)
        .add_systems(
            Update,
            run_in_game_update.run_if(in_state(PauseState::None)),
        )
        .add_systems(
            PreUpdate,
            run_in_game_pre_update.run_if(in_state(PauseState::None)),
        );

    player_plugin(app);
    shadow_plugin(app);
    movement_plugin(app);
    souls_plugin(app);
    teleport_plugin(app);
}

#[derive(Component)]
struct InGame;

fn setup(
    mut commands: Commands,
    assets: Res<MainGameAssets>,
    mut rng: ResMut<GlobalRng>,
    mut windows: Query<&mut Window>,
) {
    for mut window in windows.iter_mut() {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Confined;
    }

    let rng = rng.get_mut();
    commands.insert_resource(ClearColor(DEFAULT_CLEAR));
    commands.insert_resource(DEFAULT_AMBIENT);
    commands
        .spawn((
            InGame,
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .with_children(|p| {
            p.spawn(AudioBundle {
                source: assets.menu_music.clone(),
                settings: PlaybackSettings {
                    paused: true,
                    volume: Volume::Absolute(VolumeLevel::new(0.)),
                    ..Default::default()
                },
            });
            p.spawn((SpatialBundle::default(), ConstructPlayer));

            for _ in 0..5 {
                let pos = Vec3::new(
                    rng.f32_normalized() * 300.,
                    rng.f32_normalized() * 300.,
                    -5.,
                );
                p.spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(pos),
                        ..Default::default()
                    },
                    Shadow {
                        radius: rng.f32_normalized().abs() * 50. + 20.,
                    },
                ));
            }
        });
}

fn exit(
    mut commands: Commands,
    query: Query<Entity, With<InGame>>,
    mut windows: Query<&mut Window>,
) {
    commands.insert_resource(NextState(Some(GameState::None)));
    commands.insert_resource(NextState(Some(PauseState::None)));
    for item in query.iter() {
        commands.entity(item).despawn_recursive();
    }
    for mut window in windows.iter_mut() {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn clear_audio(audio: Query<&AudioSink>) {
    for audio in audio.iter() {
        audio.stop();
    }
}

fn enable_audio(audio: Query<&AudioSink>) {
    for audio in audio.iter() {
        if audio.is_paused() {
            audio.play();
        }
    }
}

fn run_in_game_update(world: &mut World) {
    let _ = world.try_run_schedule(InGameUpdate);
}

fn run_in_game_pre_update(world: &mut World) {
    let _ = world.try_run_schedule(InGamePreUpdate);
}
