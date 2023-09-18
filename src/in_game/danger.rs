use bevy::{
    ecs::query::Has,
    math::Vec3Swizzles,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_inspector_egui::InspectorOptions;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::{ActionSpan, Actor, HasThinker, Thinker},
};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};
use serde::Deserialize;

use crate::{
    app_state::{AppState, DrawDebugGizmos},
    in_game::{angelic_archers::AngelicArcher, divine_detonator::DivineDetonator},
};

use super::{
    angelic_archers::angelic_archer_plugin,
    divine_detonator::divine_detonator_plugin,
    game_state::TemporaryIgnore,
    guardian_angel::guardian_angel_plugin,
    holy_hulk::{spawn_holy_hulk, HolyHulk},
    movement::Moving,
    player::Player,
    schedule::{InGameActions, InGamePostUpdate, InGameScorers, InGameUpdate},
    souls::Death,
    stealthy_seraphim::{stealthy_seraphim_plugin, StealthySeraphim},
    InGame,
};

pub fn danger_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        InGameScorers,
        (
            restless_scorer_system,
            chase_scorer_system,
            shoot_scorer_system,
        ),
    )
    .add_systems(
        InGameActions,
        (
            meandering_action_system,
            rest_action_system,
            chasing_action_system,
            shooting_action_system,
        ),
    )
    .add_systems(
        InGameUpdate,
        (
            restlessness_system,
            mark_teleported_danger,
            (spawn_holy_hulk,),
        ),
    )
    .add_systems(InGamePostUpdate, spawn_danger)
    .add_systems(
        PostUpdate,
        (draw_danger, despawn_danger, setup_danger_in_grid),
    )
    .add_systems(OnExit(AppState::InGame), clear_grid)
    .reset_resource::<CollisionGrid>();
    stealthy_seraphim_plugin(app);
    guardian_angel_plugin(app);
    angelic_archer_plugin(app);
    divine_detonator_plugin(app);
}

#[derive(Component)]
pub struct Danger(pub f32);
#[derive(Component)]
pub struct DangerSpawner(pub Entity);

#[derive(Component, Clone, Copy, Debug, Reflect, Deserialize, InspectorOptions)]
pub enum DangerType {
    HolyHulk,
    StealthySeraphim,
    GuardianAngel,
    AngelicArcher,
    DivineDetonator,
}

#[derive(Component)]
pub struct DangerAwaits;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DangerExists;

#[derive(Component)]
pub struct SpawnTime(pub f32);

pub const COLLISION_CELL_SIZE: f32 = 500.;
pub const DESPAWN_DISTANCE: f32 = 1500.;

#[derive(Resource, Default)]
pub struct CollisionGrid {
    pub map: HashMap<(i32, i32), HashSet<DangerInGrid>>,
}

fn clear_grid(mut commands: Commands) {
    commands.insert_resource(CollisionGrid::default());
}

pub struct DangerInGrid(Entity, pub Vec2, pub DangerType);

impl PartialEq for DangerInGrid {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for DangerInGrid {}

impl std::hash::Hash for DangerInGrid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

fn setup_danger_in_grid(
    dangers: Query<
        (Entity, &Transform, &DangerType),
        (Without<DangerAwaits>, Without<DangerSpawner>),
    >,
    mut grid: ResMut<CollisionGrid>,
    mut commands: Commands,
) {
    if !dangers.is_empty() {
        info!("Setting up danger grid");
    }
    for (danger, transform, danger_type) in &dangers {
        let pos = transform.translation.xy() / COLLISION_CELL_SIZE;
        let cell = (pos.x.floor() as i32, pos.y.floor() as i32);
        let cell_container = grid.map.entry(cell).or_default();
        cell_container.insert(DangerInGrid(
            danger,
            transform.translation.xy(),
            *danger_type,
        ));
        commands.entity(danger).insert(DangerAwaits);
    }
}

fn mark_teleported_danger(
    dangers: Query<Entity, (With<TemporaryIgnore>, With<Danger>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for danger in &dangers {
        commands.entity(danger).insert(SpawnTime(now));
    }
}
fn despawn_danger(
    dangers: Query<(Entity, &Transform, &SpawnTime, &DangerSpawner), Without<TemporaryIgnore>>,
    player: Query<&GlobalTransform, With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    let positions = player.iter().map(|v| v.translation()).collect::<Box<[_]>>();
    for (entity, transform, spawn_time, danger) in &dangers {
        if now - spawn_time.0 < 20. {
            continue;
        }
        let pos = transform.translation;
        if positions.iter().all(|v| v.distance(pos) > DESPAWN_DISTANCE) {
            if let Some(v) = commands.get_entity(entity) {
                v.despawn_recursive();
                if let Some(mut v) = commands.get_entity(danger.0) {
                    v.remove::<DangerExists>();
                }
            }
        }
    }
}

pub fn spawn_danger(
    dangers: Query<Entity, (With<DangerAwaits>, Without<DangerExists>)>,
    mut commands: Commands,
    grid: Res<CollisionGrid>,
    player: Query<&GlobalTransform, With<Player>>,
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

    for cell in adjacent_cells.iter() {
        let Some(grid_cell) = grid.map.get(cell) else {
            continue;
        };
        for DangerInGrid(danger, position, danger_type) in grid_cell.iter() {
            let Ok(danger) = dangers.get(*danger) else {
                continue;
            };
            info!("Found danger without danger awaits");

            let Some(mut danger_cmd) = commands.get_entity(danger) else {
                error!("Danger does not exist");
                continue;
            };
            danger_cmd.insert(DangerExists);
            let mut child = commands.spawn((
                *danger_type,
                SpatialBundle {
                    transform: Transform::from_translation(position.extend(0.)),
                    ..Default::default()
                },
                DangerSpawner(danger),
                InGame,
            ));
            match danger_type {
                DangerType::HolyHulk => {
                    child.insert(HolyHulk);
                }
                DangerType::StealthySeraphim => {
                    child.insert(StealthySeraphim);
                }
                DangerType::GuardianAngel => {
                    error!("Shouldn't get here");
                    child.despawn();
                    commands.entity(danger).despawn();
                }
                DangerType::AngelicArcher => {
                    child.insert(AngelicArcher);
                }
                DangerType::DivineDetonator => {
                    child.insert(DivineDetonator);
                }
            };
        }
    }
}

fn draw_danger(
    dangers: Query<(&GlobalTransform, &HasThinker, &Danger, &Restlessness)>,
    dangers_await: Query<(&GlobalTransform, &DangerAwaits)>,
    thinkers: Query<&Thinker>,
    mut painter: ShapePainter,
    gizmos: Res<DrawDebugGizmos>,
) {
    if !matches!(gizmos.as_ref(), DrawDebugGizmos::Collision) {
        return;
    }

    for (transform, thinker, danger, restlessness) in dangers.iter() {
        painter.color = Color::PINK;
        painter.hollow = true;
        painter.set_translation(transform.translation());
        painter.circle(danger.0);
        let Ok(thinker) = thinkers.get(thinker.entity()) else {
            continue;
        };
        info!("Thinking... {thinker:?}");
        let Some(value) = thinker.field("current_action_label") else {
            continue;
        };
        let Some(Some(Some(value))) = value.downcast_ref::<Option<Option<String>>>() else {
            continue;
        };
        painter.color = match value.as_str() {
            "Resting" => Color::BLUE,
            "Chasing" => Color::ORANGE,
            "Meandering" => Color::WHITE,
            _ => Color::BLACK,
        };
        painter.circle(danger.0 / 2.);
        painter.circle((restlessness.current_restlessness / 100.) * (danger.0 / 2.));
    }

    for (danger, _) in &dangers_await {
        painter.color = Color::BLUE;
        painter.hollow = false;
        painter.set_translation(danger.translation());
        painter.circle(15.);
    }
}

#[derive(Component, Debug)]
pub struct Restlessness {
    pub per_second: f32,
    pub current_restlessness: f32,
}

fn restlessness_system(
    time: Res<Time>,
    mut restlessness: Query<&mut Restlessness, Without<TemporaryIgnore>>,
) {
    let delta = time.delta_seconds();
    for mut restless in restlessness.iter_mut() {
        restless.current_restlessness += restless.per_second * delta;
    }
}

#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct Meandering {
    pub recovery_per_second: f32,
}

fn meandering_action_system(
    time: Res<Time>,
    mut restless: Query<&mut Restlessness>,
    mut actors: Query<(&Actor, &mut ActionState, &Meandering, &ActionSpan)>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    let delta = time.delta_seconds();
    for (Actor(actor), mut state, meandering, span) in &mut actors {
        let _guard = span.span().enter();

        if let Ok(mut restless) = restless.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    *state = ActionState::Executing;
                    let rng = rng.get_mut();
                    let direction = Vec2::new(rng.f32_normalized(), rng.f32_normalized());
                    commands.entity(*actor).insert(Moving(direction));
                }
                ActionState::Cancelled => {
                    commands.entity(*actor).remove::<Moving>();
                    *state = ActionState::Failure;
                }
                ActionState::Executing => {
                    if restless.current_restlessness <= 0. {
                        commands.entity(*actor).remove::<Moving>();
                        *state = ActionState::Success;
                    }

                    restless.current_restlessness -=
                        delta * (meandering.recovery_per_second + restless.per_second);
                }
                ActionState::Failure => {
                    commands.entity(*actor).remove::<Moving>();
                }
                ActionState::Success => {
                    commands.entity(*actor).remove::<Moving>();
                }
                _ => {}
            }
        }
    }
}

#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct Resting;

fn rest_action_system(mut actors: Query<(&Actor, &mut ActionState, &Resting, &ActionSpan)>) {
    for (Actor(_actor), mut state, _, span) in &mut actors {
        let _guard = span.span().enter();

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                *state = ActionState::Success;
            }
            ActionState::Cancelled => {
                *state = ActionState::Success;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct Restless;

fn restless_scorer_system(
    restlessness: Query<&Restlessness>,
    mut query: Query<(&Actor, &mut Score), With<Restless>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(restless) = restlessness.get(*actor) {
            let s = (restless.current_restlessness / 100.).clamp(0., 1.);
            score.set(s);
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Chasing {
    pub max_distance: f32,
    pub target_distance: f32,
    pub player: Option<Entity>,
}

fn chasing_action_system(
    mut actors: Query<(&Actor, &mut ActionState, &mut Chasing)>,
    mut chaser: Query<(&GlobalTransform, Option<&mut Restlessness>), With<Danger>>,
    players: Query<(Entity, &GlobalTransform), With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
    _death: EventWriter<Death>,
) {
    let delta = time.delta_seconds();
    for (Actor(actor), mut state, chasing) in &mut actors {
        let Ok((position, restless)) = chaser.get_mut(*actor) else {
            continue;
        };
        let position = position.translation();

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            ActionState::Executing => {
                if let Some(mut restless) = restless {
                    restless.current_restlessness -= delta * restless.per_second;
                }

                let player = if let Some(player) = chasing.player {
                    player
                } else {
                    let (_, player) =
                        players
                            .iter()
                            .fold((f32::MAX, None), |(distance, p), next| {
                                let dist = next.1.translation().distance(position);
                                if dist < distance {
                                    (dist, Some(next.0))
                                } else {
                                    (distance, p)
                                }
                            });

                    let Some(player) = player else {
                        continue;
                    };
                    player
                };

                let Ok((_, player_transform)) = players.get(player) else {
                    *state = ActionState::Failure;
                    continue;
                };

                let direction = player_transform.translation() - position;
                let distance = direction.length();

                let distance_to_target = distance - chasing.target_distance;

                if distance_to_target.abs() < 10. {
                    *state = ActionState::Success;
                    continue;
                }
                if distance > chasing.max_distance {
                    *state = ActionState::Failure;
                    continue;
                }

                let direction = if distance_to_target < 0. {
                    -direction
                } else {
                    direction
                };

                commands
                    .entity(*actor)
                    .insert(Moving(direction.normalize_or_zero().xy()));
            }
            ActionState::Failure => {
                commands.entity(*actor).remove::<Moving>();
            }
            ActionState::Success => {
                commands.entity(*actor).remove::<Moving>();
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct Chase {
    pub trigger_distance: f32,
    pub max_distance: f32,
    pub target_distance: f32,
}

fn chase_scorer_system(
    dangers: Query<&GlobalTransform, With<Danger>>,
    players: Query<&GlobalTransform, With<Player>>,
    mut query: Query<(&Actor, &mut Score, &Chase)>,
) {
    for (Actor(actor), mut score, chase) in &mut query {
        if let Ok(danger) = dangers.get(*actor) {
            let danger = danger.translation();
            for player in &players {
                let distance = danger.distance(player.translation());
                let s = if distance > chase.target_distance {
                    (distance - chase.trigger_distance).max(0.)
                        / (chase.max_distance - chase.target_distance)
                } else {
                    (chase.target_distance - distance).max(0.) / chase.target_distance
                };
                let s = 1. - s.clamp(0., 1.);
                score.set(s);
            }
        }
    }
}

#[derive(Clone, Component, Debug, ActionBuilder)]
pub struct Shooting {
    pub max_range: f32,
    pub too_close: f32,
    pub player: Option<Entity>,
    pub last_shot: f32,
    pub shot_speed: f32,
}

fn shooting_action_system(
    mut actors: Query<(&Actor, &mut ActionState, &mut Shooting)>,
    mut shooter: Query<(&GlobalTransform, Has<Shot>), With<Danger>>,
    players: Query<(Entity, &GlobalTransform), With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    for (Actor(actor), mut state, mut shooting) in &mut actors {
        let Ok((position, has_shot)) = shooter.get_mut(*actor) else {
            continue;
        };
        let position = position.translation();

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            ActionState::Executing => {
                if has_shot {
                    continue;
                }
                let player = if let Some(player) = shooting.player {
                    player
                } else {
                    let (_, player) =
                        players
                            .iter()
                            .fold((f32::MAX, None), |(distance, p), next| {
                                let dist = next.1.translation().distance(position);
                                if dist < distance {
                                    (dist, Some(next.0))
                                } else {
                                    (distance, p)
                                }
                            });

                    let Some(player) = player else {
                        continue;
                    };
                    player
                };

                let Ok((_, player_transform)) = players.get(player) else {
                    *state = ActionState::Failure;
                    continue;
                };

                let direction = player_transform.translation() - position;
                let distance = direction.length();

                if distance > shooting.max_range || distance < shooting.too_close {
                    *state = ActionState::Failure;
                    continue;
                }

                if now - shooting.last_shot < shooting.shot_speed {
                    continue;
                }

                shooting.last_shot = now;
                commands.entity(*actor).insert(Shot {
                    direction,
                    target_point: player_transform.translation(),
                });
            }
            ActionState::Failure => {
                commands.entity(*actor).remove::<Moving>();
            }
            ActionState::Success => {
                commands.entity(*actor).remove::<Moving>();
            }
            _ => {}
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[component(storage = "SparseSet")]
pub struct Shot {
    pub direction: Vec3,
    pub target_point: Vec3,
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct Shoot {
    pub max_range: f32,
    pub too_close: f32,
    pub preferred_distance: f32,
}

fn shoot_scorer_system(
    dangers: Query<(&GlobalTransform, Has<Shot>), With<Danger>>,
    players: Query<&GlobalTransform, With<Player>>,
    mut query: Query<(&Actor, &mut Score, &Shoot)>,
) {
    for (Actor(actor), mut score, shoot) in &mut query {
        if let Ok((danger, has_shot)) = dangers.get(*actor) {
            if has_shot {
                score.set(1.);
                continue;
            }
            let danger = danger.translation();
            for player in &players {
                let distance = danger.distance(player.translation());

                let s = if distance < shoot.too_close || distance > shoot.max_range {
                    0f32
                } else if distance > shoot.preferred_distance {
                    (distance - shoot.preferred_distance).abs().max(0.)
                        / (shoot.max_range - shoot.preferred_distance)
                } else {
                    (distance - shoot.preferred_distance).abs().max(0.)
                        / (shoot.preferred_distance - shoot.too_close)
                };
                let s = 1. - s.clamp(0., 1.);
                score.set(s);
            }
        }
    }
}
