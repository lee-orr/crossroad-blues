use crate::{
    app_state::AppState,
    ui::{classes::*, intermediary_node_bundles::IntoIntermediaryNodeBundle},
};
use bevy::prelude::*;
use bevy_ui_dsl::*;
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use super::{schedule::InGameUpdate, CurrentLevel, InGame};

pub fn in_game_text_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGameUpdate, display_new_text)
        .add_systems(OnEnter(AppState::InGame), setup);
}

#[derive(Component, Clone)]
pub struct InGameText {
    current_index: Option<usize>,
    time_so_far: f32,
    root: Entity,
    visible: bool,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut in_game_text = None;
    let root = root(c_root, &asset_server, &mut commands, |p| {
        node(in_game_text_box.nb(), p, |p| {
            image(devil_in_game_text_box, p);
            let t = text("", (), standard_text, p);
            in_game_text = Some(t);
        });
    });
    commands
        .entity(root)
        .insert((Visibility::Hidden, InGame, Name::new("In Game Text UI")));
    if let Some(text) = in_game_text {
        commands.entity(text).insert(InGameText {
            current_index: None,
            time_so_far: 0.,
            root,
            visible: false,
        });
    }
}

pub fn display_new_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut existing_text: Query<(Entity, &mut InGameText)>,
    time: Res<Time>,
    level: Res<CurrentLevel>,
) {
    let delta = time.delta_seconds();
    for (entity, mut in_game_text) in &mut existing_text {
        in_game_text.time_so_far += delta;
        let now = in_game_text.time_so_far;

        let mut current_text = None;
        for (id, (start, end, text)) in level.timed_text.iter().enumerate() {
            if now < *start || now > *end {
                continue;
            }
            current_text = Some((id, text.as_str()));
        }

        if let Some((id, text)) = current_text {
            if !in_game_text.visible {
                commands
                    .entity(in_game_text.root)
                    .insert(Visibility::Visible);
            }
            let insert = if let Some(current) = in_game_text.current_index {
                id != current
            } else {
                true
            };
            if insert {
                let mut style = TextStyle::default();
                standard_text(&asset_server, &mut style);
                commands
                    .entity(entity)
                    .insert(Text::from_section(text, style));
                in_game_text.current_index = Some(id);
            }
        } else if in_game_text.visible {
            in_game_text.current_index = None;
            commands
                .entity(in_game_text.root)
                .insert(Visibility::Hidden);
        }
    }
}
