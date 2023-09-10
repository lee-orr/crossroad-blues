use std::ops::{Div, Mul};

use crate::ui::classes::*;

use super::{
    actions::{input_manager, PlayerAction},
    movement::{CanMove, Moving},
    shadow::{CheckForShadow, InShadow},
    souls::{MaxSouls, Souls, SunSensitivity},
    teleport::{CanTeleport, TeleportationTarget, TeleportationTargetDirection, Teleporting},
    InGame,
};
use bevy::{ecs::query::Has, prelude::*};
use bevy_ui_dsl::*;
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use leafwing_input_manager::prelude::ActionState;
use seldom_state::{
    prelude::StateMachine,
    trigger::{DoneTrigger, JustReleasedTrigger},
};

#[derive(Component, Default)]
pub struct Player;

pub fn construct_player() -> impl bevy::prelude::Bundle {
    (
        Name::new("Player"),
        SpatialBundle::default(),
        Player,
        CanTeleport::default(),
        CanMove::default(),
        Moving::default(),
        input_manager(),
        CheckForShadow,
        Souls(50.),
        MaxSouls(50.),
        SunSensitivity(5.),
        StateMachine::default()
            .trans::<Moving>(JustReleasedTrigger(PlayerAction::Teleport), Teleporting)
            .trans::<Teleporting>(DoneTrigger::Success, Moving::default()),
    )
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

pub fn player_target_teleportation(
    player: Query<
        (
            Entity,
            &Transform,
            &ActionState<PlayerAction>,
            Option<&TeleportationTargetDirection>,
        ),
        With<Moving>,
    >,
    mut commands: Commands,
) {
    for (player, transform, action, target) in player.iter() {
        if action.just_pressed(PlayerAction::Teleport) {
            let target = commands
                .spawn((
                    InGame,
                    SpatialBundle {
                        transform: *transform,
                        ..Default::default()
                    },
                    TeleportationTarget(player),
                    CheckForShadow,
                ))
                .id();
            commands
                .entity(player)
                .insert(TeleportationTargetDirection(Vec2::ZERO, target));
        } else if action.pressed(PlayerAction::Teleport) {
            let direction = action
                .axis_pair(PlayerAction::Target)
                .map(|v| v.xy())
                .unwrap_or_default();
            let Some(target) = target.map(|v| v.1) else {
                continue;
            };
            commands
                .entity(player)
                .insert(TeleportationTargetDirection(direction, target));
        }
    }
}

pub fn setup_souls_ui(
    player: Query<Entity, With<Player>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
