use bevy::prelude::*;
use big_brain::{prelude::FirstToScore, thinker::Thinker};

use crate::assets::WithMesh;

use super::{
    danger::{Chase, Chasing, Danger, Meandering, Resting, Restless, Restlessness, SpawnTime},
    movement::CanMove,
    souls::LethalTouch,
};

#[derive(Component)]
pub struct HolyHulk;

pub fn spawn_holy_hulk(
    dangers: Query<Entity, (With<HolyHulk>, Without<Danger>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for danger in &dangers {
        commands.entity(danger).insert((
            Name::new("Holy Hulk"),
            Danger(20.),
            CanMove { move_speed: 50. },
            SpawnTime(now),
            Restlessness {
                per_second: 25.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Holy Hulk Thinker")
                .picker(FirstToScore { threshold: 0.8 })
                .when(
                    Chase {
                        trigger_distance: 200.,
                        max_distance: 300.,
                        target_distance: 0.,
                    },
                    Chasing {
                        max_distance: 300.,
                        player: None,
                        target_distance: 0.,
                    },
                )
                .when(
                    Restless,
                    Meandering {
                        recovery_per_second: 35.,
                    },
                )
                .otherwise(Resting),
            WithMesh::HolyHulk,
            LethalTouch,
        ));
    }
}
