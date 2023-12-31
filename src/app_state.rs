use bevy::{prelude::*, reflect::Reflect};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

#[derive(Clone, Eq, PartialEq, Copy, Debug, Hash, Default, States, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub enum AppState {
    #[default]
    LoadingMenu,
    MainMenu,
    Credits,
    InGame,
    Levels,
    ToNextLevel,
}

#[derive(Resource, Default)]
pub enum DrawDebugGizmos {
    #[default]
    None,
    Collision,
}
