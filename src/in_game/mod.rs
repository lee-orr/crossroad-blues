mod actions;
mod checkpoints;
mod danger;
mod game_completed;
mod game_over;
mod game_state;
mod generate_level;
mod guardian_angel;
mod holy_hulk;
mod movement;
mod pause_screen;
mod player;
mod ritual;
mod schedule;
mod shadow;
mod souls;
mod stealthy_seraphim;
mod teleport;

use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::StateInspectorPlugin;

use big_brain::{
    prelude::ActionState,
    scorers::Score,
    thinker::{Action, Actor, Scorer, Thinker},
    BigBrainPlugin, BigBrainSet,
};
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::{
    app_state::AppState,
    in_game::{
        checkpoints::checkpoint_plugin, danger::danger_plugin, ritual::ritual_plugin,
        souls::souls_plugin,
    },
};

use self::{
    actions::PlayerAction,
    game_completed::GameCompletedPlugin,
    game_over::GameOverPlugin,
    game_state::{GameState, PauseState},
    generate_level::*,
    movement::*,
    pause_screen::PausePlugin,
    player::*,
    schedule::*,
    shadow::*,
    souls::{Damage, Death},
    teleport::*,
};
use dexterous_developer::{
    dexterous_developer_setup, ReloadableAppContents, ReloadableElementsSetup,
};

pub use player::TrackingCamera;
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<PlayerAction>::default(),
            BigBrainPlugin::new(InGamePreUpdate),
        ))
        .add_plugins((PausePlugin, GameOverPlugin, GameCompletedPlugin))
        .add_state::<GameState>()
        .register_type::<GameState>()
        .register_type::<Thinker>()
        .register_type::<Scorer>()
        .register_type::<Action>()
        .register_type::<Actor>()
        .register_type::<Score>()
        .register_type::<ActionState>()
        .add_event::<Damage>()
        .add_event::<Death>()
        .add_plugins(
            StateInspectorPlugin::<GameState>::default()
                .run_if(input_toggle_active(false, KeyCode::F1)),
        )
        .add_systems(OnExit(AppState::InGame), (exit, clear_audio))
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
            run_in_game_update
                .run_if(in_state(GameState::InGame).and_then(in_state(PauseState::None))),
        )
        .add_systems(
            PreUpdate,
            run_in_game_pre_update
                .run_if(in_state(GameState::InGame).and_then(in_state(PauseState::None))),
        )
        .add_systems(
            PostUpdate,
            run_in_game_post_update
                .run_if(in_state(GameState::InGame).and_then(in_state(PauseState::None))),
        )
        .setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(in_game)]
fn reloadable(app: &mut ReloadableAppContents) {
    player_plugin(app);
    shadow_plugin(app);
    movement_plugin(app);
    souls_plugin(app);
    teleport_plugin(app);
    checkpoint_plugin(app);
    danger_plugin(app);
    level_generate_plugin(app);
    ritual_plugin(app);
}

#[derive(Component)]
struct InGame;

fn exit(mut commands: Commands, _query: Query<Entity, With<InGame>>, _windows: Query<&mut Window>) {
    commands.insert_resource(NextState(Some(GameState::None)));
    commands.insert_resource(NextState(Some(PauseState::None)));
}

fn clear_audio(audio: Query<&AudioSink>) {
    for audio in audio.iter() {
        audio.stop();
    }
}

fn run_in_game_update(world: &mut World) {
    let _ = world.try_run_schedule(InGameUpdate);
}

fn run_in_game_pre_update(world: &mut World) {
    let _ = world.try_run_schedule(InGamePreUpdate);
}

fn run_in_game_post_update(world: &mut World) {
    let _ = world.try_run_schedule(InGamePostUpdate);
}

fn run_in_game_scorers(world: &mut World) {
    let _ = world.try_run_schedule(InGameScorers);
}

fn run_in_game_actions(world: &mut World) {
    let _ = world.try_run_schedule(InGameActions);
}
