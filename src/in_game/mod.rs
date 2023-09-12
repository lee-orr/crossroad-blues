mod actions;
mod checkpoints;
mod devils;
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
};
use bevy_inspector_egui::quick::StateInspectorPlugin;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};

use big_brain::{BigBrainPlugin, BigBrainSet};
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::{
    app_state::AppState,
    assets::MainGameAssets,
    in_game::{checkpoints::checkpoint_plugin, devils::devils_plugin, souls::souls_plugin},
    ui::colors::{DEFAULT_AMBIENT, DEFAULT_CLEAR},
};

use self::{
    actions::PlayerAction,
    checkpoints::Checkpoint,
    devils::LumberingDevil,
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

pub use player::TrackingCamera;
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
        .add_systems(
            PreUpdate,
            run_in_game_scorers
                .run_if(in_state(PauseState::None).and_then(in_state(AppState::InGame)))
                .in_set(BigBrainSet::Scorers),
        )
        .add_systems(
            PreUpdate,
            run_in_game_actions
                .run_if(in_state(PauseState::None).and_then(in_state(AppState::InGame)))
                .in_set(BigBrainSet::Actions),
        )
        .add_systems(
            Update,
            run_in_game_update.run_if(in_state(PauseState::None)),
        )
        .add_systems(
            PreUpdate,
            run_in_game_pre_update.run_if(in_state(PauseState::None)),
        )
        .setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(in_game)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<InGame, _, _>(AppState::InGame, setup);

    player_plugin(app);
    shadow_plugin(app);
    movement_plugin(app);
    souls_plugin(app);
    teleport_plugin(app);
    checkpoint_plugin(app);
    devils_plugin(app);
}

#[derive(Component)]
struct InGame;

fn setup(
    mut commands: Commands,
    assets: Res<MainGameAssets>,
    mut rng: ResMut<GlobalRng>,
    _windows: Query<&mut Window>,
) {
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
                source: assets.game_music.clone(),
                settings: PlaybackSettings {
                    volume: Volume::Absolute(VolumeLevel::new(0.7)),
                    ..Default::default()
                },
            });
            p.spawn((SpatialBundle::default(), ConstructPlayer));

            for _ in 0..15 {
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

            for _ in 0..2 {
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
                    Checkpoint,
                ));
            }

            for _ in 0..2 {
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
                    LumberingDevil,
                ));
            }
        });
}

fn exit(mut commands: Commands, query: Query<Entity, With<InGame>>, _windows: Query<&mut Window>) {
    commands.insert_resource(NextState(Some(GameState::None)));
    commands.insert_resource(NextState(Some(PauseState::None)));
    for item in query.iter() {
        commands.entity(item).despawn_recursive();
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

fn run_in_game_scorers(world: &mut World) {
    let _ = world.try_run_schedule(InGameScorers);
}

fn run_in_game_actions(world: &mut World) {
    let _ = world.try_run_schedule(InGameActions);
}
