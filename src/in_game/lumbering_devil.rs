use bevy::prelude::*;
use big_brain::{prelude::FirstToScore, thinker::Thinker};

use crate::assets::WithMesh;

use super::{
    danger::{Chase, Chasing, Danger, Meandering, Resting, Restless, Restlessness, SpawnTime},
    movement::CanMove,
    souls::LethalTouch,
};

#[derive(Component)]
pub struct LumberingDevil;

pub fn spawn_lumbering_devil(
    dangers: Query<Entity, (With<LumberingDevil>, Without<Danger>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for danger in &dangers {
        commands.entity(danger).insert((
            Name::new("Lumbering Devil"),
            Danger(20.),
            CanMove { move_speed: 40. },
            SpawnTime(now),
            Restlessness {
                per_second: 20.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Lumbering Devil Thinker")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    Chase {
                        trigger_distance: 300.,
                        max_distance: 500.,
                        target_distance: 0.,
                    },
                    Chasing {
                        max_distance: 500.,
                        player: None,
                        target_distance: 0.,
                    },
                )
                .when(
                    Restless,
                    Meandering {
                        recovery_per_second: 65.,
                    },
                )
                .otherwise(Resting),
            WithMesh::LumberingDevil,
            LethalTouch,
        ));
    }
}
