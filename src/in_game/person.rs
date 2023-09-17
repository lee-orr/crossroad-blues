use std::sync::Arc;

use bevy::prelude::*;
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::assets::WithMesh;

use super::schedule::{InGamePreUpdate, InGameUpdate};

pub fn person_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGamePreUpdate, spawn_person)
        .add_systems(InGameUpdate, move_person);
}

#[derive(Component)]
pub struct Person(pub f32, pub Arc<[(Vec2, f32)]>);

#[derive(Component)]
struct TimeSoFar(f32, usize);

fn spawn_person(people: Query<Entity, (With<Person>, Without<TimeSoFar>)>, mut commands: Commands) {
    for person in &people {
        commands
            .entity(person)
            .insert((TimeSoFar(0., 0), WithMesh::Person));
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
            warn!("No next segment with id {} in {path:?}", start_time.1 + 1);
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
