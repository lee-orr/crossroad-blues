use bevy::prelude::*;
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

pub fn movement(mut mover: Query<(&mut Transform, &Moving, &CanMove)>, time: Res<Time>) {
    let delta = time.delta_seconds();
    for (mut transform, movement, can_move) in mover.iter_mut() {
        let direction = Vec3::new(movement.0.x, movement.0.y, 0.);
        if direction.length_squared() < 0.1 {
            continue;
        }
        let translation = direction * can_move.move_speed * delta;

        transform.translation += translation;
        transform.translation.z = 0.;

        let angle = direction.y.atan2(direction.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
