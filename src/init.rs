use bevy::prelude::*;

#[derive(Resource)]
pub struct GameAssets {
    font: Handle<Font>,
    map: Handle<Scene>,
    tower_base_bright: Handle<Scene>,
    tower_base_purple: Handle<Scene>,
    tower_base_bad: Handle<Scene>,
}

pub enum Scenes {
    TowerBaseBright,
    TowerBasePurple,
    TowerBaseBad,
    Map,
}

impl GameAssets {
    pub fn scene(&self, scene: Scenes) -> Handle<Scene> {
        match scene {
            Scenes::TowerBaseBright => self.tower_base_bright.clone(),
            Scenes::TowerBasePurple => self.tower_base_purple.clone(),
            Scenes::TowerBaseBad => self.tower_base_bad.clone(),
            Scenes::Map => self.map.clone(),
        }
    }

    pub fn font(&self) -> Handle<Font> {
        self.font.clone()
    }
}

pub struct InitializationPlugin;

impl Plugin for InitializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(
            asset_loading.in_base_set(StartupSet::PreStartup),
        );
    }
}

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        font: assets.load("QuattrocentoSans-Bold.ttf"),
        map: assets.load("map_a_map_only.glb#Scene0"),
        tower_base_bright: assets.load("tower_base_a_bright.glb#Scene0"),
        tower_base_purple: assets.load("tower_base_a_purple.glb#Scene0"),
        tower_base_bad: assets.load("tower_base_bad.glb#Scene0"),
    });
}
