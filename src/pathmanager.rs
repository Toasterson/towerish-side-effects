use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct PathManager {
    pub waypoints: Vec<Vec3>,
}

impl PathManager {
    pub fn new(waypoints: Vec<Vec3>) -> Self {
        Self { waypoints }
    }

    pub fn get_position(&self, progress: f32) -> Vec3 {
        let mut tail: Vec3 = self.waypoints.first().unwrap().clone();
        let mut progress = progress;
        for &waypoint in &self.waypoints {
            if tail.distance(waypoint) > progress {
                return tail.lerp(waypoint, progress / tail.distance(waypoint));
            } else {
                progress = progress - tail.distance(waypoint);
                tail = waypoint.clone();
            }
        }
        *self.waypoints.last().unwrap()
    }
}
