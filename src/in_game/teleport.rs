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
pub struct TeleportationTarget;

#[derive(Debug, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct Teleporting;

pub fn trigger_teleport(
    targets: Query<&Transform, (With<InShadow>, With<TeleportationTarget>)>,
    teleporters: Query<
        (Entity, &Children, &Transform),
        (With<Teleporting>, Without<Animator<Transform>>),
    >,
    mut commands: Commands,
) {
    for (teleporter, children, transform) in teleporters.iter() {
        let Some(target) = children.iter().find_map(|entity| targets.get(*entity).ok()) else {
            commands.entity(teleporter).insert(Done::Success);
            continue;
        };

        let next_position = transform.transform_point(target.translation);

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

pub fn clear_teleportation_targets(
    targets: Query<Entity, With<TeleportationTarget>>,
    teleporters: Query<(Entity, &Children), With<Teleporting>>,
    mut commands: Commands,
) {
    for (_teleporter, children) in teleporters.iter() {
        let Some(target) = children.iter().find_map(|entity| targets.get(*entity).ok()) else {
            continue;
        };
        commands.entity(target).despawn_recursive();
    }
}

pub fn target_teleportation(
    mut targets: Query<(&Parent, &mut Transform), With<TeleportationTarget>>,
    parents: Query<&CanTeleport, With<Children>>,
) {
    for (parent, mut target) in targets.iter_mut() {
        let Ok(parent) = parents.get(parent.get()) else {
            continue;
        };

        let current_dist = target.translation.length();
        let dist = if current_dist > parent.max_distance {
            parent.max_distance
        } else {
            current_dist + 5.
        };

        target.translation = Vec3::X * dist;
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
