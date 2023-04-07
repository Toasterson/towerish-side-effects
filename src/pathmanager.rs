use bevy::prelude::*;

use crate::{Enemy, Proxy, StateUpdateEvent};

pub fn path_manager_plugin(app: &mut App) {
    app.add_event::<PathManagerUpdate>()
        .add_system(handle_pathmanager_update)
        .add_system(handle_despawn);
}

#[derive(Component)]
pub struct PathManager {
    pub waypoints: Vec<Proxy>,
    pub despawn_distance: f32,
}

const DEFAULT_DESPAWN_DISTANCE: f32 = 0.2;

impl PathManager {
    pub fn new() -> Self {
        Self {
            waypoints: vec![],
            despawn_distance: DEFAULT_DESPAWN_DISTANCE,
        }
    }

    pub fn push(&mut self, proxy: Proxy) {
        let mut waypoints = self.waypoints.clone();
        waypoints.push(proxy);
        waypoints.sort_by(|a, b| b.node_id.cmp(&a.node_id));
        waypoints.reverse();
        self.waypoints = waypoints;
    }

    pub fn get_position(&self, progress: f32) -> Vec3 {
        let mut tail: Vec3 = self.waypoints.first().unwrap().clone().location;
        let mut progress = progress;
        for waypoint in &self.waypoints {
            if tail.distance(waypoint.location) > progress {
                return tail.lerp(
                    waypoint.location,
                    progress / tail.distance(waypoint.location),
                );
            } else {
                progress = progress - tail.distance(waypoint.location);
                tail = waypoint.clone().location;
            }
        }
        self.waypoints.last().unwrap().location
    }

    pub fn get_start(&self) -> Option<Proxy> {
        self.waypoints.first().map(|pp| pp.clone())
    }

    pub fn get_end(&self) -> Option<Proxy> {
        self.waypoints.last().map(|pp| pp.clone())
    }
}

#[derive(Debug)]
pub enum PathManagerUpdate {
    AddNode(Proxy),
    #[allow(dead_code)]
    RemoveNode(Proxy),
}

fn handle_despawn(
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

fn handle_pathmanager_update(
    mut ev_pathmanager_update: EventReader<PathManagerUpdate>,
    mut path_manager: Query<&mut PathManager>,
) {
    for event in ev_pathmanager_update.iter() {
        match path_manager.get_single_mut() {
            Ok(mut path_manager) => match event {
                PathManagerUpdate::AddNode(p) => {
                    info!("Adding Proxy Waypoint {} to path", p);
                    path_manager.push(p.clone());
                }
                PathManagerUpdate::RemoveNode(p) => {
                    info!("Removing Proxy Waypoint {} from path", p);
                    path_manager.waypoints = path_manager
                        .waypoints
                        .clone()
                        .into_iter()
                        .filter(|ve| ve.node_id != p.node_id)
                        .collect();
                }
            },
            Err(_) => info!("No Pathmanager yet while adding {:#?}", event),
        }
    }
}
