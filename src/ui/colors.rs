#![allow(dead_code)]
use bevy::prelude::{AmbientLight, Color};

pub const OVERLAY_COLOR: Color = Color::rgba(0., 0., 0., 0.9);
pub const BORDER_COLOR: Color = Color::rgb(0., 0., 0.02);

pub const SCREEN_BACKGROUND_COLOR: Color = Color::rgb(0.38, 0.17, 0.15);

pub const PRIMARY_BACKGROUND_COLOR: Color = Color::rgb(0.08, 0.05, 0.04);

pub const PRIMARY_COLOR: Color = Color::rgb(0.7, 0.4, 0.08);
pub const PRIMARY_COLOR_PRIORITIZED: Color = Color::rgb(0.45, 0.26, 0.06);
pub const PRIMARY_COLOR_FOCUSED: Color = PRIMARY_COLOR_PRIORITIZED;
pub const PRIMARY_COLOR_ACTIVE: Color = PRIMARY_COLOR_PRIORITIZED;
pub const PRIMARY_COLOR_BLOCKED: Color = Color::rgb(0.71, 0.6, 0.48);

pub const BAD_COLOR: Color = Color::rgb(0.93, 0.27, 0.27);

pub const DEFAULT_AMBIENT: AmbientLight = AmbientLight {
    color: Color::rgb(1., 1., 1.),
    brightness: 0.26,
};

pub const DEFAULT_CLEAR: Color = Color::rgb(0.68, 0.66, 0.62);
