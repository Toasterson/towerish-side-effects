mod camera;
mod init;
mod tower;
mod world;

use bevy::{prelude::*, window::WindowMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_mod_picking::*;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

pub use camera::*;
pub use init::*;
pub use tower::*;
pub use world::*;

pub const LAUNCHER_TITLE: &str = "Towering Sideffects";

pub fn app(fullscreen: bool) -> App {
    let mode = if fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: LAUNCHER_TITLE.to_string(),
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
            mode,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
    .add_plugin(InitializationPlugin)
    .add_plugin(CameraPlugin)
    .add_plugin(WorldPlugin)
    .add_plugin(TowerPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(DefaultPickingPlugins);

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new());
        // app.add_plugin(RapierDebugRenderPlugin::default());
    }
    app
}
