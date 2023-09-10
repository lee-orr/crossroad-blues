use std::time::Duration;

use super::player::*;
use super::shadow::InShadow;
use bevy::ecs::query::Has;
use bevy::prelude::*;
use bevy_tweening::lens::*;
use bevy_tweening::*;
use bevy_vector_shapes::prelude::ShapePainter;
use bevy_vector_shapes::shapes::DiscPainter;

use seldom_state::trigger::Done;

#[derive(Component)]
pub struct CanTeleport {
    pub max_distance: f32,
}

impl Default for CanTeleport {
    fn default() -> Self {
        Self { max_distance: 200. }
    }
}

#[derive(Debug, Component, Clone)]
pub struct TeleportationTarget(pub Entity);

#[derive(Debug, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct TeleportationTargetDirection(pub Vec2, pub Entity);

#[derive(Debug, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct Teleporting;

pub fn trigger_teleport(
    targets: Query<&Transform, (With<InShadow>, With<TeleportationTarget>)>,
    teleporters: Query<
        (Entity, &TeleportationTargetDirection, &Transform),
        (With<Teleporting>, Without<Animator<Transform>>),
    >,
    mut commands: Commands,
) {
    for (teleporter, target, transform) in teleporters.iter() {
        println!("Handling teleport");
        if let Some(e) = commands.get_entity(target.1) {
            e.despawn_recursive();
        } else {
            warn!("Target doesn't exist");
            commands
                .entity(teleporter)
                .remove::<TeleportationTargetDirection>()
                .insert(Done::Success);
        }
        let Some(target) = targets.get(target.1).ok() else {
            commands
                .entity(teleporter)
                .remove::<TeleportationTargetDirection>()
                .insert(Done::Success);
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

        commands
            .entity(teleporter)
            .remove::<TeleportationTargetDirection>()
            .insert(Animator::new(seq));
    }
}

pub fn clear_teleportation_targets(
    teleporters: Query<&TeleportationTargetDirection, With<Teleporting>>,
    stuck_teleporters: Query<Entity, (With<Teleporting>, Without<Animator<Transform>>)>,
    mut commands: Commands,
) {
    for children in teleporters.iter() {
        if let Some(entity) = commands.get_entity(children.1) {
            entity.despawn_recursive()
        }
    }

    for stuck_teleporter in stuck_teleporters.iter() {
        commands.entity(stuck_teleporter).insert(Done::Success);
    }
}

pub fn target_teleportation(
    mut targets: Query<(&TeleportationTarget, &mut Transform)>,
    parents: Query<(
        &CanTeleport,
        &TeleportationTargetDirection,
        &GlobalTransform,
    )>,
) {
    for (parent, mut target) in targets.iter_mut() {
        let Ok((parent, target_direction, parent_transform)) = parents.get(parent.0) else {
            continue;
        };

        let parent_position = parent_transform.translation();

        let direction = Vec3::new(target_direction.0.x, target_direction.0.y, 0.);

        let direction = if direction.length_squared() < 0.1 {
            Vec3::ZERO
        } else {
            direction
        };

        let translation = target.translation;
        let translation = translation + direction;
        let current_dist = translation.distance(parent_position);
        let translation = if current_dist > parent.max_distance {
            let direction = (translation - parent_position).normalize_or_zero();
            parent_position + direction.normalize_or_zero() * parent.max_distance
        } else {
            translation
        };

        target.translation = translation;
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

pub fn draw_teleportation_target(
    target: Query<(&GlobalTransform, Has<InShadow>), With<TeleportationTarget>>,
    mut painter: ShapePainter,
) {
    for (transform, in_shadow) in target.iter() {
        painter.transform = Transform::from_translation(transform.translation());
        painter.color = if in_shadow {
            crate::ui::colors::PRIMARY_COLOR
        } else {
            crate::ui::colors::BAD_COLOR
        };
        painter.circle(3.);
    }
}
