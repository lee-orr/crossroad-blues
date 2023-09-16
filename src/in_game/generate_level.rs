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
    assets::{WithMesh, ROAD_TILE_SIZE},
    ui::colors::{DEFAULT_AMBIENT, SCREEN_BACKGROUND_COLOR},
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

    let num_checkpoints = level.num_checkpoints.clone().collect::<Arc<[_]>>();
    let num_devils = level.num_devils.clone().collect::<Arc<[_]>>();

    let _num_checkpoints = rng.sample(&num_checkpoints).cloned().unwrap_or(1);
    let _num_devils = rng.sample(&num_devils).cloned().unwrap_or(1);

    let default_move = CanMove::default().move_speed;

    let width = level.song_length.as_secs_f32() * default_move * (0.5 + 0.3 * rng.f32()) / 2.;
    let height = level.song_length.as_secs_f32() * default_move * (0.5 + 0.3 * rng.f32()) / 2.;

    let level_shapes = define_level_shape(rng, width, height);

    commands
        .spawn((InGame, SpatialBundle::default(), Name::new("Level")))
        .with_children(|p| {
            p.spawn((SpatialBundle::default(), Name::new("Road")))
                .with_children(|p| {
                    for road in level_shapes.roads.iter() {
                        spawn_road_segment(p, road, rng);
                    }
                });

            for section in level_shapes.section.iter() {
                fill_section(p, section, rng);
            }

            p.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(level_shapes.crossroads.extend(0.)),
                    ..Default::default()
                },
                WithMesh::Pentagram,
            ));

            let player_position = Vec2::new(rng.f32() * 0.1, rng.f32() * 0.1);
            let first_section = level_shapes
                .section
                .first()
                .cloned()
                .unwrap_or((Vec2::ZERO, Vec2::Y, Vec2::ONE, Vec2::X).into());
            let player_position = first_section.point_from_normalized(player_position);

            p.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(player_position.extend(0.)),
                    ..Default::default()
                },
                ConstructPlayer,
            ));
        });
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
    .map(|(start, end, _traverse)| LevelRoadSegment {
        start: *start,
        end: *end,
        // traversal: if *traverse { Some((0., 1.)) } else { None },
    })
    .collect();

    let sections = [
        (
            push_section_point(start_pos, start_pos_2, crossroads, rng),
            start_pos,
            crossroads,
            start_pos_2,
        ),
        (
            start_pos,
            push_section_point(start_pos, end_pos_2, crossroads, rng),
            end_pos_2,
            crossroads,
        ),
        (
            crossroads,
            end_pos_2,
            push_section_point(end_pos_2, end_pos, crossroads, rng),
            end_pos,
        ),
        (
            start_pos_2,
            crossroads,
            end_pos,
            push_section_point(end_pos, start_pos_2, crossroads, rng),
        ),
    ]
    .iter()
    .map(LevelSections::from)
    .collect();

    LevelShape {
        crossroads,
        roads,
        section: sections,
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

fn push_section_point(p1: Vec2, p2: Vec2, opposite: Vec2, rng: &Rng) -> Vec2 {
    let midpoint = (p2 - p1) / 2. + p1;

    let diff = midpoint - opposite;

    midpoint + diff * (rng.f32() / 2.)
}
fn spawn_road_segment(commands: &mut ChildBuilder, segment: &LevelRoadSegment, rng: &Rng) {
    let start = segment.start;
    let diff = segment.end - segment.start;
    let total_distance = segment.end.distance(segment.start);
    let road_tile_size_t = ROAD_TILE_SIZE / total_distance;
    let mut current_t = 0.;

    while current_t < 1. {
        let point = start + diff * current_t;
        let tile_size_mod = rng.f32_normalized() * 0.2 + 1.;
        current_t += tile_size_mod * road_tile_size_t;
        commands.spawn((
            Name::new("road segment"),
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

fn fill_section(commands: &mut ChildBuilder, section: &LevelSections, rng: &Rng) {
    fill_section_inner(1, commands, section, rng);
}

const MIN_SECTION_SIZE: f32 = 100.;

fn fill_section_inner(
    level: usize,
    commands: &mut ChildBuilder,
    section: &LevelSections,
    rng: &Rng,
) {
    let subdivide = level > 0 && rng.bool() && section.size_min() > MIN_SECTION_SIZE;
    if subdivide {
        let level = level - 1;
        let sub_sections = section.subdivide_sections(rng);
        for section in sub_sections.iter() {
            fill_section_inner(level, commands, section, rng);
        }
    } else {
        let center = section.section_center();
        commands
            .spawn((SpatialBundle::default(), Name::new("Trees")))
            .with_children(|p| {
                let seed = rng.f32() * 1423.;
                let tree_density = simplex_noise_2d_seeded(center, seed).abs() * 0.4 + 0.5;

                place_trees(2, p, section, rng, tree_density);
            });

        commands
            .spawn((SpatialBundle::default(), Name::new("Dangers")))
            .with_children(|p| {
                let seed = rng.f32() * 115834.;
                let danger_density = simplex_noise_2d_seeded(center, seed).abs() * 0.4 + 0.2;
                place_danger(2, p, section, rng, danger_density);
            });

        commands
            .spawn((SpatialBundle::default(), Name::new("Checkpoints")))
            .with_children(|p| {
                let seed = rng.f32() * 124326.;
                let danger_density = simplex_noise_2d_seeded(center, seed).abs() * 0.3 + 0.1;
                place_checkpoints(3, p, section, rng, danger_density);
            });
    }
}

const TREE_SIZES: &[f32] = &[1000., 500., 100.];

fn place_trees(
    level: usize,
    commands: &mut ChildBuilder,
    section: &LevelSections,
    rng: &Rng,
    density: f32,
) {
    let main_axis = section.main_axis_min_length();
    let cross_axis = section.cross_axis_min_length();

    let Some(size) = TREE_SIZES.get(level) else {
        if level > TREE_SIZES.len() && !TREE_SIZES.is_empty() {
            place_trees(TREE_SIZES.len() - 1, commands, section, rng, density);
        }
        error!("No tree sizes available");
        return;
    };

    if (main_axis < *size || cross_axis < *size) && level > 0 {
        place_trees(level - 1, commands, section, rng, density);
        return;
    }

    let main_tiles = (main_axis / size).round().max(1.) as usize;
    let cross_tiles = (cross_axis / size).round().max(1.) as usize;
    let main_step = 1. / (main_tiles as f32);
    let cross_step = 1. / (cross_tiles as f32);
    let tree_radius = (main_step * main_axis / 2.)
        .min(size / 2.)
        .min(cross_step * cross_axis / 2.);

    for x in 0..main_tiles {
        for y in 0..cross_tiles {
            let x = (x as f32) * main_step;
            let y = (y as f32) * cross_step;
            let inner = LevelSections {
                top_left: section.point_from_normalized(Vec2::new(x, y + cross_step)),
                top_right: section.point_from_normalized(Vec2::new(x + main_step, y + cross_step)),
                bottom_left: section.point_from_normalized(Vec2::new(x, y)),
                bottom_right: section.point_from_normalized(Vec2::new(x + main_step, y)),
            };

            let spawn_here = rng.f32() < density;
            if !spawn_here {
                if level > 0 {
                    place_trees(level - 1, commands, &inner, rng, density);
                }
                continue;
            }
            let point = inner.point_from_normalized(Vec2::new(
                rng.f32_normalized() * 0.25 + 0.5,
                rng.f32_normalized() * 0.25 + 0.5,
            ));
            let radius = tree_radius * (rng.f32() * 0.2 + 0.9);
            commands.spawn((
                Name::new("tree shadow"),
                SpatialBundle {
                    transform: Transform::from_translation(point.extend(0.)),
                    ..Default::default()
                },
                Shadow { radius },
            ));
        }
    }
}

const DANGER_DISTANCES: &[f32] = &[1000., 500., 300.];

fn place_danger(
    level: usize,
    commands: &mut ChildBuilder,
    section: &LevelSections,
    rng: &Rng,
    density: f32,
) {
    let main_axis = section.main_axis_min_length();
    let cross_axis = section.cross_axis_min_length();

    let Some(size) = DANGER_DISTANCES.get(level) else {
        if level > DANGER_DISTANCES.len() && !DANGER_DISTANCES.is_empty() {
            place_danger(DANGER_DISTANCES.len() - 1, commands, section, rng, density);
        }
        error!("No danger distances available");
        return;
    };

    if (main_axis < *size || cross_axis < *size) && level > 0 {
        place_danger(level - 1, commands, section, rng, density);
        return;
    }

    let main_tiles = (main_axis / size).round().max(1.) as usize;
    let cross_tiles = (cross_axis / size).round().max(1.) as usize;
    let main_step = 1. / (main_tiles as f32);
    let cross_step = 1. / (cross_tiles as f32);

    for x in 0..main_tiles {
        for y in 0..cross_tiles {
            let x = (x as f32) * main_step;
            let y = (y as f32) * cross_step;
            let inner = LevelSections {
                top_left: section.point_from_normalized(Vec2::new(x, y + cross_step)),
                top_right: section.point_from_normalized(Vec2::new(x + main_step, y + cross_step)),
                bottom_left: section.point_from_normalized(Vec2::new(x, y)),
                bottom_right: section.point_from_normalized(Vec2::new(x + main_step, y)),
            };

            let spawn_here = rng.f32() < density;
            if !spawn_here {
                if level > 0 {
                    place_danger(level - 1, commands, &inner, rng, density);
                }
                continue;
            }
            let point = inner.point_from_normalized(Vec2::new(
                rng.f32_normalized() * 0.5 + 0.5,
                rng.f32_normalized() * 0.5 + 0.5,
            ));
            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(point.extend(0.)),
                    ..Default::default()
                },
                LumberingDevil,
            ));
        }
    }
}

const CHECKPOINT_DISTANCES: &[f32] = &[1200., 600., 400., 200.];

fn place_checkpoints(
    level: usize,
    commands: &mut ChildBuilder,
    section: &LevelSections,
    rng: &Rng,
    density: f32,
) {
    let main_axis = section.main_axis_min_length();
    let cross_axis = section.cross_axis_min_length();

    let Some(size) = CHECKPOINT_DISTANCES.get(level) else {
        if level > CHECKPOINT_DISTANCES.len() && !CHECKPOINT_DISTANCES.is_empty() {
            place_checkpoints(
                CHECKPOINT_DISTANCES.len() - 1,
                commands,
                section,
                rng,
                density,
            );
        }
        error!("No danger distances available");
        return;
    };

    if (main_axis < *size || cross_axis < *size) && level > 0 {
        place_checkpoints(level - 1, commands, section, rng, density);
        return;
    }

    let main_tiles = (main_axis / size).round().max(1.) as usize;
    let cross_tiles = (cross_axis / size).round().max(1.) as usize;
    let main_step = 1. / (main_tiles as f32);
    let cross_step = 1. / (cross_tiles as f32);

    for x in 0..main_tiles {
        for y in 0..cross_tiles {
            let x = (x as f32) * main_step;
            let y = (y as f32) * cross_step;
            let inner = LevelSections {
                top_left: section.point_from_normalized(Vec2::new(x, y + cross_step)),
                top_right: section.point_from_normalized(Vec2::new(x + main_step, y + cross_step)),
                bottom_left: section.point_from_normalized(Vec2::new(x, y)),
                bottom_right: section.point_from_normalized(Vec2::new(x + main_step, y)),
            };

            let spawn_here = rng.f32() < density;
            if !spawn_here {
                if level > 0 {
                    place_checkpoints(level - 1, commands, &inner, rng, density);
                }
                continue;
            }
            let point = inner.point_from_normalized(Vec2::new(
                rng.f32_normalized() * 0.5 + 0.5,
                rng.f32_normalized() * 0.5 + 0.5,
            ));
            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(point.extend(0.)),
                    ..Default::default()
                },
                Checkpoint,
                WithMesh::Checkpoint,
            ));
        }
    }
}

fn randomized_midpoint(a: &Vec2, b: &Vec2, rng: &Rng) -> Vec2 {
    (*a - *b) * (rng.f32_normalized() * 0.1 + 0.5) + *b
}

struct LevelShape {
    crossroads: Vec2,
    roads: Arc<[LevelRoadSegment]>,
    section: Arc<[LevelSections]>,
}

#[derive(Debug, Clone)]
struct LevelSections {
    top_left: Vec2,
    top_right: Vec2,
    bottom_left: Vec2,
    bottom_right: Vec2,
}

impl LevelSections {
    fn section_center(&self) -> Vec2 {
        (self.top_left + self.top_right + self.bottom_left + self.bottom_right) / 4.
    }

    fn size_min(&self) -> f32 {
        self.top_left
            .distance(self.bottom_right)
            .min(self.bottom_left.distance(self.top_right))
    }
    fn cross_axis_min_length(&self) -> f32 {
        self.top_left
            .distance(self.bottom_left)
            .min(self.top_right.distance(self.bottom_right))
    }
    fn main_axis_min_length(&self) -> f32 {
        self.top_left
            .distance(self.top_right)
            .min(self.bottom_left.distance(self.bottom_right))
    }
    fn subdivide_sections(&self, rng: &Rng) -> [LevelSections; 4] {
        let LevelSections {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        } = self;
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

    fn point_from_normalized(&self, point: Vec2) -> Vec2 {
        let x_point_1 = (self.bottom_right - self.bottom_left) * point.x + self.bottom_left;
        let x_point_2 = (self.top_right - self.top_left) * point.x + self.top_left;
        (x_point_2 - x_point_1) * point.y + x_point_1
    }
}

impl From<&(Vec2, Vec2, Vec2, Vec2)> for LevelSections {
    fn from((bottom_left, top_left, top_right, bottom_right): &(Vec2, Vec2, Vec2, Vec2)) -> Self {
        Self {
            top_left: *top_left,
            top_right: *top_right,
            bottom_left: *bottom_left,
            bottom_right: *bottom_right,
        }
    }
}

impl From<(Vec2, Vec2, Vec2, Vec2)> for LevelSections {
    fn from(value: (Vec2, Vec2, Vec2, Vec2)) -> Self {
        LevelSections::from(&value)
    }
}
struct LevelRoadSegment {
    start: Vec2,
    end: Vec2,
}
