use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::mouse::MouseWheel,
    prelude::*,
};
#[cfg(feature = "particles")]
use bevy_atmosphere::prelude::*;
use bevy_mod_picking::PickingCameraBundle;

pub fn camera_plugin(app: &mut App) {
    #[cfg(feature = "particles")]
    app.add_plugin(AtmospherePlugin);
    app.add_startup_system(spawn_camera)
        .add_system(camera_controls);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-6.0, 18.1, 16.5)
                .looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: false,
                ..Default::default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..Default::default()
        },
        Name::new("Camera"),
        BloomSettings {
            // intensity: 0.25,
            ..Default::default()
        },
        #[cfg(feature = "particles")]
        AtmosphereCamera::default(),
        PickingCameraBundle::default(),
    ));
}

fn camera_controls(
    keyboard: Res<Input<KeyCode>>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    let mut camera = camera_query.single_mut();

    let mut forward = camera.forward();
    forward.y = 0.0;
    forward = forward.normalize();

    let mut left = camera.left();
    left.y = 0.0;
    left = left.normalize();

    let speed = if keyboard.pressed(KeyCode::LShift) {
        15.0
    } else {
        7.0
    };
    let rotate_speed = if keyboard.pressed(KeyCode::LShift) {
        1.4
    } else {
        0.6
    };
    let scroll_speed = 32.0;

    for ev in scroll_evr.iter() {
        match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                camera.translation.y +=
                    ev.y * time.delta_seconds() * scroll_speed
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                camera.translation.y +=
                    ev.y * time.delta_seconds() * scroll_speed
            }
        }
    }

    if keyboard.pressed(KeyCode::W) {
        camera.translation += forward * time.delta_seconds() * speed;
    }

    if keyboard.pressed(KeyCode::S) {
        camera.translation -= forward * time.delta_seconds() * speed;
    }

    if keyboard.pressed(KeyCode::A) {
        camera.translation += left * time.delta_seconds() * speed;
    }

    if keyboard.pressed(KeyCode::D) {
        camera.translation -= left * time.delta_seconds() * speed;
    }

    if keyboard.pressed(KeyCode::E) {
        camera.rotate_axis(Vec3::Y, rotate_speed * time.delta_seconds());
    }

    if keyboard.pressed(KeyCode::Q) {
        camera.rotate_axis(Vec3::Y, -rotate_speed * time.delta_seconds());
    }
}
