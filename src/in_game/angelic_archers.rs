use bevy::{math::Vec3Swizzles, prelude::*};

use big_brain::{prelude::FirstToScore, thinker::Thinker};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::assets::WithMesh;

use super::{
    danger::{
        Chase, Chasing, Danger, DangerSpawner, DangerType, Resting, Restlessness, Shoot, Shooting,
        Shot, SpawnTime,
    },
    movement::{CanMove, Moving},
    player::Player,
    schedule::InGameUpdate,
    souls::LethalTouch,
    InGame,
};

#[derive(Component)]
pub struct AngelicArcher;

#[derive(Component)]
pub struct AngelicArrow;

pub fn angelic_archer_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        InGameUpdate,
        (shooting, spawn_angelic_archer, despawn_angelic_arrow),
    );
}

fn spawn_angelic_archer(
    dangers: Query<Entity, (With<AngelicArcher>, Without<Danger>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for danger in &dangers {
        commands.entity(danger).insert((
            Name::new("Angelic Archer"),
            Danger(10.),
            CanMove { move_speed: 200. },
            SpawnTime(now),
            Restlessness {
                per_second: 25.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Angelic Archer")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    Shoot {
                        max_range: 300.,
                        too_close: 100.,
                        preferred_distance: 150.,
                    },
                    Shooting {
                        max_range: 300.,
                        too_close: 100.,
                        player: None,
                        last_shot: 0.,
                        shot_speed: 6.,
                    },
                )
                .when(
                    Chase {
                        trigger_distance: 600.,
                        max_distance: 700.,
                        target_distance: 150.,
                    },
                    Chasing {
                        max_distance: 600.,
                        player: None,
                        target_distance: 150.,
                    },
                )
                .otherwise(Resting),
            WithMesh::AngelicArchers,
        ));
    }
}

fn shooting(
    dangers: Query<(Entity, &Shot, &Transform), With<AngelicArcher>>,
    mut commands: Commands,
) {
    for (danger, shot, transform) in &dangers {
        let start = transform.translation;
        let direction = shot.direction.xy().normalize_or_zero();
        let angle = direction.y.atan2(direction.x);
        let rotation = Quat::from_rotation_z(angle);

        commands.entity(danger).remove::<Shot>();

        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(start).with_rotation(rotation),
                ..Default::default()
            },
            AngelicArrow,
            Name::new("Angelic Arrow"),
            Danger(10.),
            CanMove { move_speed: 200. },
            Moving(direction),
            WithMesh::AngelicArrow,
            LethalTouch,
            InGame,
            DangerType::AngelicArcher,
            DangerSpawner(danger),
        ));
    }
}

fn despawn_angelic_arrow(
    dangers: Query<(Entity, &Transform), With<AngelicArrow>>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    for (danger, transform) in &dangers {
        let position = transform.translation;
        let found = player.iter().any(|player| {
            let distance = player.translation.distance(position);
            distance < 1000.
        });

        if !found {
            commands.entity(danger).despawn_recursive();
        }
    }
}
