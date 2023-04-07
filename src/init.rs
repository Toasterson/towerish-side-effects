use bevy::{gltf::Gltf, prelude::*};

#[derive(Resource)]
pub struct GameAssets {
    font: Handle<Font>,
    map: Handle<Gltf>,
    map_collider: Handle<Mesh>,
    tower_base_bright: Handle<Scene>,
    tower_base_purple: Handle<Scene>,
    tower_base_bad: Handle<Scene>,
    capsule_shape: Handle<Mesh>,
    pub tower_slice_a: Handle<Scene>,
    pub ring_a: Handle<Scene>,
    pub gun_a: Handle<Scene>,
    pub default_collider_color: Handle<StandardMaterial>,
    pub tower_base_selected_color: Handle<StandardMaterial>,
    pub enemy_color: Handle<StandardMaterial>,
}

pub enum Scenes {
    TowerBaseBright,
    TowerBasePurple,
    TowerBaseBad,
}

impl GameAssets {
    pub fn scene(&self, scene: Scenes) -> Handle<Scene> {
        match scene {
            Scenes::TowerBaseBright => self.tower_base_bright.clone(),
            Scenes::TowerBasePurple => self.tower_base_purple.clone(),
            Scenes::TowerBaseBad => self.tower_base_bad.clone(),
        }
    }

    pub fn font(&self) -> Handle<Font> {
        self.font.clone()
    }

    pub fn map(&self) -> &Handle<Gltf> {
        &self.map
    }
    pub fn get_capsule_shape(&self) -> &Handle<Mesh> {
        &self.capsule_shape
    }
}

pub fn initialization_plugin(app: &mut App) {
    app.add_startup_system(asset_loading.in_base_set(StartupSet::PreStartup));
}

fn asset_loading(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let default_collider_color = materials.add(Color::NONE.into());
    let tower_base_selected_color =
        materials.add(Color::rgba(0.3, 0.9, 0.3, 0.9).into());
    let capsule_shape = meshes.add(shape::Capsule::default().into());

    commands.insert_resource(GameAssets {
        font: assets.load("QuattrocentoSans-Bold.ttf"),
        map: assets.load("map_a_0.2.glb"),
        map_collider: assets.load("map_a_collision.glb#Mesh0/Primitive0"),
        tower_base_bright: assets.load("tower_base_a_bright.glb#Scene0"),
        tower_base_purple: assets.load("tower_base_a_purple.glb#Scene0"),
        tower_base_bad: assets.load("tower_base_bad.glb#Scene0"),
        capsule_shape,
        tower_slice_a: assets.load("tower_slice_a.glb#Scene0"),
        gun_a: assets.load("gun_a.glb#Scene0"),
        ring_a: assets.load("ring_a#Scene0"),
        enemy_color: tower_base_selected_color.clone(),
        tower_base_selected_color,
        default_collider_color,
    });
}
