use std::{f32::consts, sync::Arc, time::Duration};

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
    checkpoints::Checkpoint, danger::LumberingDevil, game_state::GameState, movement::CanMove,
    player::ConstructPlayer, ritual::Person, shadow::Shadow, InGame,
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
    pub curviness: f32,
    pub num_segments: usize,
}

impl Default for CurrentLevel {
    fn default() -> Self {
        Self {
            song: "music/crossroad-blues.flac".to_string(),
            song_length: Duration::from_secs(60),
            bg_color: SCREEN_BACKGROUND_COLOR,
            ambient: DEFAULT_AMBIENT,
            curviness: 120.,
            num_segments: 3,
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

    let default_move = CanMove::default().move_speed;

    let level_shapes = define_level_shape(
        rng,
        level.song_length.as_secs_f32() * default_move * (0.5 + 0.3 * rng.f32()) / 2.,
        level.curviness,
        level.num_segments,
    );

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
                    transform: Transform::from_translation(
                        level_shapes.target_start_point.extend(0.),
                    ),
                    ..Default::default()
                },
                Person(
                    level.song_length.as_secs_f32(),
                    level_shapes.target_path.clone(),
                ),
            ));

            p.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(
                        level_shapes.player_start_point.extend(0.),
                    ),
                    ..Default::default()
                },
                ConstructPlayer,
            ));
        });

    commands.insert_resource(NextState(Some(GameState::InGame)));
}

fn define_level_shape(rng: &Rng, length: f32, curviness: f32, segments: usize) -> LevelShape {
    let start_pos = Vec2::ZERO;
    let crossroads = Vec2::X * length + Vec2::Y * curviness * rng.f32_normalized();
    let end_pos = crossroads + Vec2::X * 600. + Vec2::Y * curviness * rng.f32_normalized();

    let segments = ([0f32].into_iter())
        .chain(
            (1..segments)
                .map(|v| v as f32)
                .map(|v| v + rng.f32_normalized() * 0.2)
                .map(|v| v / (segments as f32)),
        )
        .chain([1f32])
        .collect::<Box<[_]>>();

    let target_path = segments
        .iter()
        .map(|v| {
            (
                start_pos.lerp(crossroads, *v) + Vec2::Y * curviness * rng.f32_normalized(),
                *v,
            )
        })
        .collect::<Arc<[_]>>();

    info!("Setting up path {target_path:?}");

    let crossroads = target_path.last().map(|(v, _)| *v).unwrap_or(crossroads);

    let cross_road_points = [
        crossroads - Vec2::Y * 600.,
        crossroads,
        crossroads + Vec2::Y * 600.,
    ];

    let roads = target_path
        .iter()
        .map_windows(|[a, b]| LevelRoadSegment {
            start: a.0,
            end: b.0,
        })
        .chain(
            cross_road_points
                .iter()
                .map_windows(|[a, b]| LevelRoadSegment {
                    start: **a,
                    end: **b,
                }),
        )
        .chain([LevelRoadSegment {
            start: crossroads,
            end: end_pos,
        }])
        .collect::<Arc<[_]>>();

    let sections = target_path
        .iter()
        .map_windows(|[a, b]| {
            (
                a.0 + Vec2::Y * 50. - Vec2::X * 50.,
                a.0 + Vec2::Y * 650. - Vec2::X * 50.,
                b.0 + Vec2::Y * 650. - Vec2::X * 50.,
                b.0 + Vec2::Y * 50. - Vec2::X * 50.,
            )
        })
        .chain(target_path.iter().map_windows(|[a, b]| {
            (
                a.0 - Vec2::Y * 650. - Vec2::X * 50.,
                a.0 - Vec2::Y * 50. - Vec2::X * 50.,
                b.0 - Vec2::Y * 50. - Vec2::X * 50.,
                b.0 - Vec2::Y * 650. - Vec2::X * 50.,
            )
        }))
        .chain([
            (
                crossroads - Vec2::Y * 650.,
                crossroads - Vec2::Y * 50. + Vec2::X * 50.,
                end_pos - Vec2::Y * 50.,
                end_pos - Vec2::Y * 300.,
            ),
            (
                crossroads + Vec2::Y * 650.,
                crossroads + Vec2::Y * 50. + Vec2::X * 50.,
                end_pos + Vec2::Y * 50.,
                end_pos + Vec2::Y * 300.,
            ),
        ])
        .map(LevelSections::from)
        .collect();

    LevelShape {
        crossroads,
        roads,
        target_start_point: target_path
            .first()
            .cloned()
            .map(|v| v.0)
            .unwrap_or_default(),
        target_path,
        section: sections,
        player_start_point: start_pos
            + 230. * Vec2::Y
            + (100. * rng.f32_normalized())
            + Vec2::X * 200. * rng.f32(),
    }
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

pub struct LevelShape {
    pub crossroads: Vec2,
    pub roads: Arc<[LevelRoadSegment]>,
    pub target_path: Arc<[(Vec2, f32)]>,
    pub section: Arc<[LevelSections]>,
    pub player_start_point: Vec2,
    pub target_start_point: Vec2,
}

#[derive(Debug, Clone)]
pub struct LevelSections {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_left: Vec2,
    pub bottom_right: Vec2,
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
pub struct LevelRoadSegment {
    pub start: Vec2,
    pub end: Vec2,
}
