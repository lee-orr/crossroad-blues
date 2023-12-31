use bevy::{
    audio::{PlaybackMode, Volume, VolumeLevel},
    prelude::*,
};
use bevy_ui_dsl::*;

use crate::{
    app_state::AppState,
    assets::{MainGameAssets, WithMesh},
    in_game::TrackingCamera,
    ui::{
        buttons::{focus_text_button, focused_button_activated, TypedFocusedButtonQuery},
        classes::*,
        colors::SCREEN_BACKGROUND_COLOR,
        intermediary_node_bundles::*,
    },
};
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};

use super::game_title;
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(main_menu)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<Screen, _, _>(AppState::MainMenu, setup)
        .add_systems(
            Update,
            (focused_button_activated.pipe(process_input)).run_if(in_state(AppState::MainMenu)),
        );
}

#[derive(Component)]
struct Screen;

#[derive(Component)]
enum Buttons {
    Start,
    Credits,
}

fn setup(
    mut commands: Commands,
    assets: Res<MainGameAssets>,
    asset_server: Res<AssetServer>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
) {
    commands.insert_resource(ClearColor(SCREEN_BACKGROUND_COLOR));

    let mut start_button = None;
    let mut credits_button = None;

    let r = root(c_root, &asset_server, &mut commands, |p| {
        node(primary_box, p, |p| {
            game_title::game_title(p);
            focus_text_button(
                "Start Game",
                (c_button.nb(), primary_box_item.nb()),
                apply_button_state,
                button_text,
                p,
            )
            .set(&mut start_button);
            focus_text_button(
                "Credits",
                (c_button.nb(), primary_box_item.nb()),
                apply_button_state,
                button_text,
                p,
            )
            .set(&mut credits_button);
        });
    });
    commands.entity(r).insert(Screen);
    commands
        .entity(start_button.unwrap())
        .insert(Buttons::Start);
    commands
        .entity(credits_button.unwrap())
        .insert(Buttons::Credits);
    commands.spawn((
        AudioBundle {
            source: assets.menu_music.clone(),
            settings: PlaybackSettings {
                volume: Volume::Absolute(VolumeLevel::new(0.7)),
                mode: PlaybackMode::Loop,
                ..Default::default()
            },
        },
        Screen,
    ));

    for mut camera in &mut camera {
        camera.translation = Vec3::new(0., 0., 5.);
        camera.look_at(Vec3::ZERO, Vec3::Y);
    }

    commands.spawn((
        Screen,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(150., 121., -2.))
                .with_scale(Vec3::ONE * 2.5)
                .with_rotation(Quat::from_rotation_z(15f32.to_radians())),
            ..Default::default()
        },
        WithMesh::PlayerFace,
    ));
}

fn process_input(
    In(focused): In<Option<Entity>>,
    mut commands: Commands,
    interaction_query: TypedFocusedButtonQuery<'_, '_, '_, Buttons>,
) {
    let Some(focused) = focused else {
        return;
    };
    let Some((_entity, btn)) = interaction_query.get(focused).ok() else {
        return;
    };
    match btn {
        Buttons::Start => commands.insert_resource(NextState(Some(AppState::Levels))),
        Buttons::Credits => commands.insert_resource(NextState(Some(AppState::Credits))),
    };
}
