use bevy::{
    prelude::{KeyCode, MouseButton},
    reflect::Reflect,
};
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Move,
    Teleport,
    Target,
    ConsumeCheckpointHealth,
    SendDevilToCheckpoint,
}

pub fn input_manager() -> InputManagerBundle<PlayerAction> {
    InputManagerBundle {
        action_state: ActionState::default(),
        input_map: InputMap::new([
            (UserInput::from(KeyCode::Space), PlayerAction::Teleport),
            (
                VirtualDPad {
                    up: KeyCode::W.into(),
                    down: KeyCode::S.into(),
                    left: KeyCode::A.into(),
                    right: KeyCode::D.into(),
                }
                .into(),
                PlayerAction::Move,
            ),
            (
                VirtualDPad {
                    up: KeyCode::Up.into(),
                    down: KeyCode::Down.into(),
                    left: KeyCode::Left.into(),
                    right: KeyCode::Right.into(),
                }
                .into(),
                PlayerAction::Target,
            ),
            (KeyCode::Key1.into(), PlayerAction::ConsumeCheckpointHealth),
            (KeyCode::Key2.into(), PlayerAction::SendDevilToCheckpoint),
            (MouseButton::Left.into(), PlayerAction::Teleport),
        ])
        .build(),
    }
}
