use bevy::prelude::*;

use crate::{
    pathmanager::PathManager, GameAssets, GameState, Health, PhysicsBundle,
    StateUpdateEvent, UiState,
};

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
            spawn_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
        }
    }
}

pub fn enemy_plugin(app: &mut App) {
    app.register_type::<Waypoint>()
        .register_type::<Enemy>()
        .register_type::<Portal>()
        .register_type::<PathProgress>()
        .add_system(enemy_spawner)
        .add_system(move_enemies.after(enemy_spawner))
        .add_system(enemy_death);
}

fn enemy_death(
    mut commands: Commands,
    targets: Query<(Entity, &Health)>,
    mut ev_status_update: EventWriter<StateUpdateEvent>,
) {
    for (ent, health) in &targets {
        if health.value <= 0.0 {
            info!("Enemy {:?} died", ent);
            commands.entity(ent).despawn_recursive();
            ev_status_update.send(StateUpdateEvent::EnemyKilled(50.0));
        }
    }
}

fn enemy_spawner(
    mut commands: Commands,
    mut portal_query: Query<&mut Portal>,
    paths: Query<(Entity, &PathManager)>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    ui_state: Res<UiState>,
) {
    for mut portal in &mut portal_query {
        if matches!(ui_state.game_state, GameState::RunningWave) {
            if portal.spawn_timer.paused() {
                portal.spawn_timer.unpause();
            }
            portal.spawn_timer.tick(time.delta());
            if portal.spawn_timer.just_finished() {
                match paths.get_single() {
                    Ok((path, path_manager)) => {
                        if let Some(path_start) = path_manager.get_start() {
                            commands.spawn((
                                PbrBundle {
                                    mesh: assets.get_capsule_shape().clone(),
                                    transform: Transform::from_translation(
                                        path_start.location,
                                    )
                                    .with_scale(Vec3::new(1.0, 1.0, 1.0)),
                                    ..Default::default()
                                },
                                Name::new("Enemy"),
                                Enemy { speed: 1.5 },
                                Health { value: 2.0 },
                                PathProgress::new(path),
                                PhysicsBundle::moving_entity().make_kinematic(),
                            ));
                        }
                    }
                    Err(_) => {}
                }
            } else if matches!(ui_state.game_state, GameState::TowerUpgrade) {
                portal.spawn_timer.pause();
            }
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
