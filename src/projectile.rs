use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::tower_shoot;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

pub struct HitEvent {
    pub entity: Entity,
    pub force: f32,
}

#[derive(Reflect, Component, Default)]
pub struct Projectile {
    pub direction: Option<Vec3>,
    pub speed: f32,
    pub force: f32,
    pub target: Option<Entity>,
}

pub fn projectile_plugin(app: &mut App) {
    app.register_type::<Lifetime>()
        .register_type::<Projectile>()
        .add_event::<HitEvent>()
        .add_system(move_projectile.after(tower_shoot))
        .add_system(projectile_despawn)
        .add_system(projectile_collision_detection);
}

fn projectile_collision_detection(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Projectile)>,
    mut colliding_entities_query: Query<(Entity, &CollidingEntities)>,
    mut ev_hit_event: EventWriter<HitEvent>,
) {
    for (entity, colliding_entities) in colliding_entities_query.iter_mut() {
        for (projectile, projectile_info) in projectile_query.iter() {
            if colliding_entities.contains(projectile) {
                debug!("Hit!");
                commands.entity(projectile).despawn_recursive();
                ev_hit_event.send(HitEvent {
                    entity,
                    force: projectile_info.force,
                });
            }
        }
    }
}

fn move_projectile(
    mut commands: Commands,
    mut projectiles: Query<(
        Entity,
        &Projectile,
        &mut Transform,
        &GlobalTransform,
    )>,
    possible_targets: Query<(Entity, &GlobalTransform)>,
    time: Res<Time>,
) {
    for (projectile_ent, projectile, mut transform, location) in
        &mut projectiles
    {
        if let Some(direction) = projectile.direction {
            transform.translation +=
                direction.normalize() * projectile.speed * time.delta_seconds();
        } else if let Some(target) = projectile.target {
            if let Ok(target_pos) = possible_targets.get(target) {
                let direction =
                    target_pos.1.translation() - location.translation();

                transform.translation += direction.normalize()
                    * projectile.speed
                    * time.delta_seconds();
            } else {
                commands.entity(projectile_ent).despawn_recursive();
            }
        }
    }
}

fn projectile_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut bullet) in &mut bullets {
        bullet.timer.tick(time.delta());
        if bullet.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
