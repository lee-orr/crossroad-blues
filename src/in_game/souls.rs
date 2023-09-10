use bevy::prelude::*;

use super::shadow::InShadow;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Souls(pub f32);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MaxSouls(pub f32);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SunSensitivity(pub f32);

pub fn sun_sensitivity(
    mut sensitives: Query<(&mut Souls, &SunSensitivity), Without<InShadow>>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (mut souls, sensitivity) in sensitives.iter_mut() {
        souls.0 -= sensitivity.0 * delta;
    }
}
