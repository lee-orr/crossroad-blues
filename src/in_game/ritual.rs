use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{lens::TransformScaleLens, Animator, Delay, EaseFunction, Tween};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::assets::WithMesh;

use super::{
    game_state::GameState,
    player::{DiedOf, Player},
    schedule::{InGamePostUpdate, InGamePreUpdate, InGameUpdate},
    InGame,
};

pub fn ritual_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGamePreUpdate, spawn_person)
        .add_systems(InGameUpdate, move_person)
        .add_systems(
            InGamePostUpdate,
            (start_ritual, end_ritual).run_if(in_state(GameState::InGame)),
        );
}

#[derive(Component)]
pub struct Person(pub f32, pub Arc<[(Vec2, f32)]>);

#[derive(Component)]
struct TimeSoFar(f32, usize);

#[derive(Component)]
struct Ritual {
    position: Vec2,
    radius: f32,
    start_time: f32,
    end_time: f32,
}

#[derive(Component)]
struct RitualProceeding;

fn spawn_person(
    mut people: Query<(Entity, &mut Person), Without<TimeSoFar>>,
    mut commands: Commands,
) {
    for (person, mut p) in &mut people {
        let start_time = p.0 - 10.;
        let end_time = p.0;
        let radius = 30.;

        p.0 -= 11.;
        let mut path = Vec::from_iter(p.1.iter().cloned());
        let position = if let Some(last) = path.last_mut() {
            let position = last.0;
            last.0 -= Vec2::X * 45.;
            position
        } else {
            Vec2::default()
        };
        p.1 = path.into_iter().collect();

        commands.entity(person).insert((
            TimeSoFar(0., 0),
            WithMesh::Person,
            Ritual {
                position,
                radius,
                start_time,
                end_time,
            },
        ));
    }
}

fn start_ritual(
    mut commands: Commands,
    rituals: Query<(Entity, &Ritual, &TimeSoFar), Without<RitualProceeding>>,
) {
    for (entity, ritual, time) in &rituals {
        if time.0 >= ritual.start_time {
            let punch_duration = Duration::from_secs_f32(0.5);
            let circle_duration = Duration::from_secs_f32(4.);
            commands.entity(entity).insert(RitualProceeding);

            let punch_animation = Tween::new(
                EaseFunction::QuadraticIn,
                circle_duration,
                TransformScaleLens {
                    start: Vec3::ONE * 100.,
                    end: Vec3::ONE,
                },
            );

            commands.spawn((
                Name::new("Pentagram Circle"),
                SpatialBundle {
                    transform: Transform::from_translation(ritual.position.extend(0.))
                        .with_scale(Vec3::ZERO),
                    ..Default::default()
                },
                WithMesh::PentagramCircle,
                InGame,
                Animator::new(punch_animation),
            ));

            let angle_offset = (-360. / 5f32).to_radians();
            let triangle_start = 5.;

            for i in 0..5 {
                let i = i as f32;
                let delay =
                    Delay::new(Duration::from_secs_f32(triangle_start + i) - punch_duration);
                let punch_animation = Tween::new(
                    EaseFunction::QuadraticIn,
                    punch_duration,
                    TransformScaleLens {
                        start: Vec3::ONE * 100.,
                        end: Vec3::ONE,
                    },
                );
                let sequence = delay.then(punch_animation);

                commands.spawn((
                    Name::new(format!("Pentagram {i}")),
                    SpatialBundle {
                        transform: Transform::from_translation(ritual.position.extend(0.))
                            .with_scale(Vec3::ZERO),
                        ..Default::default()
                    },
                    WithMesh::PentagramTriangle(angle_offset * i),
                    InGame,
                    Animator::new(sequence),
                ));
            }
        }
    }
}

fn end_ritual(
    mut commands: Commands,
    rituals: Query<(&Ritual, &TimeSoFar), With<RitualProceeding>>,
    players: Query<(Entity, &GlobalTransform), With<Player>>,
) {
    for (ritual, time) in &rituals {
        if time.0 >= ritual.end_time {
            let position = ritual.position.extend(0.);
            let radius = ritual.radius;
            for (player, transform) in &players {
                if transform.translation().distance(position) <= radius {
                    commands.insert_resource(NextState(Some(GameState::Complete)));
                    return;
                }
                commands
                    .entity(player)
                    .insert(DiedOf(super::souls::DamageType::TimeOut));
            }
            commands.insert_resource(NextState(Some(GameState::Failed)));
        }
    }
}

fn move_person(mut people: Query<(&mut Transform, &Person, &mut TimeSoFar)>, time: Res<Time>) {
    let delta = time.delta_seconds();

    for (mut transform, person, mut start_time) in &mut people {
        start_time.0 += delta;
        let delta = start_time.0;
        let t = (delta / person.0).clamp(0., 1.);

        let path = &person.1;

        let Some(current_segment) = path.get(start_time.1) else {
            warn!("No segment with id {} in {path:?}", start_time.1);
            continue;
        };

        let Some(next_segment) = path.get(start_time.1 + 1) else {
            continue;
        };

        let (current_segment, next_segment) = if next_segment.1 > t {
            (current_segment, next_segment)
        } else {
            start_time.1 += 1;
            let Some(new_segment) = path.get(start_time.1 + 1) else {
                warn!(
                    "Can't increment id - No next segment with id {} in {path:?}",
                    start_time.1 + 1
                );
                continue;
            };
            info!(
                "Incrementing ID to {} - path from {next_segment:?} to {new_segment:?}",
                start_time.1
            );
            (next_segment, new_segment)
        };

        if next_segment.1 <= current_segment.1 {
            error!("Next segment smaller than current segment");
            continue;
        }

        let t = (t - current_segment.1) / (next_segment.1 - current_segment.1);
        let direction = next_segment.0 - current_segment.0;
        let point = direction * t + current_segment.0;

        transform.translation = point.extend(0.);
        let angle = direction.y.atan2(direction.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
