use bevy::prelude::*;

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

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Lifetime>()
            .register_type::<Projectile>()
            .add_system(move_projectile)
            .add_system(projectile_despawn);
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
