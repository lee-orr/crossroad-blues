use bevy::prelude::{Component, Query, Transform};
use bevy_vector_shapes::{prelude::ShapePainter, shapes::DiscPainter};

#[derive(Component)]
pub struct Shadow {
    pub radius: f32,
}

pub fn draw_shadow(shadow: Query<(&Transform, &Shadow)>, mut painter: ShapePainter) {
    painter.color = crate::ui::colors::PRIMARY_BACKGROUND_COLOR;
    for (trasnform, shadow) in shadow.iter() {
        painter.set_translation(trasnform.translation);
        painter.circle(shadow.radius);
    }
}
