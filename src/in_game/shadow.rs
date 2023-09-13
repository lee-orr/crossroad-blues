use bevy::prelude::*;
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::{app_state::DrawDebugGizmos, assets::WithMesh};

use super::schedule::InGamePreUpdate;

pub fn shadow_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGamePreUpdate, (check_for_shadow, spawn_shadow))
        .add_systems(PostUpdate, draw_shadow);
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

pub fn draw_shadow(
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

pub fn check_for_shadow(
    shadows: Query<(&GlobalTransform, &Shadow)>,
    check_for_shadow: Query<(Entity, &GlobalTransform), With<CheckForShadow>>,
    mut commands: Commands,
) {
    for (entity, check) in check_for_shadow.iter() {
        let check_position = check.translation();
        let in_shadow = shadows.iter().any(|(transform, shadow)| {
            let position = transform.translation();
            let distance = position.distance(check_position);
            distance < shadow.radius
        });

        if in_shadow {
            commands.entity(entity).insert(InShadow);
        } else {
            commands.entity(entity).remove::<InShadow>();
        }
    }
}

fn spawn_shadow(
    shadows: Query<(Entity, &Transform, &Shadow), (Without<Children>, Without<WithMesh>)>,
    mut commands: Commands,
) {
    for (entity, _transform, shadow) in &shadows {
        commands
            .entity(entity)
            .insert(WithMesh::Shadow(shadow.radius));
    }
}
