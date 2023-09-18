use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use bevy_turborand::{DelegatedRng, GlobalRng, TurboRand};
use serde::Deserialize;

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
        .init_resource::<MainColorMaterial>()
        .init_resource::<Locale>();
    }
}

#[derive(Copy, Clone, Default, Reflect, Deserialize, Resource)]
pub enum Locale {
    #[default]
    Forest,
    Hell,
    Snow,
}

impl Locale {
    pub fn bg_color(&self) -> Color {
        match self {
            Locale::Forest => Color::rgb(0.3, 0.39, 0.05),
            Locale::Hell => Color::rgb(0.22, 0.03, 0.02),
            Locale::Snow => Color::rgb(0.87, 0.88, 0.93),
        }
    }
}

#[derive(Component, Clone)]
#[allow(dead_code)]
pub enum WithMesh {
    Player,
    HolyHulk,
    HolyHulkFace,
    Checkpoint,
    Shadow(f32),
    RoadTile,
    PentagramCircle,
    PentagramTriangle(f32),
    PentagramFail,
    Person,
    StealthySeraphim,
    StealthySeraphimFace,
    GuardianAngel,
    AngelicArchers,
    AngelicArchersFace,
    AngelicArrow,
    DivineDetonator,
    DivineDetonatorFace,
    DivineDetonatorExplosion,
    PlayerFace,
    PlayerDead,
    PlayerCelebrate,
    DevilPretendingFace,
    GuardianAngelFace,
    DevilPretending,
    LumberingDevil,
    DevilFace,
    Sunlight,
    Decor,
    Handle(Handle<Mesh>),
}

fn spawn_mesh(
    mut commands: Commands,
    meshes: Query<(Entity, &WithMesh)>,
    material: Res<MainColorMaterial>,
    assets: Option<Res<MainGameAssets>>,
    locale: Res<Locale>,
    mut rng: ResMut<GlobalRng>,
) {
    let Some(assets) = assets else {
        return;
    };
    let rng = rng.get_mut();
    for (entity, with_mesh) in &meshes {
        let mut transform = Transform::from_scale(Vec3::new(30., 30., 1.))
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
                transform.scale = Vec3::new(1., 1., 0.2) * 1.8 * *r;
                let list = match locale.as_ref() {
                    Locale::Forest => &assets.shadows,
                    Locale::Hell => &assets.hell_shadows,
                    Locale::Snow => &assets.snow_shadows,
                };
                rng.sample(list).unwrap().clone()
            }
            WithMesh::RoadTile => {
                transform.translation.z += 1.;
                let list = match locale.as_ref() {
                    Locale::Forest => &assets.roads,
                    Locale::Hell => &assets.hellish_roads,
                    Locale::Snow => &assets.snowy_roads,
                };
                rng.sample(list).unwrap().clone()
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
            WithMesh::DivineDetonator => {
                transform.translation.z += 2.;
                assets.divine_detonator.clone()
            }
            WithMesh::DivineDetonatorExplosion => {
                transform.translation.z += 2.5;
                assets.divine_detonator_explosion.clone()
            }
            WithMesh::PlayerFace => assets.player_face.clone(),
            WithMesh::DevilPretendingFace => assets.devil_pretending_face.clone(),
            WithMesh::GuardianAngelFace => assets.guardian_angel_face.clone(),
            WithMesh::PlayerDead => assets.player_dead.clone(),
            WithMesh::PlayerCelebrate => assets.player_celebrate.clone(),
            WithMesh::HolyHulkFace => assets.holy_hulk_face.clone(),
            WithMesh::PentagramFail => assets.pentagram.clone(),
            WithMesh::StealthySeraphimFace => assets.stealthy_seraphim_face.clone(),
            WithMesh::AngelicArchersFace => assets.angelic_archer_face.clone(),
            WithMesh::DivineDetonatorFace => assets.divine_detonator_face.clone(),
            WithMesh::DevilPretending => assets.devil_pretending.clone(),
            WithMesh::LumberingDevil => assets.lumbering_devil.clone(),
            WithMesh::DevilFace => assets.devil_face.clone(),
            WithMesh::Sunlight => assets.sunlight.clone(),
            WithMesh::Handle(h) => {
                transform.translation.z += 2.;
                h.clone()
            }
            WithMesh::Decor => {
                transform.translation.z += 1.4;
                let list = match locale.as_ref() {
                    Locale::Forest => &assets.grasses,
                    Locale::Hell => continue,
                    Locale::Snow => continue,
                };
                rng.sample(list).unwrap().clone()
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

        let color_material = materials.add(ColorMaterial {
            ..Default::default()
        });
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
    #[asset(path = "fonts/Roman Antique.ttf")]
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
    #[asset(path = "models/meshes.gltf#Mesh34/Primitive0")]
    pub player_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh40/Primitive0")]
    pub player_dead: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh39/Primitive0")]
    pub player_celebrate: Handle<Mesh>,
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
    #[asset(
        paths(
            "models/meshes.gltf#Mesh42/Primitive0",
            "models/meshes.gltf#Mesh43/Primitive0",
        ),
        collection(typed)
    )]
    pub hellish_roads: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh44/Primitive0",
            "models/meshes.gltf#Mesh50/Primitive0",
        ),
        collection(typed)
    )]
    pub snowy_roads: Vec<Handle<Mesh>>,
    #[asset(path = "models/meshes.gltf#Mesh7/Primitive0")]
    pub lumbering_devil: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh28/Primitive0")]
    pub devil_face: Handle<Mesh>,
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
            "models/meshes.gltf#Mesh12/Primitive0",
            "models/meshes.gltf#Mesh45/Primitive0",
        ),
        collection(typed)
    )]
    pub shadows: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh46/Primitive0",
            "models/meshes.gltf#Mesh47/Primitive0"
        ),
        collection(typed)
    )]
    pub hell_shadows: Vec<Handle<Mesh>>,
    #[asset(
        paths(
            "models/meshes.gltf#Mesh48/Primitive0",
            "models/meshes.gltf#Mesh49/Primitive0"
        ),
        collection(typed)
    )]
    pub snow_shadows: Vec<Handle<Mesh>>,
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
    #[asset(path = "models/meshes.gltf#Mesh29/Primitive0")]
    pub holy_hulk_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh20/Primitive0")]
    pub stealthy_seraphim: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh30/Primitive0")]
    pub stealthy_seraphim_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh21/Primitive0")]
    pub angelic_archer: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh22/Primitive0")]
    pub angelic_archer_arrow: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh31/Primitive0")]
    pub angelic_archer_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh23/Primitive0")]
    pub divine_detonator: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh33/Primitive0")]
    pub divine_detonator_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh27/Primitive0")]
    pub divine_detonator_explosion: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh24/Primitive0")]
    pub guardian_angel: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh32/Primitive0")]
    pub guardian_angel_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh35/Primitive0")]
    pub devil_pretending: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh36/Primitive0")]
    pub devil_pretending_face: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh37/Primitive0")]
    pub sunlight: Handle<Mesh>,
    #[asset(path = "models/meshes.gltf#Mesh38/Primitive0")]
    pub pentagram: Handle<Mesh>,
    #[asset(path = "levels.lvl.yaml")]
    pub levels: Handle<Levels>,
}
