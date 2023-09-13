use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use crate::{app_state::AppState, menus::credits::Credits};

pub struct MainGameAssetPlugin;

impl Plugin for MainGameAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::LoadingMenu).continue_to_state(AppState::MainMenu),
        )
        .add_collection_to_loading_state::<_, MainGameAssets>(AppState::LoadingMenu);
    }
}

#[derive(AssetCollection, Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct MainGameAssets {
    #[asset(path = "music/crossroad-blues-menu.flac")]
    pub menu_music: Handle<AudioSource>,
    #[asset(path = "music/crossroad-blues.flac")]
    pub game_music: Handle<AudioSource>,
    #[asset(path = "fonts/AMERSN__.ttf")]
    pub default_font: Handle<Font>,

    #[asset(path = "credits.cr.yaml")]
    pub credits: Handle<Credits>,

    #[asset(
        paths(
            "models/meshes.gltf#Mesh1/Primitive0",
            "models/meshes.gltf#Mesh2/Primitive0",
            "models/meshes.gltf#Mesh3/Primitive0",
            "models/meshes.gltf#Mesh4/Primitive0"
        ),
        collection(typed)
    )]
    pub grasses: Vec<Handle<Mesh>>,
    #[asset(paths("models/meshes.gltf#Mesh5/Primitive0"), collection(typed))]
    pub player: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh6/Primitive0",
            "models/meshes.gltf#Mesh7/Primitive0"
        ),
        collection(typed)
    )]
    pub roads: Vec<Handle<Mesh>>,
    #[asset(paths("models/meshes.gltf#Mesh8/Primitive0"), collection(typed))]
    pub lumbering_devil: Vec<Handle<Mesh>>,
    #[asset(paths("models/meshes.gltf#Mesh9/Primitive0"), collection(typed))]
    pub checkpoint: Vec<Handle<Mesh>>,
    #[asset(paths("models/meshes.gltf#Mesh10/Primitive0"), collection(typed))]
    pub person: Vec<Handle<Mesh>>,
    #[asset(paths("models/meshes.gltf#Mesh11/Primitive0"), collection(typed))]
    pub pentagram: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh12/Primitive0",
            "models/meshes.gltf#Mesh13/Primitive0"
        ),
        collection(typed)
    )]
    pub shadows: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh14/Primitive0",
            "models/meshes.gltf#Mesh15/Primitive0",
            "models/meshes.gltf#Mesh16/Primitive0"
        ),
        collection(typed)
    )]
    pub tree_trunks: Vec<Handle<Mesh>>,
}
