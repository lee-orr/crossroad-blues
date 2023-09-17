use bevy::{math::Vec3Swizzles, prelude::*, utils::HashSet};
use big_brain::{prelude::FirstToScore, thinker::Thinker};

use crate::assets::WithMesh;

use super::{
    danger::{
        Chase, Chasing, CollisionGrid, Danger, DangerAwaits, DangerInGrid, Meandering, Resting,
        Restless, Restlessness, SpawnTime, COLLISION_CELL_SIZE,
    },
    movement::CanMove,
    player::Player,
    shadow::CheckForShadow,
};

#[derive(Component)]
pub struct HolyHulk;

pub fn spawn_holy_hulk(
    dangers: Query<(Entity, &Parent), (With<HolyHulk>, With<DangerAwaits>, Without<Danger>)>,
    mut commands: Commands,
    parents: Query<&GlobalTransform, With<Parent>>,
    grid: Res<CollisionGrid>,
    player: Query<&GlobalTransform, With<Player>>,
    time: Res<Time>,
) {
    let mut adjacent_cells = HashSet::with_capacity(10);
    for player in &player {
        let pos = player.translation().xy() / COLLISION_CELL_SIZE;
        let cell = (pos.x.floor() as i32, pos.y.floor() as i32);
        for x in -1..=1 {
            let x = cell.0 + x;
            for y in -1..=1 {
                let y = cell.1 + y;
                adjacent_cells.insert((x, y));
            }
        }
    }

    if adjacent_cells.is_empty() {
        return;
    }
    let now = time.elapsed_seconds();

    for cell in adjacent_cells.iter() {
        let Some(grid_cell) = grid.map.get(cell) else {
            continue;
        };
        for DangerInGrid(danger, position) in grid_cell.iter() {
            let Ok((danger, parent)) = dangers.get(*danger) else {
                continue;
            };

            let Ok(transform) = parents.get(parent.get()) else {
                error!("Cant get danger's parent object");
                continue;
            };
            let Some(mut danger) = commands.get_entity(danger) else {
                error!("Danger does not exist");
                continue;
            };
            danger.remove::<DangerAwaits>().insert((
                Name::new("Holy Hulk"),
                Transform::from_translation(*position - transform.translation()),
                Danger(20.),
                CanMove { move_speed: 50. },
                CheckForShadow,
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
                            trigger_distance: 150.,
                            max_distance: 200.,
                        },
                        Chasing {
                            max_distance: 200.,
                            player: None,
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
            ));
        }
    }
}
