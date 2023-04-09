use std::time::Duration;

use bevy::{gltf::Gltf, prelude::*};

use crate::{
    pathmanager::PathManager, GameAssets, GameState, HitEvent, PhysicsBundle,
    StateUpdateEvent,
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
pub struct Health {
    pub value: f32,
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

pub fn enemy_plugin(app: &mut App) {
    app.register_type::<Waypoint>()
        .register_type::<Enemy>()
        .register_type::<PathProgress>()
        .insert_resource(WaveState::default())
        .add_system(enemy_spawner)
        .add_system(move_enemies.after(enemy_spawner))
        .add_system(hit_event_handler)
        .add_system(state_update_handler)
        .add_system(enemy_reaches_portal_handler)
        .add_system(wave_timer_system);
}

fn wave_timer_system(
    mut state: ResMut<WaveState>,
    mut ev_game_state: EventWriter<StateUpdateEvent>,
    time: Res<Time>,
) {
    state.timer.tick(time.delta());
    if state.timer.just_finished() {
        ev_game_state.send(StateUpdateEvent::EndWave);
    }
}

fn hit_event_handler(
    mut ev_hit: EventReader<HitEvent>,
    mut enemies: Query<(Entity, &mut Health), With<Enemy>>,
    mut commands: Commands,
    mut ev_status_update: EventWriter<StateUpdateEvent>,
) {
    for event in ev_hit.iter() {
        for (ent, mut health) in &mut enemies {
            if ent == event.entity {
                health.value -= 0.1 * event.force;
                if health.value <= 0.0 {
                    info!("Enemy {:?} died", ent);
                    commands.entity(ent).despawn_recursive();
                    ev_status_update.send(StateUpdateEvent::EnemyKilled(50.0));
                }
            }
        }
    }
}

#[derive(Resource)]
struct WaveState {
    game_state: GameState,
    timer: Timer,
    spawn_timer: Timer,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            game_state: GameState::TowerUpgrade,
            timer: Timer::from_seconds(40.0, TimerMode::Once),
            spawn_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
        }
    }
}

fn state_update_handler(
    mut wave_state: ResMut<WaveState>,
    mut ev_status_reader: EventReader<StateUpdateEvent>,
) {
    for event in ev_status_reader.iter() {
        match event {
            StateUpdateEvent::StartWave {
                spawn_interval,
                time_of_wave,
            } => {
                wave_state.game_state = GameState::RunningWave;
                wave_state
                    .timer
                    .set_duration(Duration::from_secs_f32(*time_of_wave));
                wave_state.timer.reset();
                wave_state
                    .spawn_timer
                    .set_duration(Duration::from_secs_f32(*spawn_interval));
                wave_state.spawn_timer.reset();
                wave_state.spawn_timer.unpause();
            }
            StateUpdateEvent::GameWon => {
                wave_state.game_state = GameState::GameWon;
                wave_state.spawn_timer.pause();
            }
            StateUpdateEvent::GameLost => {
                wave_state.game_state = GameState::GameLost;
                wave_state.spawn_timer.pause();
            }
            StateUpdateEvent::EndWave => {
                wave_state.game_state = GameState::TowerUpgrade;
                wave_state.spawn_timer.pause();
            }
            _ => {}
        }
    }
}

fn enemy_spawner(
    mut commands: Commands,
    mut wave_state: ResMut<WaveState>,
    paths: Query<(Entity, &PathManager)>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if !matches!(wave_state.game_state, GameState::TowerUpgrade) {
        wave_state.spawn_timer.tick(time.delta());
        if wave_state.spawn_timer.just_finished() {
            match paths.get_single() {
                Ok((path, path_manager)) => {
                    if let Some(path_start) = path_manager.get_start() {
                        let drone = assets_gltf
                            .get(&assets.enemy_observer_drone)
                            .unwrap();
                        let mut player = AnimationPlayer::default();

                        if let Some(animation) = drone.animations.first() {
                            player.play(animation.clone_weak()).repeat();
                        }

                        commands
                            .spawn((
                                SpatialBundle {
                                    transform: Transform::from_translation(
                                        path_start.location,
                                    )
                                    .with_scale(Vec3::new(3.5, 3.5, 3.5)),
                                    ..Default::default()
                                },
                                Name::new("Enemy"),
                                Enemy { speed: 1.5 },
                                Health { value: 2.0 },
                                PathProgress::new(path),
                                PhysicsBundle::moving_entity().make_kinematic(),
                            ))
                            .with_children(|commands| {
                                commands.spawn((
                                    SceneBundle {
                                        scene: drone
                                            .default_scene
                                            .clone()
                                            .unwrap(),
                                        ..Default::default()
                                    },
                                    player,
                                ));
                            });
                    }
                }
                Err(_) => {}
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

fn enemy_reaches_portal_handler(
    mut commands: Commands,
    enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
    path_manager: Query<&PathManager>,
    mut ev_state_update: EventWriter<StateUpdateEvent>,
) {
    if let Ok(manager) = path_manager.get_single() {
        for (enemy_entity, enemy_pos) in &enemies {
            if let Some(end) = &manager.get_end() {
                if enemy_pos.translation().distance(end.location)
                    <= manager.despawn_distance
                {
                    debug!("Entity {:?} reached end of path", enemy_entity);
                    commands.entity(enemy_entity).despawn_recursive();
                    ev_state_update.send(StateUpdateEvent::EnemyReachedPortal);
                }
            }
        }
    }
}
