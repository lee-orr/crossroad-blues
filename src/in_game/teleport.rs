use std::time::Duration;

use super::actions::*;
use super::player::*;
use super::shadow::Shadow;
use bevy::prelude::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;
use leafwing_input_manager::prelude::ActionState;

#[derive(Component)]
pub struct CanTeleport {
    pub max_distance: f32,
}

impl Default for CanTeleport {
    fn default() -> Self {
        Self { max_distance: 200. }
    }
}

#[derive(Debug, Component)]
pub enum TeleportState {
    GettingReady(f32, bool),
    Teleporting,
}

pub fn teleport_control(
    players: Query<(Entity, &ActionState<PlayerAction>), With<Player>>,
    teleport_states: Query<(&TeleportState, &Transform, &CanTeleport), With<Player>>,
    shadows: Query<(&GlobalTransform, &Shadow)>,
    mut commands: Commands,
) {
    for (entity, teleport) in players.iter() {
        if teleport.just_pressed(PlayerAction::Teleport) {
            commands
                .entity(entity)
                .insert(TeleportState::GettingReady(0., false));
        } else if teleport.just_released(PlayerAction::Teleport) {
            if let Ok((teleport_state, transform, _)) = teleport_states.get(entity) {
                let dist = match &teleport_state {
                    TeleportState::GettingReady(dist, true) => Some(*dist),
                    _ => None,
                };
                if let Some(dist) = dist {
                    let next_position = transform.transform_point(Vec3::X * dist);

                    let shrink = Tween::new(
                        EaseFunction::ExponentialIn,
                        Duration::from_secs_f32(0.1),
                        TransformScaleLens {
                            start: transform.scale,
                            end: Vec3::ONE * 0.1,
                        },
                    );

                    let grow = Tween::new(
                        EaseFunction::ExponentialOut,
                        Duration::from_secs_f32(0.1),
                        TransformScaleLens {
                            start: Vec3::ONE * 0.1,
                            end: Vec3::ONE,
                        },
                    )
                    .with_completed_event(TELEPORT_COMPLETED_EVENT);

                    let movement = Tween::new(
                        EaseFunction::QuadraticInOut,
                        Duration::from_secs_f32(0.4),
                        TransformPositionLens {
                            start: transform.translation,
                            end: next_position,
                        },
                    );

                    let seq = shrink.then(movement).then(grow);

                    commands
                        .entity(entity)
                        .insert((TeleportState::Teleporting, Animator::new(seq)));
                } else {
                    commands.entity(entity).remove::<TeleportState>();
                }
            }
        } else if teleport.pressed(PlayerAction::Teleport) {
            if let Ok((TeleportState::GettingReady(dist, _), transform, can_teleport)) =
                teleport_states.get(entity)
            {
                let dist = *dist;
                let dist = if dist >= can_teleport.max_distance {
                    can_teleport.max_distance
                } else {
                    dist + 5.
                };

                let next_position = transform.transform_point(Vec3::X * dist);
                let valid = shadows.iter().any(|(transform, shadow)| {
                    let position = transform.translation();
                    let distance = position.distance(next_position);
                    distance < shadow.radius
                });
                commands
                    .entity(entity)
                    .insert(TeleportState::GettingReady(dist, valid));
            }
        }
    }
}

const TELEPORT_COMPLETED_EVENT: u64 = 22;

pub fn clear_teleport(
    players: Query<Entity, With<Player>>,
    mut event: EventReader<TweenCompleted>,
    mut commands: Commands,
) {
    for event in event.iter() {
        if event.user_data == TELEPORT_COMPLETED_EVENT {
            if let Ok(player) = players.get(event.entity) {
                commands
                    .entity(player)
                    .remove::<Animator<Transform>>()
                    .remove::<TeleportState>();
            }
        }
    }
}
