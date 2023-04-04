use bevy::{prelude::*, utils::FloatOrd};
use bevy_rapier3d::prelude::CollidingEntities;

use crate::{GameAssets, PhysicsBundle, Proxy, Route};

#[derive(Reflect, Component)]
pub struct Waypoint {
    pub coords: Transform,
}

#[derive(Reflect, Component)]
pub struct Enemy {
    pub speed: f32,
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
    app.add_event::<FindNextWaypointEvent>()
        .register_type::<Waypoint>()
        .register_type::<Enemy>()
        .register_type::<Portal>()
        .add_system(route_collision_detection)
        .add_system(find_next_waypoint_node.after(route_collision_detection))
        .add_system(enemy_spawner)
        .add_system(move_enemies.after(enemy_spawner));
}

fn enemy_spawner(
    mut commands: Commands,
    mut portal_query: Query<(&mut Portal, &Proxy, &GlobalTransform)>,
    mut ev_find_next: EventWriter<FindNextWaypointEvent>,
    time: Res<Time>,
    assets: Res<GameAssets>,
) {
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
                    Enemy { speed: 5.0 },
                    PhysicsBundle::moving_entity(Vec3::new(1.0, 1.0, 1.0))
                        .make_kinematic(),
                ))
                .id();
            ev_find_next.send(FindNextWaypointEvent(
                enemy,
                proxy.clone(),
                portal_pos.clone(),
            ));
        }
    }
}

fn move_enemies(
    mut enemies: Query<(&Enemy, &Waypoint, &mut Transform, &GlobalTransform)>,
    time: Res<Time>,
) {
    for (enemy, waypoint, mut transform, enemy_pos) in &mut enemies {
        let direction = waypoint.coords.translation - enemy_pos.translation();
        transform.translation +=
            direction.normalize() * enemy.speed * time.delta_seconds();
    }
}

pub struct FindNextWaypointEvent(Entity, Proxy, GlobalTransform);

fn route_collision_detection(
    mut ev_find_next: EventWriter<FindNextWaypointEvent>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut colliding_entities_query: Query<
        (&CollidingEntities, &Proxy, &GlobalTransform),
        With<Route>,
    >,
) {
    for (colliding_entities, proxy, gt) in colliding_entities_query.iter_mut() {
        for enemy_entity in enemy_query.iter() {
            if colliding_entities.contains(enemy_entity) {
                ev_find_next.send(FindNextWaypointEvent(
                    enemy_entity,
                    proxy.clone(),
                    gt.clone(),
                ));
            }
        }
    }
}

fn find_next_waypoint_node(
    mut commands: Commands,
    mut ev_next_waypoint: EventReader<FindNextWaypointEvent>,
    route_query: Query<(&Proxy, &GlobalTransform)>,
) {
    for ev in ev_next_waypoint.iter() {
        let closest = route_query
            .iter()
            .filter(|(proxy, _)| proxy.node_id > ev.1.node_id)
            .min_by_key(|(_, target_transform)| {
                FloatOrd(Vec3::distance(
                    target_transform.translation(),
                    ev.2.translation(),
                ))
            });
        if let Some(wp) = closest {
            commands.entity(ev.0).remove::<Waypoint>().insert(Waypoint {
                coords: wp.1.compute_transform(),
            });
        }
    }
}
