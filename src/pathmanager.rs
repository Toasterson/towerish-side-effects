use bevy::prelude::*;

use crate::{Enemy, Proxy};

pub fn path_manager_plugin(app: &mut App) {
    app.add_event::<PathManagerUpdate>()
        .add_system(handle_pathmanager_update)
        .add_system(handle_despawn);
}

#[derive(Component)]
pub struct PathManager {
    pub waypoints: Vec<Proxy>,
    pub end: Option<Proxy>,
    pub despawn_distance: f32,
}

const DEFAULT_DESPAWN_DISTANCE: f32 = 0.2;

impl PathManager {
    pub fn new() -> Self {
        Self {
            waypoints: vec![],
            end: None,
            despawn_distance: DEFAULT_DESPAWN_DISTANCE,
        }
    }

    pub fn push(&mut self, proxy: Proxy) {
        let mut waypoints = self.waypoints.clone();
        waypoints.push(proxy);
        waypoints.sort_by(|a, b| b.node_id.cmp(&a.node_id));
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
) {
    if let Ok(manager) = path_manager.get_single() {
        for (enemy_entity, enemy_pos) in &enemies {
            if let Some(end) = &manager.end {
                if enemy_pos.translation().distance(end.location)
                    <= manager.despawn_distance
                {
                    debug!("Entity {:?} reached end of path", enemy_entity);
                    commands.entity(enemy_entity).despawn_recursive();
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
                    if let Some(current_end) = &path_manager.end {
                        if current_end.node_id > p.node_id {
                            info!("Updating end of path to {}", p);
                            path_manager.end = Some(p.clone());
                        }
                    } else {
                        info!("Adding new end of path {}", p);
                        path_manager.end = Some(p.clone());
                    }
                }
                PathManagerUpdate::RemoveNode(p) => {
                    info!("Removing Proxy Waypoint {} from path", p);
                    path_manager.waypoints = path_manager
                        .waypoints
                        .clone()
                        .into_iter()
                        .filter(|ve| ve.node_id != p.node_id)
                        .collect();

                    if let Some(current_end) = &path_manager.end {
                        if current_end.node_id == p.node_id {
                            path_manager.end = path_manager
                                .waypoints
                                .last()
                                .map(|p| p.clone());
                        }
                    }
                }
            },
            Err(_) => info!("No Pathmanager yet while adding {:#?}", event),
        }
    }
}
