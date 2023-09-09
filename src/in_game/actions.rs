use bevy::{prelude::KeyCode, reflect::Reflect};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    MoveForward,
    MoveBack,
    TurnLeft,
    TurnRight,
    Teleport,
}

pub fn input_manager() -> InputManagerBundle<PlayerAction> {
    InputManagerBundle {
        action_state: ActionState::default(),
        input_map: InputMap::new([
            (KeyCode::Space, PlayerAction::Teleport),
            (KeyCode::W, PlayerAction::MoveForward),
            (KeyCode::S, PlayerAction::MoveBack),
            (KeyCode::A, PlayerAction::TurnLeft),
            (KeyCode::D, PlayerAction::TurnRight),
        ]),
    }
}
