use bevy::prelude::*;
use dexterous_developer::{ReloadableApp, ReloadableAppContents};

use super::{schedule::InGameUpdate, shadow::InShadow};

pub fn souls_plugin(app: &mut ReloadableAppContents) {
    app.add_systems(InGameUpdate, ((sun_sensitivity, take_damage).chain(),));
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Souls(pub f32);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MaxSouls(pub f32);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SunSensitivity(pub f32);

#[derive(Event, Clone, Copy, Debug)]
pub struct Damage {
    pub entity: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}

#[derive(Clone, Copy, Debug)]
pub enum DamageType {
    Sunlight,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct Death {
    pub entity: Entity,
    pub cause: DamageType,
}

pub fn sun_sensitivity(
    sensitives: Query<(Entity, &SunSensitivity), Without<InShadow>>,
    mut writer: EventWriter<Damage>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    for (entity, sensitivity) in sensitives.iter() {
        let amount = sensitivity.0 * delta;
        writer.send(Damage {
            entity,
            amount,
            damage_type: DamageType::Sunlight,
        });
    }
}

pub fn take_damage(
    mut souls: Query<&mut Souls>,
    mut events: EventReader<Damage>,
    mut death: EventWriter<Death>,
) {
    for event in events.iter() {
        let Ok(mut souls) = souls.get_mut(event.entity) else {
            continue;
        };
        souls.0 -= event.amount;

        if souls.0 <= 0. {
            death.send(Death {
                entity: event.entity,
                cause: event.damage_type,
            });
        }
    }
}
