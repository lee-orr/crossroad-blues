use std::time::Duration;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, EaseFunction, Tween, TweenCompleted,
};
use big_brain::{prelude::FirstToScore, thinker::Thinker};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::assets::WithMesh;

use super::{
    danger::{Danger, Resting, Restlessness, Shoot, Shooting, Shot, SpawnTime},
    movement::{CanMove, Moving},
    schedule::InGameUpdate,
    souls::LethalTouch,
};

#[derive(Component)]
pub struct DivineDetonator;

pub fn divine_detonator_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        InGameUpdate,
        (shooting, spawn_divine_detonator, clear_teleport),
    );
}

fn spawn_divine_detonator(
    dangers: Query<Entity, (With<DivineDetonator>, Without<Danger>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for danger in &dangers {
        commands.entity(danger).insert((
            Name::new("Divine Detonator"),
            Danger(10.),
            CanMove { move_speed: 200. },
            SpawnTime(now),
            Restlessness {
                per_second: 25.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Divine Detonator")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    Shoot {
                        max_range: 250.,
                        too_close: 0.,
                        preferred_distance: 90.,
                    },
                    Shooting {
                        max_range: 250.,
                        too_close: 0.,
                        player: None,
                        last_shot: 0.,
                        shot_speed: 6.,
                    },
                )
                .otherwise(Resting),
            WithMesh::DivineDetonator,
        ));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct IsShot;

fn shooting(
    dangers: Query<(Entity, &Shot, &Transform), (With<DivineDetonator>, Without<IsShot>)>,
    mut commands: Commands,
) {
    for (danger, shot, transform) in &dangers {
        let start = transform.translation;
        let end = shot.target_point;
        let _start_rotation = transform.rotation.z;
        let direction = end.xy() - start.xy();
        let _angle = direction.y.atan2(direction.x);

        let grow_initial = Tween::new(
            EaseFunction::ExponentialIn,
            Duration::from_secs_f32(0.2),
            TransformScaleLens {
                start: Vec3::ONE,
                end: Vec3::ONE * 1.3,
            },
        );

        let shrink = Tween::new(
            EaseFunction::ExponentialIn,
            Duration::from_secs_f32(0.2),
            TransformScaleLens {
                start: Vec3::ONE * 1.3,
                end: Vec3::ONE * 0.3,
            },
        );

        let movement = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(0.7),
            TransformPositionLens { start, end },
        )
        .with_completed_event(TELEPORT_COMPLETED_EVENT);

        let grow = Tween::new(
            EaseFunction::ExponentialOut,
            Duration::from_secs_f32(0.2),
            TransformScaleLens {
                end: Vec3::ONE * 40.,
                start: Vec3::ONE * 0.3,
            },
        );

        let shrink_final = Tween::new(
            EaseFunction::ExponentialOut,
            Duration::from_secs_f32(0.2),
            TransformScaleLens {
                start: Vec3::ONE * 40.,
                end: Vec3::ZERO,
            },
        )
        .with_completed_event(EXPLOSION_DONE);

        let seq = grow_initial
            .then(shrink)
            .then(movement)
            .then(grow)
            .then(shrink_final);
        commands
            .entity(danger)
            .remove::<Moving>()
            .insert((IsShot, Animator::new(seq)));
    }
}

const TELEPORT_COMPLETED_EVENT: u64 = 241412;

const EXPLOSION_DONE: u64 = 1212443;

fn clear_teleport(
    teleporters: Query<Entity, With<IsShot>>,
    mut event: EventReader<TweenCompleted>,
    mut commands: Commands,
) {
    for event in event.iter() {
        if event.user_data == EXPLOSION_DONE {
            if let Ok(teleporter) = teleporters.get(event.entity) {
                commands.entity(teleporter).despawn_recursive();
            }
        } else if event.user_data == TELEPORT_COMPLETED_EVENT {
            if let Ok(teleporter) = teleporters.get(event.entity) {
                commands
                    .entity(teleporter)
                    .insert(Danger(80.))
                    .despawn_descendants()
                    .insert((WithMesh::DivineDetonatorExplosion, LethalTouch));
            }
        }
    }
}
