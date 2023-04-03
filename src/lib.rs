mod camera;
mod init;
mod world;

use bevy::{prelude::*, window::WindowMode};

pub use camera::*;
pub use init::*;
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
    .add_plugin(WorldPlugin);
    app
}
