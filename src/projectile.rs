use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::SideEffects;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Reflect, Component, Default)]
pub struct Projectile {
    pub direction: Vec3,
    pub speed: f32,
}

#[derive(Reflect, Component)]
pub struct Health {
    pub value: f32,
}

pub fn projectile_plugin(app: &mut App) {
    app.register_type::<Lifetime>()
        .register_type::<Projectile>()
        .add_system(move_projectile)
        .add_system(projectile_despawn)
        .add_system(projectile_collision_detection);
}

fn projectile_collision_detection(
    mut commands: Commands,
    projectile_query: Query<(Entity, &SideEffects), With<Projectile>>,
    mut colliding_entities_query: Query<(&mut Health, &CollidingEntities)>,
) {
    for (mut health, colliding_entities) in colliding_entities_query.iter_mut()
    {
        for (projectile, side_effects) in projectile_query.iter() {
            if colliding_entities.contains(projectile) {
                debug!("Hit!");
                debug!("Side effects: {:#?}", side_effects);
                commands.entity(projectile).despawn_recursive();
                health.value -= 1.0;
            }
        }
    }
}

fn move_projectile(
    mut projectiles: Query<(&Projectile, &mut Transform)>,
    time: Res<Time>,
) {
    for (projectile, mut transform) in &mut projectiles {
        transform.translation += projectile.direction.normalize()
            * projectile.speed
            * time.delta_seconds();
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
