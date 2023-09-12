use std::ops::{Div, Mul};

use crate::ui::classes::*;

use super::{
    actions::{input_manager, PlayerAction},
    checkpoints::Checkpoints,
    game_state::GameState,
    movement::{CanMove, Moving},
    schedule::{InGamePreUpdate, InGameUpdate},
    shadow::{CheckForShadow, InShadow},
    souls::{Death, MaxSouls, Souls, SunSensitivity},
    teleport::{CanTeleport, TargetInRange, Teleporting},
    InGame,
};
use bevy::{ecs::query::Has, prelude::*, window::PrimaryWindow};
use bevy_tweening::Lerp;
use bevy_ui_dsl::*;
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};
use leafwing_input_manager::prelude::ActionState;
use seldom_state::{
    prelude::StateMachine,
    trigger::{DoneTrigger, JustReleasedTrigger},
};

pub fn player_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(PreUpdate, construct_player)
        .add_systems(InGamePreUpdate, (move_player, player_target_movement))
        .add_systems(InGameUpdate, (move_target, setup_souls_ui, track_camera))
        .add_systems(
            PostUpdate,
            (draw_player, draw_target, end_game, draw_souls_ui),
        );
}

#[derive(Component)]
pub struct ConstructPlayer;

#[derive(Component, Default)]
pub struct Player;

#[derive(Debug, Component, Clone)]
pub struct PlayerTarget(pub Entity);

#[derive(Debug, Component, Clone)]
pub struct PlayerTargetReference(pub Entity, pub Vec2);

pub fn construct_player(
    players: Query<(Entity, &GlobalTransform), With<ConstructPlayer>>,
    mut commands: Commands,
) {
    for (player_id, transform) in players.iter() {
        let position = transform.translation();
        let target_id = commands
            .spawn((
                InGame,
                SpatialBundle {
                    transform: Transform::from_translation(position + Vec3::X * 50.),
                    ..Default::default()
                },
                CheckForShadow,
                PlayerTarget(player_id),
            ))
            .id();

        commands
            .entity(player_id)
            .remove::<ConstructPlayer>()
            .insert((
                Name::new("Player"),
                PlayerTargetReference(target_id, Vec2::ZERO),
                Player,
                CanTeleport::default(),
                CanMove::default(),
                Moving::default(),
                input_manager(),
                CheckForShadow,
                Souls(50.),
                MaxSouls(50.),
                SunSensitivity(5.),
                Checkpoints {
                    checkpoints: Default::default(),
                    max_checkpoints: 3,
                },
                StateMachine::default()
                    .trans::<Moving>(JustReleasedTrigger(PlayerAction::Teleport), Teleporting)
                    .trans::<Teleporting>(DoneTrigger::Success, Moving::default()),
            ));
    }
}

pub fn draw_player(
    player: Query<(&Transform, Has<InShadow>), With<Player>>,
    mut painter: ShapePainter,
) {
    for (transform, in_shadow) in player.iter() {
        painter.transform = *transform;
        painter.color = if in_shadow {
            crate::ui::colors::PRIMARY_COLOR
        } else {
            crate::ui::colors::BAD_COLOR
        };
        painter.circle(10.);

        let distance = 10.;
        painter.translate(Vec3::X * distance);
        painter.circle(3.);
    }
}

pub fn move_player(mut player: Query<(&mut Moving, &ActionState<PlayerAction>)>) {
    for (mut moving, player) in player.iter_mut() {
        let Some(data) = player.axis_pair(PlayerAction::Move) else {
            continue;
        };
        moving.0 = data.xy();
    }
}

pub fn player_target_movement(
    player: Query<(
        Entity,
        &Transform,
        &ActionState<PlayerAction>,
        &PlayerTargetReference,
    )>,
    mut commands: Commands,
) {
    for (player, _transform, action, target) in player.iter() {
        let direction = action
            .axis_pair(PlayerAction::Target)
            .map(|v| v.xy())
            .unwrap_or_default();
        commands
            .entity(player)
            .insert(PlayerTargetReference(target.0, direction));
    }
}

pub fn setup_souls_ui(
    player: Query<Entity, With<Player>>,
    bars: Query<&SoulBar>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if !bars.is_empty() {
        return;
    }
    let mut player_soul_bars = vec![];
    let r = root(soul_bar_root, &asset_server, &mut commands, |p| {
        for player in player.iter() {
            node(soul_bar_container, p, |p| {
                player_soul_bars.push((node(soul_bar, p, |_| {}), player));
            });
        }
    });

    for (bar, player) in player_soul_bars {
        commands.entity(bar).insert(SoulBar(player));
    }

    commands.entity(r).insert(InGame);
}

#[derive(Component, Clone, Copy)]
pub struct SoulBar(Entity);

pub fn draw_souls_ui(
    players: Query<(&Souls, &MaxSouls), With<Player>>,
    mut bars: Query<(&mut Style, &SoulBar)>,
) {
    for (mut style, player) in bars.iter_mut() {
        let Ok((souls, max_souls)) = players.get(player.0) else {
            continue;
        };
        let ratio = souls.0.div(max_souls.0).mul(100.);
        style.width = Val::Percent(ratio);
    }
}

pub fn end_game(
    mut players: Query<(Entity, &mut Checkpoints), With<Player>>,
    mut event: EventReader<Death>,
    mut commands: Commands,
) {
    for death in event.iter() {
        let Ok((player, mut checkpoints)) = players.get_mut(death.entity) else {
            continue;
        };
        if let Some(revert) = checkpoints.checkpoints.pop_front() {
            commands.entity(player).insert((
                Transform::from_translation(revert.position),
                revert.souls,
                revert.max_souls,
            ));
        } else {
            commands.insert_resource(NextState(Some(GameState::Failed)));
        }
    }
}

pub fn move_target(
    mut targets: Query<(&PlayerTarget, &mut Transform)>,
    parents: Query<&PlayerTargetReference>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<TrackingCamera>>,
    time: Res<Time>,
) {
    let set_position = if let Ok(Some(position)) = windows.get_single().map(|v| v.cursor_position())
    {
        if let Ok((camera, camera_transform)) = camera.get_single() {
            camera
                .viewport_to_world(camera_transform, position)
                .map(|v| v.origin)
        } else {
            None
        }
    } else {
        None
    };
    for (parent, mut target) in targets.iter_mut() {
        if let Some(position) = set_position {
            target.translation = Vec3::new(position.x, position.y, 0.);
            continue;
        }

        let Ok(target_direction) = parents.get(parent.0) else {
            continue;
        };

        let direction = Vec3::new(target_direction.1.x, target_direction.1.y, 0.);

        let direction = direction.normalize_or_zero() * 150. * time.delta_seconds();

        target.translation += direction;
    }
}

pub fn draw_target(
    target: Query<(
        &GlobalTransform,
        Has<InShadow>,
        Has<TargetInRange>,
        &PlayerTarget,
    )>,
    _parent: Query<(&GlobalTransform, &CanTeleport), With<PlayerTargetReference>>,
    mut painter: ShapePainter,
) {
    for (transform, in_shadow, target_in_range, _player_target) in target.iter() {
        let too_far = !target_in_range;

        painter.transform = Transform::from_translation(transform.translation());
        painter.color = if too_far {
            crate::ui::colors::BAD_COLOR
        } else if in_shadow {
            crate::ui::colors::PRIMARY_COLOR
        } else {
            crate::ui::colors::BAD_COLOR
        };
        painter.circle(3.);
    }
}

fn track_camera(
    players: Query<&GlobalTransform, With<Player>>,
    mut camera: Query<(&mut Transform, &TrackingCamera)>,
    time: Res<Time>,
) {
    let Ok((mut transform, tracking)) = camera.get_single_mut() else {
        return;
    };

    let Some(focal_point) = players
        .iter()
        .map(|v| v.translation() + v.right() * tracking.facing_offset)
        .fold(None, |prev, val| match prev {
            Some((sum, count)) => Some((sum + val, count + 1)),
            None => Some((val, 1)),
        })
        .map(|(sum, num)| sum / (num as f32))
    else {
        return;
    };

    let delta = time.delta_seconds();

    let diff = focal_point - transform.translation;
    let distance = diff.length();
    let diff = diff.normalize_or_zero();
    let speed = 0f32.lerp(
        &tracking.speed,
        &(distance / tracking.distance_for_max_speed).clamp(0., 1.),
    );

    transform.translation += diff * delta * speed;
}

#[derive(Component)]
pub struct TrackingCamera {
    pub speed: f32,
    pub facing_offset: f32,
    pub distance_for_max_speed: f32,
}

impl Default for TrackingCamera {
    fn default() -> Self {
        Self {
            speed: 200.,
            facing_offset: 50.,
            distance_for_max_speed: 100.,
        }
    }
}
