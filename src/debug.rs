use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

pub fn debug_plugin(app: &mut App) {
    app.add_plugin(DebugLinesPlugin::with_depth_test(false)); // Change to have
                                                              // lines NOT show
                                                              // through geometry
    app.add_system(locate_lights);
}

fn locate_lights(
    mut lines: ResMut<DebugLines>,
    lights: Query<&Transform, With<PointLight>>,
) {
    for pos in lights.iter() {
        lines.line(Vec3::ZERO, pos.translation, 0.0);
    }
}
