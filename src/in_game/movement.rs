use bevy::prelude::*;

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

#[derive(Component, Clone, Default)]
#[component(storage = "SparseSet")]
pub struct Moving(pub MoveDirection, pub TurnDirection);

#[derive(Clone, Default, Copy, Eq, PartialEq)]
pub enum MoveDirection {
    #[default]
    Still,
    Forward,
    Back,
}
#[derive(Clone, Default, Copy, Eq, PartialEq)]
pub enum TurnDirection {
    #[default]
    Still,
    Left,
    Right,
}

pub fn movement(mut mover: Query<(&mut Transform, &Moving, &CanMove)>) {
    for (mut transform, movement, can_move) in mover.iter_mut() {
        let vertical = match movement.0 {
            MoveDirection::Still => 0.,
            MoveDirection::Forward => 1.,
            MoveDirection::Back => -1.,
        };
        let horizontal = match movement.1 {
            TurnDirection::Still => 0.,
            TurnDirection::Left => 1.,
            TurnDirection::Right => -1.,
        };
        transform.rotate_z(horizontal * can_move.turn_speed);

        let translation = transform.transform_point(Vec3::X * vertical * can_move.move_speed);

        transform.translation = translation;
    }
}
