use bevy::prelude::*;

use bevy_ui_dsl::*;

use crate::{
    app_state::AppState,
    assets::WithMesh,
    ui::{
        buttons::{focus_text_button, focused_button_activated, TypedFocusedButtonQuery},
        classes::*,
        colors::SCREEN_BACKGROUND_COLOR,
        intermediary_node_bundles::*,
    },
    CurrentLevelID,
};

use super::{
    checkpoints::CheckpointCollected, game_state::GameState,
    player::CheckpointsConsumedForTeleport, InGame, Levels, TrackingCamera,
};
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};
pub struct GameCompletedPlugin;

impl Plugin for GameCompletedPlugin {
    fn build(&self, app: &mut App) {
        app.setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(game_completed)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<Screen, _, _>(GameState::Complete, setup)
        .add_systems(
            Update,
            (
                process_keyboard_input,
                (focused_button_activated.pipe(process_input)),
            )
                .run_if(in_state(GameState::Complete)),
        );
}

#[derive(Component)]
struct Screen;

#[derive(Component)]
enum Button {
    Menu,
    NextLevel,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players: Query<(&CheckpointCollected, &CheckpointsConsumedForTeleport)>,
    in_game: Query<Entity, With<InGame>>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
    levels: Res<Levels>,
    current_level_id: Res<CurrentLevelID>,
) {
    commands.insert_resource(ClearColor(SCREEN_BACKGROUND_COLOR));
    let mut menu_button = None;
    let mut level_button = None;
    let r = root(c_root, &asset_server, &mut commands, |p| {
        node(primary_box, p, |p| {
            node((span.nb(), primary_box_main.nb()), p, |p| {
                text("You got the Deal!", (), main_text, p);
            });

            for (collected, consumed) in &players {
                let collected = collected.0;
                let consumed = consumed.0;
                let collected_score = 10 * collected;
                let consumed_score = 5 * consumed;
                let score = collected_score + consumed_score;

                node((span.nb(), primary_box_item.nb()), p, |p| {
                    text(
                        format!("Collected Checkpoints: {collected} X 10 = {collected_score}"),
                        (),
                        standard_text,
                        p,
                    );
                });
                node((span.nb(), primary_box_item.nb()), p, |p| {
                    text(
                        format!("Consumed Checkpoints: {consumed} X 5 = {consumed_score}"),
                        (),
                        standard_text,
                        p,
                    );
                });

                node((span.nb(), primary_box_item.nb()), p, |p| {
                    text(format!("Score: {score}!"), (), main_text, p);
                });
            }

            if current_level_id.0 < levels.0.len() - 1 {
                focus_text_button(
                    "Next Level",
                    (c_button.nb(), primary_box_item.nb()),
                    apply_button_state,
                    button_text,
                    p,
                )
                .set(&mut level_button);
            }

            focus_text_button(
                "Main Menu",
                (c_button.nb(), primary_box_item.nb()),
                apply_button_state,
                button_text,
                p,
            )
            .set(&mut menu_button);
        });
    });
    commands.entity(r).insert(Screen);
    commands.entity(menu_button.unwrap()).insert(Button::Menu);
    if let Some(level_button) = level_button {
        commands.entity(level_button).insert(Button::NextLevel);
    }
    for in_game in &in_game {
        commands.entity(in_game).despawn_recursive();
    }
    for mut camera in &mut camera {
        camera.translation = Vec3::new(0., 0., 5.);
        camera.look_at(Vec3::ZERO, Vec3::Y);
    }

    commands.spawn((
        Screen,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(231.8, 146.3, -2.))
                .with_scale(Vec3::ONE)
                .with_rotation(Quat::from_rotation_z(-5f32.to_radians())),
            ..Default::default()
        },
        WithMesh::PlayerCelebrate,
    ));
}

fn process_input(
    In(focused): In<Option<Entity>>,
    mut commands: Commands,
    interaction_query: TypedFocusedButtonQuery<'_, '_, '_, Button>,
    levels: Res<Levels>,
    current_level_id: Res<CurrentLevelID>,
) {
    let Some(focused) = focused else {
        return;
    };
    let Some((_entity, btn)) = interaction_query.get(focused).ok() else {
        return;
    };

    match btn {
        Button::Menu => {
            commands.insert_resource(NextState(Some(AppState::MainMenu)));
        }
        Button::NextLevel => {
            let next_level_id = current_level_id.0 + 1;
            if let Some(level) = levels.0.get(next_level_id) {
                commands.insert_resource(level.clone());
                commands.insert_resource(CurrentLevelID(next_level_id));
                commands.insert_resource(NextState(Some(AppState::ToNextLevel)));
            } else {
                commands.insert_resource(NextState(Some(AppState::Levels)));
            }
        }
    }
}

fn process_keyboard_input(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(Some(AppState::MainMenu)));
    }
}
