use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use bevy_xpbd_2d::prelude::{debug::PhysicsDebugConfig, *};
use big_brain::{
    prelude::{ActionBuilder, ActionState, FirstToScore, ScorerBuilder},
    scorers::Score,
    thinker::{ActionSpan, Actor, HasThinker, Thinker},
};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::{app_state::DrawDebugGizmos, assets::WithMesh};

use super::{
    game_state::TemporaryIgnore,
    movement::{CanMove, Moving},
    player::Player,
    schedule::{InGameActions, InGameScorers, InGameUpdate},
    shadow::CheckForShadow,
    souls::Death,
    InGame,
};

pub fn devils_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(PreUpdate, spawn_lumbering_devil)
        .add_systems(InGameScorers, (restless_scorer_system, chase_scorer_system))
        .add_systems(
            InGameActions,
            (
                meandering_action_system,
                rest_action_system,
                chasing_action_system,
            ),
        )
        .add_systems(InGameUpdate, (restlessness_system,))
        .add_systems(PostUpdate, draw_devil);
}

#[derive(Component)]
pub struct Danger(pub f32);

#[derive(Component)]
pub struct LumberingDevil;

pub fn spawn_lumbering_devil(
    devils: Query<Entity, (With<LumberingDevil>, Without<Danger>)>,
    mut commands: Commands,
) {
    for devil in devils.iter() {
        commands.entity(devil).insert((
            Name::new("Lumbering Devil"),
            Danger(20.),
            CanMove { move_speed: 50. },
            InGame,
            CheckForShadow,
            Restlessness {
                per_second: 25.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Lumbering Devil Thinker")
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
            WithMesh::LumberingDevil,
            Collider::ball(20.),
            Sensor,
            CollidingEntities::default(),
            RigidBody::Kinematic,
        ));
    }
}

fn draw_devil(
    devils: Query<(&GlobalTransform, &HasThinker, &Danger, &Restlessness)>,
    thinkers: Query<&Thinker>,
    mut painter: ShapePainter,
    gizmos: Res<DrawDebugGizmos>,
) {
    if !matches!(gizmos.as_ref(), DrawDebugGizmos::InternalCircles) {
        return;
    }

    for (devil, thinker, danger, restlessness) in devils.iter() {
        painter.color = Color::PINK;
        painter.hollow = true;
        painter.set_translation(devil.translation());
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
}

#[derive(Component, Debug)]
struct Restlessness {
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
struct Meandering {
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
struct Resting;

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
struct Restless;

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
struct Chasing {
    max_distance: f32,
    player: Option<Entity>,
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
struct Chase {
    trigger_distance: f32,
    max_distance: f32,
}

fn chase_scorer_system(
    devils: Query<&GlobalTransform, With<Danger>>,
    players: Query<&GlobalTransform, With<Player>>,
    mut query: Query<(&Actor, &mut Score, &Chase)>,
) {
    for (Actor(actor), mut score, chase) in &mut query {
        if let Ok(devil) = devils.get(*actor) {
            let devil = devil.translation();
            for player in &players {
                let distance = devil.distance(player.translation());
                let s = (distance - chase.trigger_distance).max(0.)
                    / (chase.max_distance - chase.trigger_distance);
                let s = 1. - s.clamp(0., 1.);
                score.set(s);
            }
        }
    }
}
