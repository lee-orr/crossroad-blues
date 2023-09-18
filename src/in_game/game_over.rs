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
};
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};

use super::{
    danger::DangerType, game_state::GameState, player::DiedOf, souls::DamageType, InGame,
    TrackingCamera,
};
pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.setup_reloadable_elements::<reloadable>();
    }
}
#[dexterous_developer_setup(game_over)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<Screen, _, _>(GameState::Failed, setup)
        .add_systems(
            Update,
            (
                process_keyboard_input,
                (focused_button_activated.pipe(process_input)),
            )
                .run_if(in_state(GameState::Failed)),
        );
}

#[derive(Component)]
struct Screen;

#[derive(Component)]
struct Button;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players: Query<&DiedOf>,
    in_game: Query<Entity, With<InGame>>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
) {
    commands.insert_resource(ClearColor(SCREEN_BACKGROUND_COLOR));
    let mut menu_button = None;
    let r = root(c_root, &asset_server, &mut commands, |p| {
        node(primary_box, p, |p| {
            node((span.nb(), primary_box_main.nb()), p, |p| {
                text("Game Over", (), main_text, p);
            });
            for player in &players {
                node((span.nb(), primary_box_item.nb()), p, |p| {
                    text(
                        match &player.0 {
                            DamageType::Sunlight => "Sunlight Purifies, You are Impure",
                            DamageType::Danger(name) => match name {
                                DangerType::HolyHulk => "Hammered by Holy Hulk",
                                DangerType::StealthySeraphim => "Slain by Stealthy Seraphim",
                                DangerType::GuardianAngel => "Gutted by Guardian Angel",
                                DangerType::AngelicArcher => "Abolished by Angelic Archers",
                                DangerType::DivineDetonator => "Demolished by Divine Detonator",
                                DangerType::LumberingDevil => "Dunked by a Devil",
                            },
                            DamageType::TimeOut => "You didn't reach the summoning on time",
                        },
                        (),
                        standard_text,
                        p,
                    );
                });
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
    commands.entity(menu_button.unwrap()).insert(Button);
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
            transform: Transform::from_translation(Vec3::new(195.8, -109., -2.))
                .with_scale(Vec3::ONE * 2.5)
                .with_rotation(Quat::from_rotation_z(15f32.to_radians())),
            ..Default::default()
        },
        WithMesh::PlayerDead,
    ));

    let Ok(player) = players.get_single() else {
        return;
    };
    let mesh = match &player.0 {
        DamageType::Sunlight => WithMesh::Sunlight,
        DamageType::Danger(name) => match name {
            DangerType::HolyHulk => WithMesh::HolyHulkFace,
            DangerType::StealthySeraphim => WithMesh::StealthySeraphimFace,
            DangerType::GuardianAngel => WithMesh::GuardianAngelFace,
            DangerType::AngelicArcher => WithMesh::AngelicArchersFace,
            DangerType::DivineDetonator => WithMesh::DivineDetonatorFace,
            DangerType::LumberingDevil => WithMesh::DevilFace,
        },
        DamageType::TimeOut => WithMesh::PentagramFail,
    };
    commands.spawn((
        Screen,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-141.8, 50., -2.))
                .with_scale(Vec3::ONE * 1.5)
                .with_rotation(Quat::from_rotation_z(15f32.to_radians())),
            ..Default::default()
        },
        mesh,
    ));
}
fn process_input(
    In(focused): In<Option<Entity>>,
    mut commands: Commands,
    interaction_query: TypedFocusedButtonQuery<'_, '_, '_, Button>,
) {
    let Some(focused) = focused else {
        return;
    };
    let Some((_entity, _btn)) = interaction_query.get(focused).ok() else {
        return;
    };
    commands.insert_resource(NextState(Some(AppState::MainMenu)));
}

fn process_keyboard_input(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(Some(AppState::MainMenu)));
    }
}
