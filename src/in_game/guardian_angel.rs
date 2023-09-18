use bevy::prelude::*;
use big_brain::{prelude::FirstToScore, thinker::Thinker};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::{assets::WithMesh, in_game::danger::DangerSpawner};

use super::{
    danger::{Chase, Chasing, Danger, DangerType, Resting},
    movement::CanMove,
    player::Player,
    ritual::{Person, RitualProceeding},
    schedule::{InGamePostUpdate, InGamePreUpdate},
    souls::LethalTouch,
    InGame,
};

pub fn guardian_angel_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGamePreUpdate, despawn_guardian_angel)
        .add_systems(InGamePostUpdate, spawn_guardian_angel);
}

#[derive(Component)]
pub struct GuardianAngel;

fn spawn_guardian_angel(
    person: Query<(Entity, &Transform, &Person), Without<RitualProceeding>>,
    dangers: Query<Entity, With<GuardianAngel>>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    if !dangers.is_empty() {
        return;
    }
    let Ok((person, transform, person_info)) = person.get_single() else {
        return;
    };

    let person_position = transform.translation;
    let _now = time.elapsed_seconds();
    for player in &player {
        let position = player.translation;
        let distance = position.distance(person_position);

        if distance > 120. {
            continue;
        }

        let diff = position - person_position;
        let angle = diff.y.atan2(diff.x);
        let rotation = Quat::from_rotation_z(angle);

        let position = diff / 3. + person_position;
        info!("Spawning Guardian Angel - {position} - distance is {distance}");

        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(position).with_rotation(rotation),
                ..Default::default()
            },
            GuardianAngel,
            Name::new("Guardian Angel"),
            Danger(20.),
            CanMove { move_speed: 150. },
            Thinker::build()
                .label("Guardian Angel Thinker")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    Chase {
                        trigger_distance: 75.,
                        max_distance: 100.,
                        target_distance: 0.,
                    },
                    Chasing {
                        max_distance: 100.,
                        player: None,
                        target_distance: 0.,
                    },
                )
                .otherwise(Resting),
            if let Some(handle) = &person_info.3 {
                WithMesh::Handle(handle.clone())
            } else {
                WithMesh::GuardianAngel
            },
            LethalTouch,
            InGame,
            DangerType::GuardianAngel,
            DangerSpawner(person),
        ));
    }
}

fn despawn_guardian_angel(
    dangers: Query<(Entity, &Transform), With<GuardianAngel>>,
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    for (danger, transform) in &dangers {
        let position = transform.translation;
        let found = player.iter().any(|player| {
            let distance = player.translation.distance(position);
            distance < 120.
        });

        if !found {
            info!("DESPAWNING GUARDIAN");
            commands.entity(danger).despawn_recursive();
        }
    }
}
