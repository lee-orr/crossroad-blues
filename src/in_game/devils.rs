use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use big_brain::{
    prelude::{ActionBuilder, ActionState, FirstToScore, Highest, ScorerBuilder},
    scorers::Score,
    thinker::{ActionSpan, Actor, Thinker},
};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};
use seldom_state::prelude::StateMachine;

use super::{
    movement::{CanMove, Moving},
    player::Player,
    schedule::{InGamePreUpdate, InGameUpdate},
    shadow::CheckForShadow,
    InGame,
};

pub fn devils_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(PreUpdate, spawn_lumbering_devil)
        .add_systems(
            InGamePreUpdate,
            (restless_scorer_system, restful_scorer_system),
        )
        .add_systems(
            InGameUpdate,
            (
                restlessness_system,
                meandering_action_system,
                rest_action_system,
            ),
        )
        .add_systems(PostUpdate, draw_lumbering_devil);
}

#[derive(Component)]
pub struct Devil;

#[derive(Component)]
pub struct LumberingDevil;

pub fn spawn_lumbering_devil(
    devils: Query<Entity, (With<LumberingDevil>, Without<Devil>)>,
    mut commands: Commands,
) {
    for devil in devils.iter() {
        commands.entity(devil).insert((
            Name::new("Lumbering Devil"),
            Devil,
            CanMove { move_speed: 50. },
            InGame,
            CheckForShadow,
            Restlessness {
                per_second: 25.,
                current_restlessness: 0.,
            },
            Thinker::build()
                .label("Lumbering Devil Thinker")
                .picker(FirstToScore { threshold: 0.9 })
                .when(
                    Restless,
                    Meandering {
                        recovery_per_second: 35.,
                    },
                )
                .when(Restful, Resting)
                .otherwise(Resting),
        ));
    }
}

fn draw_lumbering_devil(
    devils: Query<&GlobalTransform, With<LumberingDevil>>,
    mut painter: ShapePainter,
) {
    painter.color = crate::ui::colors::BAD_COLOR;

    for devil in devils.iter() {
        painter.set_translation(devil.translation() + Vec3::Z * 3.);
        painter.circle(30.);
    }
}

#[derive(Component, Debug)]
struct Restlessness {
    pub per_second: f32,
    pub current_restlessness: f32,
}

fn restlessness_system(time: Res<Time>, mut restlessness: Query<&mut Restlessness>) {
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
                    *state = ActionState::Failure;
                }
                ActionState::Executing => {
                    if restless.current_restlessness <= 0. {
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
    for (Actor(_), mut state, _, span) in &mut actors {
        let _guard = span.span().enter();

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
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
            score.set((restless.current_restlessness / 100.).clamp(0., 1.));
        }
    }
}

#[derive(Clone, Component, Debug, ScorerBuilder)]
struct Restful;

fn restful_scorer_system(
    restlessness: Query<&Restlessness>,
    mut query: Query<(&Actor, &mut Score), With<Restful>>,
) {
    for (Actor(actor), mut score) in &mut query {
        if let Ok(restless) = restlessness.get(*actor) {
            score.set(1. - (restless.current_restlessness / 100.).clamp(0., 1.));
        }
    }
}
