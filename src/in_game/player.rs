use bevy::prelude::*;
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};

use super::{
    actions::input_manager,
    movement::CanMove,
    teleport::{CanTeleport, TeleportState},
};

#[derive(Component, Default)]
pub struct Player;

pub fn construct_player() -> impl bevy::prelude::Bundle {
    (
        SpatialBundle::default(),
        Player,
        CanTeleport::default(),
        CanMove::default(),
        input_manager(),
    )
}

pub fn draw_player(
    player: Query<(&Transform, Option<&TeleportState>), With<Player>>,
    mut painter: ShapePainter,
) {
    for (transform, teleporting) in player.iter() {
        painter.transform = *transform;
        painter.color = crate::ui::colors::PRIMARY_COLOR;
        painter.circle(10.);

        let distance = if let Some(TeleportState::GettingReady(distance, is_valid)) = teleporting {
            if !is_valid {
                painter.color = crate::ui::colors::BORDER_COLOR;
            }
            distance + 10.
        } else {
            10.
        };
        painter.translate(Vec3::X * distance);
        painter.circle(3.);
    }
}
