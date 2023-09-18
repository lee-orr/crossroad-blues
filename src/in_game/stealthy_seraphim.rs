use std::time::Duration;

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotateZLens, TransformScaleLens},
    Animator, EaseFunction, Sequence, Tracks, Tween, TweenCompleted,
};
use big_brain::{prelude::FirstToScore, thinker::Thinker};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::assets::WithMesh;

use super::{
    danger::{Chase, Chasing, Danger, Resting, Restlessness, Shoot, Shooting, Shot, SpawnTime},
    movement::{CanMove, Moving},
    schedule::InGameUpdate,
    shadow::CheckForShadow,
    souls::LethalTouch,
};

#[derive(Component)]
pub struct StealthySeraphim;

pub fn stealthy_seraphim_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        InGameUpdate,
        (shooting, spawn_stealthy_seraphim, clear_teleport),
    );
}

fn spawn_stealthy_seraphim(
    dangers: Query<Entity, (With<StealthySeraphim>, Without<Danger>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for danger in &dangers {
        commands.entity(danger).insert((
            Name::new("Stealthy Seraphim"),
            Danger(10.),
            CanMove { move_speed: 200. },
            CheckForShadow,
            SpawnTime(now),
            Restlessness {
                per_second: 25.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Stealthy Seraphim")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    Shoot {
                        max_range: 140.,
                        too_close: 70.,
                        preferred_distance: 90.,
                    },
                    Shooting {
                        max_range: 140.,
                        too_close: 70.,
                        player: None,
                        last_shot: 0.,
                        shot_speed: 6.,
                    },
                )
                .when(
                    Chase {
                        trigger_distance: 400.,
                        max_distance: 500.,
                        target_distance: 90.,
                    },
                    Chasing {
                        max_distance: 500.,
                        player: None,
                        target_distance: 90.,
                    },
                )
                .otherwise(Resting),
            WithMesh::StealthySeraphim,
            LethalTouch,
        ));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct IsShot;

fn shooting(
    dangers: Query<(Entity, &Shot, &Transform), (With<StealthySeraphim>, Without<IsShot>)>,
    mut commands: Commands,
) {
    for (danger, shot, transform) in &dangers {
        let start = transform.translation;
        let end = shot.target_point;
        let start_rotation = transform.rotation.z;
        let direction = end.xy() - start.xy();
        let angle = direction.y.atan2(direction.x);

        let grow_initial = Tween::new(
            EaseFunction::ExponentialIn,
            Duration::from_secs_f32(0.3),
            TransformScaleLens {
                start: Vec3::ONE,
                end: Vec3::ONE * 1.3,
            },
        );
        let rotate = Tween::new(
            EaseFunction::ExponentialIn,
            Duration::from_secs_f32(0.2),
            TransformRotateZLens {
                start: start_rotation,
                end: angle,
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
            Duration::from_secs_f32(0.5),
            TransformPositionLens { start, end },
        );

        let grow = Tween::new(
            EaseFunction::ExponentialOut,
            Duration::from_secs_f32(0.5),
            TransformScaleLens {
                end: Vec3::ONE,
                start: Vec3::ONE * 0.3,
            },
        )
        .with_completed_event(TELEPORT_COMPLETED_EVENT);

        let start = Tracks::new([grow_initial, rotate]);

        let seq = Sequence::new([start])
            .then(shrink)
            .then(movement)
            .then(grow);
        commands
            .entity(danger)
            .remove::<Moving>()
            .insert((IsShot, Animator::new(seq)));
    }
}

const TELEPORT_COMPLETED_EVENT: u64 = 5377;
fn clear_teleport(
    teleporters: Query<Entity, With<IsShot>>,
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
                    .remove::<IsShot>()
                    .remove::<Shot>();
            }
        }
    }
}
