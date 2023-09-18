use bevy::prelude::*;

use bevy_ui_dsl::*;
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};

use crate::{
    app_state::AppState,
    in_game::Levels,
    ui::{
        buttons::{focus_text_button, focused_button_activated},
        classes::*,
        colors::SCREEN_BACKGROUND_COLOR,
        intermediary_node_bundles::*,
    },
    CurrentLevelID,
};

use super::game_title;
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(levels)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<Screen, _, _>(AppState::Levels, setup)
        .add_systems(
            Update,
            (focused_button_activated.pipe(process_input)).run_if(in_state(AppState::Levels)),
        );
}

#[derive(Component)]
struct Screen;

#[derive(Component, Copy, Clone)]
enum LevelButton {
    Level(usize),
    Menu,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, levels: Res<Levels>) {
    commands.insert_resource(ClearColor(SCREEN_BACKGROUND_COLOR));

    let mut buttons = vec![];

    let r = root((c_root, opaque.nb()), &asset_server, &mut commands, |p| {
        node(primary_box, p, |p| {
            game_title::game_title(p);

            for (i, level) in levels.0.iter().enumerate() {
                let button = focus_text_button(
                    level.name.as_str(),
                    (c_button.nb(), primary_box_item.nb()),
                    apply_button_state,
                    button_text,
                    p,
                );
                buttons.push((button, LevelButton::Level(i)));
            }

            let button = focus_text_button(
                "Main Menu",
                (c_button.nb(), primary_box_item.nb()),
                apply_button_state,
                button_text,
                p,
            );
            buttons.push((button, LevelButton::Menu));
        });
    });
    commands.entity(r).insert(Screen);
    for (button, lvlbtn) in buttons.iter() {
        commands.entity(*button).insert(*lvlbtn);
    }
}

fn process_input(
    In(focused): In<Option<Entity>>,
    buttons: Query<&LevelButton>,
    mut commands: Commands,
    levels: Res<Levels>,
) {
    let Some(entity) = focused else {
        return;
    };
    let Ok(button) = buttons.get(entity) else {
        return;
    };
    match button {
        LevelButton::Level(i) => {
            if let Some(level) = levels.0.get(*i) {
                commands.insert_resource(CurrentLevelID(*i));
                commands.insert_resource(level.clone());
                commands.insert_resource(NextState(Some(AppState::InGame)))
            }
        }
        LevelButton::Menu => commands.insert_resource(NextState(Some(AppState::MainMenu))),
    };
}
