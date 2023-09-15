use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};
use bevy_xpbd_2d::prelude::{debug::PhysicsDebugConfig, *};
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use crate::{app_state::DrawDebugGizmos, assets::WithMesh};

use super::{
    player::Player,
    schedule::{InGamePostUpdate, InGamePreUpdate},
};

pub fn shadow_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGamePostUpdate, (check_for_shadow, spawn_shadow))
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
    if !matches!(gizmos.as_ref(), DrawDebugGizmos::InternalCircles) {
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
    check_for_shadow: Query<(Entity, &GlobalTransform, &CollidingEntities), With<CheckForShadow>>,
    mut commands: Commands,
) {
    for (entity, check, colliding) in check_for_shadow.iter() {
        let in_shadow = colliding.iter().any(|a| shadows.get(*a).is_ok());

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
    for (entity, transform, shadow) in &shadows {
        commands.entity(entity).insert((
            WithMesh::Shadow(shadow.radius),
            Collider::ball(shadow.radius),
            Sensor,
            CollidingEntities::default(),
            RigidBody::Static,
            Position(transform.translation.xy()),
        ));
    }
}
