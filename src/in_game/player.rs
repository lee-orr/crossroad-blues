use super::{
    actions::{input_manager, PlayerAction},
    movement::{CanMove, MoveDirection, Moving, TurnDirection},
    shadow::{CheckForShadow, InShadow},
    teleport::{CanTeleport, TeleportationTarget, Teleporting},
};
use bevy::{ecs::query::Has, prelude::*};
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
        moving.0 = if player.pressed(PlayerAction::MoveForward) {
            MoveDirection::Forward
        } else if player.pressed(PlayerAction::MoveBack) {
            MoveDirection::Back
        } else {
            MoveDirection::Still
        };

        moving.1 = if player.pressed(PlayerAction::TurnLeft) {
            TurnDirection::Left
        } else if player.pressed(PlayerAction::TurnRight) {
            TurnDirection::Right
        } else {
            TurnDirection::Still
        };
    }
}

pub fn player_target_teleportation(
    player: Query<(Entity, &ActionState<PlayerAction>), With<Moving>>,
    mut commands: Commands,
) {
    for (player, action) in player.iter() {
        if action.just_pressed(PlayerAction::Teleport) {
            commands.entity(player).with_children(|p| {
                p.spawn((
                    SpatialBundle::default(),
                    TeleportationTarget,
                    CheckForShadow,
                ));
            });
        }
    }
}
