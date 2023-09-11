use std::time::Duration;

use super::player::*;
use super::schedule::InGameUpdate;
use super::shadow::InShadow;

use bevy::prelude::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;

use dexterous_developer::{ReloadableApp, ReloadableAppContents};
use seldom_state::trigger::Done;

pub fn teleport_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        InGameUpdate,
        (
            trigger_teleport,
            clear_teleport,
            validate_teleporation_target,
        ),
    );
}

#[derive(Component)]
pub struct CanTeleport {
    pub max_distance: f32,
}

impl Default for CanTeleport {
    fn default() -> Self {
        Self { max_distance: 200. }
    }
}

#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct TargetInRange;

#[derive(Debug, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct Teleporting;

pub fn trigger_teleport(
    targets: Query<&Transform, (With<TargetInRange>, With<InShadow>, With<PlayerTarget>)>,
    teleporters: Query<
        (Entity, &PlayerTargetReference, &Transform),
        (With<Teleporting>, Without<Animator<Transform>>),
    >,
    mut commands: Commands,
) {
    for (teleporter, target, transform) in teleporters.iter() {
        println!("Handling teleport");
        let Some(target) = targets.get(target.0).ok() else {
            commands.entity(teleporter).insert(Done::Success);
            continue;
        };

        let next_position = target.translation;

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

        commands.entity(teleporter).insert(Animator::new(seq));
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
                    .insert(Done::Success);
            }
        }
    }
}

pub fn validate_teleporation_target(
    target: Query<(Entity, &GlobalTransform, &PlayerTarget)>,
    parent: Query<(&GlobalTransform, &CanTeleport), With<PlayerTargetReference>>,
    mut commands: Commands,
) {
    for (target, transform, player_target) in target.iter() {
        let too_far =
            if let Ok((parent_transform, parent_can_teleport)) = parent.get(player_target.0) {
                let max = parent_can_teleport.max_distance;
                let distance = parent_transform
                    .translation()
                    .distance(transform.translation());
                distance > max
            } else {
                true
            };

        if too_far {
            commands.entity(target).remove::<TargetInRange>();
        } else {
            commands.entity(target).insert(TargetInRange);
        }
    }
}
