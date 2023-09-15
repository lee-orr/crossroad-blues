use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use super::schedule::InGameUpdate;

pub fn movement_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGameUpdate, movement);
}

#[derive(Component)]
pub struct CanMove {
    pub move_speed: f32,
}

impl Default for CanMove {
    fn default() -> Self {
        Self { move_speed: 175. }
    }
}

#[derive(Component, Clone, Default)]
#[component(storage = "SparseSet")]
pub struct Moving(pub Vec2);

pub fn movement(
    mut mover: Query<(&mut Position, &mut Rotation, &Moving, &CanMove)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (mut transform, mut rotation, movement, can_move) in mover.iter_mut() {
        let direction = movement.0;
        if direction.length_squared() < 0.1 {
            continue;
        }
        let translation = direction * can_move.move_speed * delta;

        transform.0 += translation;

        let angle = direction.y.atan2(direction.x);
        *rotation = Rotation::from_radians(angle);
    }
}
