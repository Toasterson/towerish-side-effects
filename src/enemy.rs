use bevy::prelude::*;

use crate::{pathmanager::PathManager, GameAssets, Proxy};

#[derive(Reflect, Component)]
pub struct Waypoint {
    pub coords: Transform,
}

#[derive(Reflect, Component)]
pub struct Enemy {
    pub speed: f32,
}

#[derive(Reflect, Component)]
pub struct PathProgress {
    path: Entity,
    progress: f32,
}

impl PathProgress {
    pub fn new(path: Entity) -> Self {
        Self { path, progress: 0. }
    }
}

#[derive(Reflect, Component)]
pub struct Portal {
    pub spawn_timer: Timer,
}

impl Portal {
    pub fn new() -> Self {
        Self {
            spawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

pub fn enemy_plugin(app: &mut App) {
    app.register_type::<Waypoint>()
        .register_type::<Enemy>()
        .register_type::<Portal>()
        .register_type::<PathProgress>()
        .register_type::<PathManager>()
        .add_startup_system(example_setup)
        .add_system(enemy_spawner)
        .add_system(move_enemies.after(enemy_spawner));
}

fn example_setup(mut commands: Commands) {
    commands.spawn((
        (Name::new("Dummy Path")),
        PathManager::new(vec![
            Vec3::new(1., 5., 6.),
            Vec3::new(10., 5., -10.),
            Vec3::new(1., 5., 1.),
        ]),
    ));
}

fn enemy_spawner(
    mut commands: Commands,
    mut portal_query: Query<(&mut Portal, &Proxy, &GlobalTransform)>,
    paths: Query<Entity, With<PathManager>>,
    time: Res<Time>,
    assets: Res<GameAssets>,
) {
    let path = paths.get_single().unwrap(); // def change this later
    for (mut portal, proxy, portal_pos) in &mut portal_query {
        portal.spawn_timer.tick(time.delta());
        if portal.spawn_timer.just_finished() {
            let enemy = commands
                .spawn((
                    PbrBundle {
                        mesh: assets.get_capsule_shape().clone(),
                        transform: portal_pos
                            .compute_transform()
                            .with_rotation(Quat::from_xyzw(0.0, 0.0, 0.0, 0.0))
                            .with_scale(Vec3::new(1.0, 1.0, 1.0)),
                        ..Default::default()
                    },
                    Name::new("Enemy"),
                    Enemy { speed: 3. },
                    PathProgress::new(path),
                ))
                .id();
        }
    }
}

fn move_enemies(
    mut enemies: Query<(&Enemy, &mut Transform, &mut PathProgress)>,
    paths: Query<&PathManager>,
    time: Res<Time>,
) {
    for (enemy, mut transform, mut progress) in &mut enemies {
        progress.progress += enemy.speed * time.delta_seconds();
        transform.translation = paths
            .get(progress.path)
            .unwrap()
            .get_position(progress.progress);
    }
}
