use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use big_brain::{
    prelude::{ActionBuilder, ActionState, FirstToScore, ScorerBuilder},
    scorers::Score,
    thinker::{ActionSpan, Actor, HasThinker, Thinker},
};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::{
    app_state::{AppState, DrawDebugGizmos},
    assets::WithMesh,
};

use super::{
    game_state::TemporaryIgnore,
    holy_hulk::{spawn_holy_hulk, HolyHulk},
    movement::{CanMove, Moving},
    player::Player,
    schedule::{InGameActions, InGameScorers, InGameUpdate},
    shadow::CheckForShadow,
    souls::Death,
};

pub fn danger_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(
        PreUpdate,
        spawn_holy_hulk.run_if(in_state(AppState::InGame)),
    )
    .add_systems(InGameScorers, (restless_scorer_system, chase_scorer_system))
    .add_systems(
        InGameActions,
        (
            meandering_action_system,
            rest_action_system,
            chasing_action_system,
        ),
    )
    .add_systems(InGameUpdate, (restlessness_system, mark_teleported_danger))
    .add_systems(
        PostUpdate,
        (draw_danger, despawn_danger, setup_danger_in_grid),
    )
    .add_systems(OnExit(AppState::InGame), clear_grid)
    .reset_resource::<CollisionGrid>();
}

#[derive(Component)]
pub struct Danger(pub f32, pub String);

#[derive(Component)]
pub struct DangerAwaits;

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

pub struct DangerInGrid(pub Entity, pub Vec3);

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
    dangers: Query<(Entity, &Transform), (With<HolyHulk>, Without<DangerAwaits>, Without<Danger>)>,
    mut grid: ResMut<CollisionGrid>,
    mut commands: Commands,
) {
    for (danger, transform) in &dangers {
        let pos = transform.translation.xy() / COLLISION_CELL_SIZE;
        let cell = (pos.x.floor() as i32, pos.y.floor() as i32);
        let cell_container = grid.map.entry(cell).or_default();
        cell_container.insert(DangerInGrid(danger, transform.translation));
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
    dangers: Query<
        (Entity, &GlobalTransform, &SpawnTime),
        (With<Danger>, Without<TemporaryIgnore>),
    >,
    player: Query<&GlobalTransform, With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let now = time.elapsed_seconds();
    let positions = player.iter().map(|v| v.translation()).collect::<Box<[_]>>();
    for (danger, transform, spawn_time) in &dangers {
        if now - spawn_time.0 < 20. {
            continue;
        }
        let pos = transform.translation();
        if positions.iter().all(|v| v.distance(pos) > DESPAWN_DISTANCE) {
            commands
                .entity(danger)
                .remove::<Danger>()
                .remove::<Thinker>()
                .remove::<CanMove>()
                .remove::<CheckForShadow>()
                .remove::<Restlessness>()
                .remove::<WithMesh>()
                .remove::<SpawnTime>()
                .insert(DangerAwaits)
                .despawn_descendants();
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
    pub player: Option<Entity>,
}

fn chasing_action_system(
    mut actors: Query<(&Actor, &mut ActionState, &mut Chasing)>,
    mut chaser: Query<(&GlobalTransform, &mut Restlessness), With<Danger>>,
    players: Query<(Entity, &GlobalTransform), With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
    _death: EventWriter<Death>,
) {
    let delta = time.delta_seconds();
    for (Actor(actor), mut state, chasing) in &mut actors {
        let Ok((position, mut restless)) = chaser.get_mut(*actor) else {
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
                if restless.current_restlessness <= 0. {
                    *state = ActionState::Success;
                }

                restless.current_restlessness -= delta * restless.per_second;

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

                if distance <= 10. {
                    *state = ActionState::Success;
                    continue;
                }
                if distance > chasing.max_distance {
                    *state = ActionState::Failure;
                    continue;
                }

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
                let s = (distance - chase.trigger_distance).max(0.)
                    / (chase.max_distance - chase.trigger_distance);
                let s = 1. - s.clamp(0., 1.);
                score.set(s);
            }
        }
    }
}
