use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::{actions::PlayerAction, player::Player, teleport::TeleportState};

#[derive(Component)]
pub struct CanMove {
    pub move_speed: f32,
    pub turn_speed: f32,
}

impl Default for CanMove {
    fn default() -> Self {
        Self {
            move_speed: 3.,
            turn_speed: 0.1,
        }
    }
}

pub fn move_player(
    mut player: Query<
        (
            &mut Transform,
            Option<&TeleportState>,
            &ActionState<PlayerAction>,
            &CanMove,
        ),
        With<Player>,
    >,
) {
    for (mut transform, teleport, movement, can_move) in player.iter_mut() {
        let vertical = if movement.pressed(PlayerAction::MoveForward) {
            1.
        } else if movement.pressed(PlayerAction::MoveBack) {
            -1.
        } else {
            0.
        };
        let horizontal = if movement.pressed(PlayerAction::TurnRight) {
            -1.
        } else if movement.pressed(PlayerAction::TurnLeft) {
            1.
        } else {
            0.
        };
        if matches!(teleport, Some(TeleportState::Teleporting)) {
            continue;
        }
        transform.rotate_z(horizontal * can_move.turn_speed);

        let translation = transform.transform_point(Vec3::X * vertical * can_move.move_speed);

        transform.translation = translation;
    }
}
