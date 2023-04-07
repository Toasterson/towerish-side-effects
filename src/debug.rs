use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

use crate::pathmanager::PathManager;

pub fn debug_plugin(app: &mut App) {
    // app.add_system(locate_lights);
    app.add_system(display_path);
    app.add_plugin(DebugLinesPlugin::with_depth_test(false)); // Change to have
                                                              // lines NOT show
                                                              // through geometry
}

#[allow(dead_code)]
fn locate_lights(
    mut lines: ResMut<DebugLines>,
    lights: Query<&Transform, With<PointLight>>,
) {
    for pos in lights.iter() {
        lines.line(Vec3::ZERO, pos.translation, 0.0);
    }
}

fn display_path(mut lines: ResMut<DebugLines>, paths: Query<&PathManager>) {
    for path in paths.iter() {
        for (a, b) in path.waypoints.iter().zip(path.waypoints.iter().skip(1)) {
            lines.line(a.location, b.location, 0.0);
        }
    }
}
