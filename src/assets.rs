use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};

use crate::{app_state::AppState, in_game::Levels, menus::credits::Credits};

pub struct MainGameAssetPlugin;

pub const ROAD_TILE_SIZE: f32 = 50.;

impl Plugin for MainGameAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::LoadingMenu).continue_to_state(AppState::MainMenu),
        )
        .add_collection_to_loading_state::<_, MainGameAssets>(AppState::LoadingMenu)
        .add_systems(PostUpdate, spawn_mesh)
        .init_resource::<MainColorMaterial>();
    }
}

#[derive(Component, Clone, Copy)]
pub enum WithMesh {
    Player,
    HolyHulk,
    Checkpoint,
    Shadow(f32),
    RoadTile,
    PentagramCircle,
    PentagramTriangle(f32),
    Person,
    StealthySeraphim,
    GuardianAngel,
    AngelicArchers,
    AngelicArrow,
}

fn spawn_mesh(
    mut commands: Commands,
    meshes: Query<(Entity, &WithMesh)>,
    material: Res<MainColorMaterial>,
    assets: Option<Res<MainGameAssets>>,
    mut rng: ResMut<GlobalRng>,
) {
    let Some(assets) = assets else {
        return;
    };
    let rng = rng.get_mut();
    for (entity, with_mesh) in &meshes {
        let mut transform = Transform::from_scale(30. * Vec3::ONE)
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                0., //90f32.to_radians(),
                0., //-90f32.to_radians(),
                0.,
            ))
            .with_translation(Vec3::NEG_Z * 5.);

        let mesh = match with_mesh {
            WithMesh::Player => {
                transform.translation.z += 3.;
                assets.player.clone()
            }
            WithMesh::HolyHulk => {
                transform.translation.z += 2.;
                assets.holy_hulk.clone()
            }
            WithMesh::Checkpoint => {
                transform.translation.z += 4.;
                assets.checkpoint.clone()
            }
            WithMesh::Shadow(r) => {
                transform.scale = Vec3::ONE * 1.8 * *r;
                rng.sample(&assets.shadows).unwrap().clone()
            }
            WithMesh::RoadTile => {
                transform.translation.z += 1.;
                rng.sample(&assets.roads).unwrap().clone()
            }
            WithMesh::PentagramCircle => {
                transform.translation.z += 1.3;
                assets.pentagram_circle.clone()
            }
            WithMesh::Person => {
                transform.translation.z += 1.8;
                rng.sample(&assets.people).unwrap().clone()
            }
            WithMesh::PentagramTriangle(angle) => {
                transform.translation.z += 1.3;
                transform.rotate_z(*angle);
                assets.pentagram_triangle.clone()
            }
            WithMesh::StealthySeraphim => {
                transform.translation.z += 2.;
                assets.stealthy_seraphim.clone()
            }
            WithMesh::GuardianAngel => {
                transform.translation.z += 2.;
                assets.guardian_angel.clone()
            }
            WithMesh::AngelicArchers => {
                transform.translation.z += 2.;
                assets.angelic_archer.clone()
            }
            WithMesh::AngelicArrow => {
                transform.translation.z += 2.5;
                assets.angelic_archer_arrow.clone()
            }
        };
        let mesh = Mesh2dHandle(mesh.clone());
        let Some(mut e) = commands.get_entity(entity) else {
            error!("Entity doesn't exist");
            continue;
        };
        e.remove::<WithMesh>().with_children(|p| {
            p.spawn(MaterialMesh2dBundle {
                mesh,
                material: material.color_material.clone(),
                transform,
                ..Default::default()
            });
        });
    }
}
#[derive(Resource)]
pub struct MainColorMaterial {
    pub color_material: Handle<ColorMaterial>,
}

impl FromWorld for MainColorMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<ColorMaterial>>()
            .expect("Couldn't get Asset for ColorMaterial");

        let color_material = materials.add(ColorMaterial::default());
        Self { color_material }
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

    #[asset(path = "textures/checkpoint-empty.png")]
    pub checkpoint_empty: Handle<Image>,
    #[asset(path = "textures/checkpoint-full.png")]
    pub checkpoint_full: Handle<Image>,

    #[asset(path = "credits.cr.yaml")]
    pub credits: Handle<Credits>,

    #[asset(
        paths(
            "models/meshes.gltf#Mesh0/Primitive0",
            "models/meshes.gltf#Mesh1/Primitive0",
            "models/meshes.gltf#Mesh2/Primitive0",
            "models/meshes.gltf#Mesh3/Primitive0"
        ),
        collection(typed)
    )]
    pub grasses: Vec<Handle<Mesh>>,
    #[asset(path = "models/meshes.gltf#Mesh4/Primitive0")]
    pub player: Handle<Mesh>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh5/Primitive0",
            "models/meshes.gltf#Mesh6/Primitive0",
            "models/meshes.gltf#Mesh17/Primitive0",
            "models/meshes.gltf#Mesh18/Primitive0",
        ),
        collection(typed)
    )]
    pub roads: Vec<Handle<Mesh>>,
    #[asset(path = "models/meshes.gltf#Mesh7/Primitive0")]
    pub lumbering_devil: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh8/Primitive0")]
    pub checkpoint: Handle<Mesh>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh9/Primitive0",
            "models/meshes.gltf#Mesh25/Primitive0",
            "models/meshes.gltf#Mesh26/Primitive0"
        ),
        collection(typed)
    )]
    pub people: Vec<Handle<Mesh>>,
    #[asset(path = "models/meshes.gltf#Mesh10/Primitive0")]
    pub pentagram_circle: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh16/Primitive0")]
    pub pentagram_triangle: Handle<Mesh>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh11/Primitive0",
            "models/meshes.gltf#Mesh12/Primitive0"
        ),
        collection(typed)
    )]
    pub shadows: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh13/Primitive0",
            "models/meshes.gltf#Mesh14/Primitive0",
            "models/meshes.gltf#Mesh15/Primitive0"
        ),
        collection(typed)
    )]
    pub tree_trunks: Vec<Handle<Mesh>>,
    #[asset(path = "models/meshes.gltf#Mesh19/Primitive0")]
    pub holy_hulk: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh20/Primitive0")]
    pub stealthy_seraphim: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh21/Primitive0")]
    pub angelic_archer: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh22/Primitive0")]
    pub angelic_archer_arrow: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh23/Primitive0")]
    pub holy_smokes: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh24/Primitive0")]
    pub guardian_angel: Handle<Mesh>,
    #[asset(path = "levels.lvl.yaml")]
    pub levels: Handle<Levels>,
}
