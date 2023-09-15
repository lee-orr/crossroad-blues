use std::{f32::consts, ops::Range, sync::Arc, time::Duration};

use bevy::{
    audio::{Volume, VolumeLevel},
    prelude::*,
};
use bevy_turborand::{rng::Rng, DelegatedRng, GlobalRng, TurboRand};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};
use noisy_bevy::simplex_noise_2d_seeded;

use crate::{
    app_state::AppState,
    assets::{MainGameAssets, WithMesh, ROAD_TILE_SIZE},
    ui::{
        self,
        colors::{DEFAULT_AMBIENT, SCREEN_BACKGROUND_COLOR},
    },
};

use super::{
    checkpoints::Checkpoint, devils::LumberingDevil, movement::CanMove, player::ConstructPlayer,
    shadow::Shadow, InGame,
};

pub fn level_generate_plugin(app: &mut ReloadableAppContents) {
    app.reset_resource::<CurrentLevel>()
        .reset_setup_in_state::<InGame, _, _>(AppState::InGame, spawn_level);
}

#[derive(Resource)]
pub struct CurrentLevel {
    pub song: String,
    pub song_length: Duration,
    pub bg_color: Color,
    pub ambient: AmbientLight,
    pub num_devils: Range<usize>,
    pub num_checkpoints: Range<usize>,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self {
            song: "music/crossroad-blues.flac".to_string(),
            song_length: Duration::from_secs(153),
            bg_color: SCREEN_BACKGROUND_COLOR,
            ambient: DEFAULT_AMBIENT,
            num_checkpoints: 3..5,
            num_devils: 4..6,
        }
    }
}

fn spawn_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<GlobalRng>,
    level: Res<CurrentLevel>,
) {
    let rng = rng.get_mut();
    commands.insert_resource(ClearColor(level.bg_color));
    commands.insert_resource(level.ambient.clone());
    commands.spawn((
        AudioBundle {
            source: asset_server.load(&level.song),
            settings: PlaybackSettings {
                volume: Volume::Absolute(VolumeLevel::new(0.7)),
                ..Default::default()
            },
        },
        InGame,
    ));
    commands.spawn((InGame, SpatialBundle::default(), ConstructPlayer));

    let num_checkpoints = level.num_checkpoints.clone().collect::<Arc<[_]>>();
    let num_devils = level.num_devils.clone().collect::<Arc<[_]>>();

    let num_checkpoints = rng.sample(&num_checkpoints).cloned().unwrap_or(1);
    let num_devils = rng.sample(&num_devils).cloned().unwrap_or(1);

    let default_move = CanMove::default().move_speed;

    let width = level.song_length.as_secs_f32() * default_move * (0.5 + 0.5 * rng.f32());
    let height = level.song_length.as_secs_f32() * default_move * (0.5 + 0.5 * rng.f32());

    let level_shapes = define_level_shape(rng, width, height);

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_translation(-0.5 * Vec3::new(width, height, 0.)),
                ..Default::default()
            },
            InGame,
            Name::new("Level"),
        ))
        .with_children(|p| {
            p.spawn((SpatialBundle::default(), InGame, Name::new("Road")))
                .with_children(|p| {
                    for road in level_shapes.roads.iter() {
                        spawn_road_segment(p, road, rng);
                    }
                });

            for quad in level_shapes.quads.iter() {
                fill_quad(p, quad, rng);
            }

            p.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(level_shapes.crossroads.extend(0.)),
                    ..Default::default()
                },
                InGame,
                WithMesh::Pentagram,
            ));
        });

    // for _ in 0..15 {
    //     let pos = Vec3::new(rng.f32_normalized() * 300., rng.f32_normalized() * 300., 0.);
    //     commands.spawn((
    //         SpatialBundle {
    //             transform: Transform::from_translation(pos),
    //             ..Default::default()
    //         },
    //         Shadow {
    //             radius: rng.f32_normalized().abs() * 50. + 20.,
    //         },
    //         InGame,
    //     ));
    // }

    // for _ in 0..num_checkpoints {
    //     println!("Spawning Checkpoint");
    //     let pos = Vec3::new(rng.f32_normalized() * 300., rng.f32_normalized() * 300., 0.);
    //     commands.spawn((
    //         SpatialBundle {
    //             transform: Transform::from_translation(pos),
    //             ..Default::default()
    //         },
    //         Checkpoint,
    //         WithMesh::Checkpoint,
    //         InGame,
    //     ));
    // }

    // for _ in 0..num_devils {
    //     println!("Spawning Devil");
    //     let pos = Vec3::new(rng.f32_normalized() * 300., rng.f32_normalized() * 300., 0.);
    //     commands.spawn((
    //         SpatialBundle {
    //             transform: Transform::from_translation(pos),
    //             ..Default::default()
    //         },
    //         LumberingDevil,
    //         InGame,
    //     ));
    // }
}

fn define_level_shape(rng: &Rng, width: f32, height: f32) -> LevelShape {
    let start_pos = rng.f32() * 0.2 + 0.15;
    let end_pos = rng.f32() * 0.2 + 0.65;
    let start_pos_2 = rng.f32() * 0.2 + 0.9;
    let end_pos_2 = rng.f32() * 0.2 + 0.4;

    let start_pos = edge_position_to_coord(start_pos, width, height);
    let end_pos = edge_position_to_coord(end_pos, width, height);
    let start_pos_2 = edge_position_to_coord(start_pos_2, width, height);
    let end_pos_2 = edge_position_to_coord(end_pos_2, width, height);

    let crossroads =
        Vec2::new(width, height) * Vec2::new(rng.f32() * 0.1 + 0.45, rng.f32() * 0.1 + 0.45);

    let roads = [
        (start_pos, crossroads, true),
        (crossroads, end_pos, false),
        (start_pos_2, crossroads, false),
        (crossroads, end_pos_2, false),
    ]
    .iter()
    .map(|(start, end, traverse)| LevelRoadSegment {
        start: *start,
        end: *end,
        traversal: if *traverse { Some((0., 1.)) } else { None },
    })
    .collect();

    let quads = [
        (
            push_quad_point(start_pos, start_pos_2, crossroads, rng),
            start_pos,
            crossroads,
            start_pos_2,
        ),
        (
            start_pos,
            push_quad_point(start_pos, end_pos_2, crossroads, rng),
            end_pos_2,
            crossroads,
        ),
        (
            crossroads,
            end_pos_2,
            push_quad_point(end_pos_2, end_pos, crossroads, rng),
            end_pos,
        ),
        (
            start_pos_2,
            crossroads,
            end_pos,
            push_quad_point(end_pos, start_pos_2, crossroads, rng),
        ),
    ]
    .iter()
    .map(LevelQuad::from)
    .collect();

    LevelShape {
        crossroads,
        roads,
        quads,
    }
}

fn edge_position_to_coord(pos: f32, width: f32, height: f32) -> Vec2 {
    let pos = pos % 1.;
    let pos = pos * 4.;
    let offset = pos % 1.;

    if pos < 1. {
        Vec2::Y * offset * height
    } else if pos < 2. {
        Vec2::new(offset * width, height)
    } else if pos < 3. {
        Vec2::new(width, (1. - offset) * height)
    } else {
        Vec2::X * (1. - offset) * width
    }
}

fn push_quad_point(p1: Vec2, p2: Vec2, opposite: Vec2, rng: &Rng) -> Vec2 {
    let midpoint = (p2 - p1) / 2. + p1;

    let diff = midpoint - opposite;

    midpoint + diff * (rng.f32() / 2.)
}

struct LevelShape {
    crossroads: Vec2,
    roads: Arc<[LevelRoadSegment]>,
    quads: Arc<[LevelQuad]>,
}

struct LevelQuad {
    top_left: Vec2,
    top_right: Vec2,
    bottom_left: Vec2,
    bottom_right: Vec2,
}

impl From<&(Vec2, Vec2, Vec2, Vec2)> for LevelQuad {
    fn from((bottom_left, top_left, top_right, bottom_right): &(Vec2, Vec2, Vec2, Vec2)) -> Self {
        Self {
            top_left: *top_left,
            top_right: *top_right,
            bottom_left: *bottom_left,
            bottom_right: *bottom_right,
        }
    }
}

impl From<(Vec2, Vec2, Vec2, Vec2)> for LevelQuad {
    fn from(value: (Vec2, Vec2, Vec2, Vec2)) -> Self {
        LevelQuad::from(&value)
    }
}
struct LevelRoadSegment {
    start: Vec2,
    end: Vec2,
    traversal: Option<(f32, f32)>,
}

fn spawn_road_segment(commands: &mut ChildBuilder, segment: &LevelRoadSegment, rng: &Rng) {
    let start = segment.start;
    let diff = segment.end - segment.start;
    let total_distance = segment.end.distance(segment.start);
    let roat_tile_size_t = ROAD_TILE_SIZE / total_distance;
    let mut current_t = 0.;

    while current_t < 1. {
        let point = start + diff * current_t;
        let tile_size_mod = rng.f32_normalized() * 0.2 + 1.;
        current_t += tile_size_mod * roat_tile_size_t;
        commands.spawn((
            Name::new("road segment"),
            InGame,
            WithMesh::RoadTile,
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(point.x, point.y, 0.))
                    .with_scale(Vec3::ONE * tile_size_mod)
                    .with_rotation(Quat::from_rotation_z(rng.f32() * consts::PI)),
                ..Default::default()
            },
        ));
    }
}

fn fill_quad(commands: &mut ChildBuilder, quad: &LevelQuad, rng: &Rng) {
    fill_quad_inner(1, commands, quad, rng);
}

fn fill_quad_inner(level: usize, commands: &mut ChildBuilder, quad: &LevelQuad, rng: &Rng) {
    let subdivide = level > 0 && rng.bool();
    if subdivide {
        let level = level - 1;
        let sub_quads = subdivide_quads(quad, rng);
        for quad in sub_quads.iter() {
            fill_quad_inner(level, commands, &quad, rng);
        }
    } else {
        let center = quad_center(quad);
        commands
            .spawn((SpatialBundle::default(), Name::new("Trees"), InGame))
            .with_children(|p| {
                let seed = rng.f32() * 1423.;
                let tree_density = simplex_noise_2d_seeded(center, seed).abs();

                let tree_depth = (2. * tree_density + 1.).floor() as usize;
                place_trees(tree_depth, p, quad, rng, tree_density);
            });

        commands
            .spawn((SpatialBundle::default(), Name::new("Dangers"), InGame))
            .with_children(|p| {
                let seed = rng.f32() * 115834.;
                let danger_density = simplex_noise_2d_seeded(center, seed);
                let danger_depth = (2. * danger_density + 1.).floor() as usize;
                place_danger(danger_depth, p, quad, rng, danger_density);
            });
    }
}

fn place_trees(
    level: usize,
    commands: &mut ChildBuilder,
    quad: &LevelQuad,
    rng: &Rng,
    density: f32,
) {
    if let Some(level) = level.checked_sub(1) {
        let sub_quads = subdivide_quads(quad, rng);
        for quad in sub_quads.iter() {
            place_trees(level, commands, quad, rng, density);
        }
    } else {
        let center = quad_center(quad);
        commands.spawn((
            Name::new("tree shadow"),
            InGame,
            SpatialBundle {
                transform: Transform::from_translation(center.extend(0.)),
                ..Default::default()
            },
            Shadow {
                radius: density * (rng.f32_normalized() * 0.1 + 1.) * dist_to_corners(quad, center),
            },
        ));
    }
}

fn place_danger(
    level: usize,
    commands: &mut ChildBuilder,
    quad: &LevelQuad,
    rng: &Rng,
    density: f32,
) {
    if let Some(level) = level.checked_sub(1) {
        let sub_quads = subdivide_quads(quad, rng);
        for quad in sub_quads.iter() {
            place_danger(level, commands, quad, rng, density);
        }
    } else {
        let place_danger = density > rng.f32();
        if !place_danger {
            return;
        }
        let center = quad_center(quad);
        commands.spawn((
            InGame,
            SpatialBundle {
                transform: Transform::from_translation(center.extend(0.)),
                ..Default::default()
            },
            LumberingDevil,
        ));
    }
}

fn subdivide_quads(quad: &LevelQuad, rng: &Rng) -> [LevelQuad; 4] {
    let LevelQuad {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    } = quad;
    let center = randomized_midpoint(top_left, bottom_left, rng);
    let left = randomized_midpoint(top_left, bottom_left, rng);
    let top = randomized_midpoint(top_left, top_right, rng);
    let right = randomized_midpoint(top_right, bottom_right, rng);
    let bottom = randomized_midpoint(bottom_left, bottom_right, rng);

    [
        (*bottom_left, left, center, bottom).into(),
        (left, *top_left, top, center).into(),
        (center, top, *top_right, right).into(),
        (bottom, center, right, *bottom_right).into(),
    ]
}

fn randomized_midpoint(a: &Vec2, b: &Vec2, rng: &Rng) -> Vec2 {
    (*a - *b) * (rng.f32_normalized() * 0.1 + 0.5) + *b
}

fn quad_center(quad: &LevelQuad) -> Vec2 {
    (quad.top_left + quad.top_right + quad.bottom_left + quad.bottom_right) / 4.
}

fn dist_to_corners(quad: &LevelQuad, point: Vec2) -> f32 {
    quad.top_left
        .distance(point)
        .min(quad.top_right.distance(point))
        .min(quad.bottom_right.distance(point))
        .min(quad.bottom_left.distance(point))
}
