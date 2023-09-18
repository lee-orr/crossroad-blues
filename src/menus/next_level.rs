use bevy::prelude::*;

use bevy_ui_dsl::*;
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};

use crate::{
    app_state::AppState,
    assets::WithMesh,
    in_game::{CurrentLevel, Levels, TrackingCamera},
    ui::{
        buttons::{focus_text_button, focused_button_activated},
        classes::*,
        colors::SCREEN_BACKGROUND_COLOR,
        intermediary_node_bundles::*,
    },
    CurrentLevelID,
};

use super::game_title;
pub struct NextLevelPlugin;

impl Plugin for NextLevelPlugin {
    fn build(&self, app: &mut App) {
        app.setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(next_level)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<Screen, _, _>(AppState::ToNextLevel, setup)
        .add_systems(
            Update,
            (focused_button_activated.pipe(process_input), level_ready)
                .run_if(in_state(AppState::ToNextLevel)),
        );
}

#[derive(Component)]
struct Screen;

#[derive(Component, Copy, Clone)]
struct LevelButton;

#[derive(Component)]
struct LevelButtonReady;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_level: ResMut<CurrentLevel>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
) {
    if current_level.song_handle.is_none() {
        current_level.song_handle = Some(asset_server.load(&current_level.song));
    }
    commands.insert_resource(ClearColor(SCREEN_BACKGROUND_COLOR));

    let mut buttons = vec![];

    let r = root(c_root, &asset_server, &mut commands, |p| {
        node(primary_box, p, |p| {
            game_title::game_title(p);

            for t in current_level.initial_text.iter() {
                text(t.as_str(), primary_box_item.nb(), standard_text, p);
            }

            let button = focus_text_button(
                "Start Mission",
                (c_button.nb(), primary_box_item.nb(), disable_button.nb()),
                apply_button_state,
                button_text,
                p,
            );
            buttons.push((button, LevelButton));
        });
    });
    commands.entity(r).insert(Screen);
    for (button, lvlbtn) in buttons.iter() {
        commands.entity(*button).insert(*lvlbtn);
    }
    for mut camera in &mut camera {
        camera.translation = Vec3::new(0., 0., 5.);
        camera.look_at(Vec3::ZERO, Vec3::Y);
    }

    commands.spawn((
        Screen,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-157.7, 62.6, -2.))
                .with_scale(Vec3::ONE * 3.2)
                .with_rotation(Quat::from_rotation_z(5f32.to_radians())),
            ..Default::default()
        },
        WithMesh::DevilFace,
    ));

    commands.spawn((
        Screen,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(239., -99., -2.))
                .with_scale(Vec3::ONE * 3.),
            ..Default::default()
        },
        WithMesh::PlayerFace,
    ));
}

fn level_ready(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    audio: Res<Assets<AudioSource>>,
    buttons: Query<(Entity, &LevelButton), Without<LevelButtonReady>>,
) {
    if buttons.is_empty() {
        return;
    }
    let Some(handle) = &current_level.song_handle else {
        return;
    };
    if audio.get(handle).is_some() {
        for (button, _) in &buttons {
            commands.entity(button).insert(LevelButtonReady);
        }
    }
}

fn process_input(
    In(focused): In<Option<Entity>>,
    buttons: Query<&LevelButton, With<LevelButtonReady>>,
    mut commands: Commands,
) {
    let Some(entity) = focused else {
        return;
    };
    let Ok(_button) = buttons.get(entity) else {
        return;
    };
    commands.insert_resource(NextState(Some(AppState::InGame)));
}
