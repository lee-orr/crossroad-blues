mod game_completed;
mod game_over;
mod game_state;
mod pause_screen;

use std::{default, time::Duration};

use bevy::{
    audio::{Volume, VolumeLevel},
    ecs::schedule::ScheduleLabel,
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_inspector_egui::quick::StateInspectorPlugin;
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, EaseFunction, Tween, TweenCompleted,
};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};

use crate::{
    app_state::AppState,
    assets::MainGameAssets,
    ui::colors::{DEFAULT_AMBIENT, DEFAULT_CLEAR},
};

use self::{
    game_completed::GameCompletedPlugin,
    game_over::GameOverPlugin,
    game_state::{GameState, PauseState},
    pause_screen::PausePlugin,
};
use dexterous_developer::{
    dexterous_developer_setup, ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PausePlugin, GameOverPlugin, GameCompletedPlugin))
            .add_state::<GameState>()
            .register_type::<GameState>()
            .add_plugins(
                StateInspectorPlugin::<GameState>::default()
                    .run_if(input_toggle_active(false, KeyCode::F1)),
            )
            .add_systems(OnExit(AppState::InGame), (exit, clear_audio))
            .add_systems(Update, (enable_audio).run_if(in_state(AppState::InGame)))
            .setup_reloadable_elements::<reloadable>();
    }
}

#[dexterous_developer_setup(in_game)]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup_in_state::<InGame, _, _>(AppState::InGame, setup)
        .add_systems(
            Update,
            run_in_game_update.run_if(in_state(PauseState::None)),
        )
        .add_systems(
            InGameUpdate,
            (move_player, teleport_control, clear_teleport),
        )
        .add_systems(PostUpdate, (draw_player, draw_shadow));
}

#[derive(Component)]
struct InGame;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InGameUpdate;

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct Shadow {
    radius: f32,
}

#[derive(Debug, Component)]
enum TeleportState {
    GettingReady(f32, bool),
    Teleporting,
}

fn setup(mut commands: Commands, assets: Res<MainGameAssets>, mut rng: ResMut<GlobalRng>) {
    let mut rng = rng.get_mut();
    commands.insert_resource(ClearColor(DEFAULT_CLEAR));
    commands.insert_resource(DEFAULT_AMBIENT);
    commands
        .spawn((
            InGame,
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .with_children(|p| {
            p.spawn(AudioBundle {
                source: assets.menu_music.clone(),
                settings: PlaybackSettings {
                    paused: true,
                    volume: Volume::Absolute(VolumeLevel::new(0.)),
                    ..Default::default()
                },
            });

            p.spawn((SpatialBundle::default(), Player));

            for _ in 0..5 {
                let pos = Vec3::new(
                    rng.f32_normalized() * 300.,
                    rng.f32_normalized() * 300.,
                    -5.,
                );
                p.spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(pos),
                        ..Default::default()
                    },
                    Shadow {
                        radius: rng.f32_normalized().abs() * 50. + 20.,
                    },
                ));
            }
        });
}

fn exit(mut commands: Commands, query: Query<Entity, With<InGame>>) {
    commands.insert_resource(NextState(Some(GameState::None)));
    commands.insert_resource(NextState(Some(PauseState::None)));
    for item in query.iter() {
        commands.entity(item).despawn_recursive();
    }
}

fn clear_audio(audio: Query<&AudioSink>) {
    for audio in audio.iter() {
        audio.stop();
    }
}

fn enable_audio(audio: Query<&AudioSink>) {
    for audio in audio.iter() {
        if audio.is_paused() {
            audio.play();
        }
    }
}

fn run_in_game_update(world: &mut World) {
    let _ = world.try_run_schedule(InGameUpdate);
}

fn move_player(
    mut player: Query<(&mut Transform, Option<&TeleportState>), With<Player>>,
    movement: Res<Input<KeyCode>>,
) {
    let vertical = if movement.pressed(KeyCode::W) {
        1.
    } else if movement.pressed(KeyCode::S) {
        -1.
    } else {
        0.
    };
    let horizontal = if movement.pressed(KeyCode::D) {
        -1.
    } else if movement.pressed(KeyCode::A) {
        1.
    } else {
        0.
    };

    for (mut transform, teleport) in player.iter_mut() {
        if matches!(teleport, Some(TeleportState::Teleporting)) {
            continue;
        }
        transform.rotate_z(horizontal * 0.1);

        let translation = transform.transform_point(Vec3::X * vertical * 3.0);

        transform.translation = translation;
    }
}

fn draw_player(
    player: Query<(&Transform, Option<&TeleportState>), With<Player>>,
    mut painter: ShapePainter,
) {
    for (transform, teleporting) in player.iter() {
        painter.transform = *transform;
        painter.color = crate::ui::colors::PRIMARY_COLOR;
        painter.circle(10.);

        let distance = if let Some(TeleportState::GettingReady(distance, is_valid)) = teleporting {
            if !is_valid {
                painter.color = crate::ui::colors::BORDER_COLOR;
            }
            distance + 10.
        } else {
            10.
        };
        painter.translate(Vec3::X * distance);
        painter.circle(3.);
    }
}

fn draw_shadow(shadow: Query<(&Transform, &Shadow)>, mut painter: ShapePainter) {
    painter.color = crate::ui::colors::PRIMARY_BACKGROUND_COLOR;
    for (trasnform, shadow) in shadow.iter() {
        painter.set_translation(trasnform.translation);
        painter.circle(shadow.radius);
    }
}

fn teleport_control(
    players: Query<Entity, With<Player>>,
    teleport_states: Query<(Entity, &TeleportState, &Transform), With<Player>>,
    shadows: Query<(&GlobalTransform, &Shadow)>,
    mut commands: Commands,
    teleport: Res<Input<KeyCode>>,
) {
    if teleport.just_pressed(KeyCode::Space) {
        for player in players.iter() {
            commands
                .entity(player)
                .insert(TeleportState::GettingReady(0., false));
        }
    } else if teleport.just_released(KeyCode::Space) {
        for (entity, teleport_state, transform) in teleport_states.iter() {
            let dist = match &teleport_state {
                TeleportState::GettingReady(dist, true) => Some(*dist),
                _ => None,
            };
            if let Some(dist) = dist {
                let next_position = transform.transform_point(Vec3::X * dist);

                let shrink = Tween::new(
                    EaseFunction::ExponentialIn,
                    Duration::from_secs_f32(0.1),
                    TransformScaleLens {
                        start: transform.scale,
                        end: Vec3::ONE * 0.1,
                    },
                );

                let grow = Tween::new(
                    EaseFunction::ExponentialOut,
                    Duration::from_secs_f32(0.1),
                    TransformScaleLens {
                        start: Vec3::ONE * 0.1,
                        end: Vec3::ONE,
                    },
                )
                .with_completed_event(TELEPORT_COMPLETED_EVENT);

                let movement = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(0.4),
                    TransformPositionLens {
                        start: transform.translation,
                        end: next_position,
                    },
                );

                let seq = shrink.then(movement).then(grow);

                commands
                    .entity(entity)
                    .insert((TeleportState::Teleporting, Animator::new(seq)));
            } else {
                commands.entity(entity).remove::<TeleportState>();
            }
        }
    } else if teleport.pressed(KeyCode::Space) {
        for (entity, teleport_state, transform) in teleport_states.iter() {
            if let TeleportState::GettingReady(dist, _) = teleport_state {
                let dist = *dist;
                let dist = if dist >= 200. { 200. } else { dist + 5. };

                let next_position = transform.transform_point(Vec3::X * dist);
                let valid = shadows.iter().any(|(transform, shadow)| {
                    let position = transform.translation();
                    let distance = position.distance(next_position);
                    distance < shadow.radius
                });
                commands
                    .entity(entity)
                    .insert(TeleportState::GettingReady(dist, valid));
            }
        }
    }
}

const TELEPORT_COMPLETED_EVENT: u64 = 22;

fn clear_teleport(
    players: Query<Entity, With<Player>>,
    mut event: EventReader<TweenCompleted>,
    mut commands: Commands,
) {
    for event in event.iter() {
        if event.user_data == TELEPORT_COMPLETED_EVENT {
            if let Ok(player) = players.get(event.entity) {
                commands
                    .entity(player)
                    .remove::<Animator<Transform>>()
                    .remove::<TeleportState>();
            }
        }
    }
}
