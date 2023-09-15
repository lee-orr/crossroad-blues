use std::time::Duration;

use super::schedule::InGameUpdate;
use super::shadow::InShadow;
use super::{game_state::TemporaryIgnore, player::*};

use bevy::ecs::query::Has;
use bevy::prelude::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;

use dexterous_developer::{ReloadableApp, ReloadableAppContents};

pub fn teleport_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        InGameUpdate,
        (trigger_teleport, validate_teleporation_target),
    )
    .add_systems(PostUpdate, (clear_teleport, run_teleport));
}

#[derive(Component)]
pub struct CanTeleport {
    pub max_distance: f32,
    pub min_distance: f32,
}

impl Default for CanTeleport {
    fn default() -> Self {
        Self {
            max_distance: 300.,
            min_distance: 75.,
        }
    }
}

#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct TargetInRange;

#[derive(Debug, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct Teleporting;

#[derive(Debug, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct StartTeleport(pub Vec3);

pub fn trigger_teleport(
    targets: Query<
        &Transform,
        (
            With<TargetInRange>,
            With<InShadow>,
            With<PlayerTarget>,
            Without<TemporaryIgnore>,
        ),
    >,
    teleporters: Query<
        (Entity, &PlayerTargetReference),
        (
            With<Teleporting>,
            Without<StartTeleport>,
            Without<TemporaryIgnore>,
        ),
    >,
    mut commands: Commands,
) {
    for (teleporter, target) in teleporters.iter() {
        println!("Handling teleport");
        let Some(target) = targets.get(target.0).ok() else {
            continue;
        };

        let next_position = target.translation;

        commands
            .entity(teleporter)
            .insert(StartTeleport(next_position));
    }
}

pub fn run_teleport(
    teleporter: Query<(
        Entity,
        &Transform,
        &GlobalTransform,
        &StartTeleport,
        Has<TemporaryIgnore>,
    )>,
    mut commands: Commands,
) {
    for (entity, transform, _global, start_teleport, ignore) in &teleporter {
        println!("Running Teleport");
        if ignore {
            continue;
        }
        let mut start = transform.translation;
        let start_scale = transform.scale;
        let mut end = start_teleport.0;
        start.z = 0.;
        end.z = 0.;
        let direction = end - start;
        let angle = direction.y.atan2(direction.x);

        let shrink = Tween::new(
            EaseFunction::ExponentialIn,
            Duration::from_secs_f32(0.1),
            TransformScaleLens {
                start: start_scale,
                end: start_scale * 0.3,
            },
        );

        let grow = Tween::new(
            EaseFunction::ExponentialOut,
            Duration::from_secs_f32(0.1),
            TransformScaleLens {
                start: start_scale * 0.3,
                end: start_scale,
            },
        )
        .with_completed_event(TELEPORT_COMPLETED_EVENT);

        let movement = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(0.4),
            TransformPositionLens { start, end },
        );

        let rotate = Tween::new(
            EaseFunction::CubicIn,
            Duration::from_secs_f32(0.1),
            TransformRotateZLens {
                start: transform.rotation.z,
                end: angle,
            },
        );

        let start = Tracks::new([shrink, rotate]);

        let seq = Sequence::new([start]).then(movement).then(grow);

        commands
            .entity(entity)
            .remove::<StartTeleport>()
            .insert((Animator::new(seq), TemporaryIgnore));
    }
}

const TELEPORT_COMPLETED_EVENT: u64 = 22;

pub fn clear_teleport(
    teleporters: Query<Entity>,
    mut event: EventReader<TweenCompleted>,
    mut commands: Commands,
) {
    for event in event.iter() {
        if event.user_data == TELEPORT_COMPLETED_EVENT {
            if let Ok(teleporter) = teleporters.get(event.entity) {
                println!("Clearing Teleport");
                commands
                    .entity(teleporter)
                    .remove::<Animator<Transform>>()
                    .remove::<TemporaryIgnore>();
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
        let not_in_range =
            if let Ok((parent_transform, parent_can_teleport)) = parent.get(player_target.0) {
                let max = parent_can_teleport.max_distance;
                let min = parent_can_teleport.min_distance;
                let distance = parent_transform
                    .translation()
                    .distance(transform.translation());
                distance > max || distance < min
            } else {
                true
            };

        if not_in_range {
            commands.entity(target).remove::<TargetInRange>();
        } else {
            commands.entity(target).insert(TargetInRange);
        }
    }
}
