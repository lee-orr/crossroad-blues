use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_ui_dsl::{node, root};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::ui::classes::{checkpoint_marker, checkpoint_marker_content, checkpoint_marker_root};

use super::{
    player::Player,
    schedule::InGameUpdate,
    souls::{MaxSouls, Souls},
    InGame,
};

pub fn checkpoint_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(Update, draw_checkpoint)
        .add_systems(InGameUpdate, (collect_checkpoint, setup_checkpoint_ui))
        .add_systems(PostUpdate, draw_checkpoint_ui);
}

#[derive(Clone, Debug, Copy)]
pub struct StoredCheckpoint {
    pub position: Vec3,
    pub souls: Souls,
    pub max_souls: MaxSouls,
}

#[derive(Component, Clone, Debug)]
pub struct Checkpoints {
    pub checkpoints: VecDeque<StoredCheckpoint>,
    pub max_checkpoints: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Checkpoint;

fn draw_checkpoint(
    checkpoints: Query<&GlobalTransform, With<Checkpoint>>,
    mut painter: ShapePainter,
) {
    painter.color = crate::ui::colors::PRIMARY_COLOR_PRIORITIZED;

    for transform in checkpoints.iter() {
        painter.set_translation(transform.translation() + Vec3::Z * 2.);
        painter.circle(10.);
    }
}

fn collect_checkpoint(
    checkpoints: Query<(Entity, &GlobalTransform), With<Checkpoint>>,
    mut player: Query<(&GlobalTransform, &Souls, &MaxSouls, &mut Checkpoints)>,
    mut commands: Commands,
) {
    for (checkpoint, position) in checkpoints.iter() {
        let position = position.translation();
        for (player_pos, souls, max_souls, mut checkpoints) in player.iter_mut() {
            let player_pos = player_pos.translation();
            let distance = player_pos.distance(position);
            if distance < 20. {
                commands.entity(checkpoint).despawn_recursive();
                checkpoints.checkpoints.push_back(StoredCheckpoint {
                    position: player_pos,
                    souls: *souls,
                    max_souls: *max_souls,
                });
                if checkpoints.checkpoints.len() > checkpoints.max_checkpoints {
                    let _ = checkpoints.checkpoints.pop_front();
                }
                break;
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
struct CheckpointMarker(Entity, usize);

fn setup_checkpoint_ui(
    player: Query<(Entity, &Checkpoints), With<Player>>,
    markers: Query<&CheckpointMarker>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if !markers.is_empty() {
        return;
    }

    let mut checkpoint_marekers = vec![];

    let r = root(checkpoint_marker_root, &asset_server, &mut commands, |p| {
        for (player, checkpoints) in player.iter() {
            for i in 0..checkpoints.max_checkpoints {
                node(checkpoint_marker, p, |p| {
                    checkpoint_marekers.push((
                        i,
                        node(checkpoint_marker_content, p, |_| {}),
                        player,
                    ))
                });
            }
        }
    });

    for (i, marker, player) in checkpoint_marekers {
        commands.entity(marker).insert(CheckpointMarker(player, i));
    }

    commands.entity(r).insert(InGame);
}

fn draw_checkpoint_ui(
    mut markers: Query<(&CheckpointMarker, &mut Style)>,
    players: Query<&Checkpoints>,
) {
    for (CheckpointMarker(player, marker_index), mut style) in markers.iter_mut() {
        let Ok(player) = players.get(*player) else {
            continue;
        };

        if let Some(checkpoint) = player.checkpoints.get(*marker_index) {
            let percent = checkpoint.souls.0 * 100. / checkpoint.max_souls.0;
            style.height = Val::Percent(percent);
            style.display = Display::Flex;
        } else {
            style.display = Display::None;
        }
    }
}
