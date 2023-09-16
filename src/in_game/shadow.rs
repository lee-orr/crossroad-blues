use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::{
    app_state::{AppState, DrawDebugGizmos},
    assets::WithMesh,
};

use super::schedule::InGamePreUpdate;

pub fn shadow_plugin(app: &mut ReloadableAppContents) {
    app.reset_resource::<ShadowCollisionGrid>()
        .add_systems(InGamePreUpdate, (check_for_shadow, spawn_shadow))
        .add_systems(PostUpdate, draw_shadow)
        .add_systems(OnExit(AppState::InGame), clear_grid);
}

const SHADOW_COLLISION_CELL_SIZE: f32 = 2000.;

#[derive(Resource, Default)]
struct ShadowCollisionGrid {
    map: HashMap<(i32, i32), HashSet<Entity>>,
}

#[derive(Component)]
pub struct Shadow {
    pub radius: f32,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct InShadow;

#[derive(Component)]
pub struct CheckForShadow;

fn draw_shadow(
    shadow: Query<(&GlobalTransform, &Shadow)>,
    mut painter: ShapePainter,
    gizmos: Res<DrawDebugGizmos>,
) {
    if !matches!(gizmos.as_ref(), DrawDebugGizmos::Collision) {
        return;
    }
    painter.color = Color::RED;
    for (trasnform, shadow) in shadow.iter() {
        painter.hollow = true;
        painter.set_translation(trasnform.translation());
        painter.circle(shadow.radius);
    }
}

fn check_for_shadow(
    shadows: Query<(&GlobalTransform, &Shadow)>,
    cells: Res<ShadowCollisionGrid>,
    check_for_shadow: Query<(Entity, &GlobalTransform), With<CheckForShadow>>,
    mut commands: Commands,
) {
    for (entity, check) in check_for_shadow.iter() {
        let check_position = check.translation();
        let cell = (
            (check_position.x / SHADOW_COLLISION_CELL_SIZE).floor() as i32,
            (check_position.y / SHADOW_COLLISION_CELL_SIZE).floor() as i32,
        );

        let in_shadow = if let Some(cell) = cells.map.get(&cell) {
            cell.iter()
                .filter_map(|v| shadows.get(*v).ok())
                .any(|(transform, shadow)| {
                    let position = transform.translation();
                    let distance = position.distance(check_position);
                    distance < shadow.radius
                })
        } else {
            false
        };

        if in_shadow {
            commands.entity(entity).insert(InShadow);
        } else {
            commands.entity(entity).remove::<InShadow>();
        }
    }
}

fn spawn_shadow(
    shadows: Query<(Entity, &GlobalTransform, &Shadow), (Without<Children>)>,
    mut commands: Commands,
    mut collision_grid: ResMut<ShadowCollisionGrid>,
) {
    for (entity, transform, shadow) in &shadows {
        let pos = transform.translation().xy();
        let bl = pos - Vec2::ONE * shadow.radius;
        let tr = pos + Vec2::ONE * shadow.radius;
        let bl = bl / SHADOW_COLLISION_CELL_SIZE;
        let tr = tr / SHADOW_COLLISION_CELL_SIZE;
        let bl = (bl.x.floor() as i32, bl.y.floor() as i32);
        let tr = (tr.x.floor() as i32, tr.y.floor() as i32);
        let mut cells = vec![];
        for x in bl.0..(tr.0 + 1) {
            for y in bl.1..(tr.1 + 1) {
                let cell = (x, y);
                let cell_container = collision_grid.map.entry(cell).or_default();
                cell_container.insert(entity);
                cells.push(cell);
            }
        }
        commands
            .entity(entity)
            .insert(WithMesh::Shadow(shadow.radius));
    }
}

fn clear_grid(mut commands: Commands) {
    commands.insert_resource(ShadowCollisionGrid::default());
}
