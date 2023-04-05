mod camera;
mod debug;
mod enemy;
mod graphics;
mod init;
mod pathmanager;
mod physics;
mod projectile;
mod tower;
mod world;

use bevy::{prelude::*, window::WindowMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy_mod_picking::*;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use debug::debug_plugin;
use graphics::graphics_plugin;
use pathmanager::path_manager_plugin;
use seldom_fn_plugin::FnPluginExt;

pub use camera::*;
pub use enemy::*;
pub use init::*;
pub use physics::*;
pub use projectile::*;
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
    .fn_plugin(initialization_plugin)
    .fn_plugin(path_manager_plugin)
    .fn_plugin(camera_plugin)
    .fn_plugin(world_plugin)
    .fn_plugin(tower_plugin)
    .fn_plugin(enemy_plugin)
    .add_plugin(ProjectilePlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(DefaultPickingPlugins)
    .fn_plugin(graphics_plugin)
    .fn_plugin(physics_plugin);

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new());
        app.fn_plugin(debug_plugin);
        app.add_plugin(RapierDebugRenderPlugin::default());
    }
    app
}
