use bevy::{prelude::*, utils::FloatOrd};
use strum::{Display as EnumDisplay, EnumIter};

use crate::{
    graphics::CreateParticleSystem, Enemy, GameAssets, Lifetime, PhysicsBundle,
    Projectile,
};

#[derive(Component)]
pub struct TowerUIRoot;

#[derive(Component, Default)]
pub struct Tower {
    pub shooting_timer: Timer,
    pub bullet_offset: Vec3,
    pub effects: Vec<TowerEffects>,
}

#[derive(Debug, Reflect, Component, EnumIter, EnumDisplay, Copy, Clone)]
pub enum TowerType {
    Test,
}

pub enum TowerBuildEvent {
    Dispatch {
        entity: Entity,
        kind: TowerType,
        pos: Vec3,
    },
}

#[derive(Debug, Reflect, Component, EnumIter, Copy, Clone)]
pub enum TowerEffects {
    SpeedShot(f32),
    StrongShot(f32),
    WeakShot(f32),
    AOEBuff(f32),
}

pub fn tower_plugin(app: &mut App) {
    app.add_event::<TowerBuildEvent>()
        .add_system(tower_button_clicked)
        .add_system(tower_shoot);
}

pub fn tower_shoot(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut towers: Query<(Entity, &mut Tower, &GlobalTransform)>,
    targets: Query<&GlobalTransform, With<Enemy>>,
    time: Res<Time>,
) {
    for (tower_ent, mut tower, transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let bullet_spawn = transform.translation() + tower.bullet_offset;

            let target_offset = transform.translation();

            let direction = targets
                .iter()
                .min_by_key(|target_transform| {
                    FloatOrd(Vec3::distance(
                        target_transform.translation(),
                        bullet_spawn,
                    ))
                })
                .map(|closest_target| {
                    closest_target.translation() - target_offset
                });

            if let Some(direction) = direction {
                debug!("Shooting at target at: {}", direction);
                commands.entity(tower_ent).with_children(|commands| {
                    commands.spawn((
                        PbrBundle {
                            mesh: assets.shpere_shape.clone(),
                            material: assets.ball_projectile_color.clone(),
                            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                                .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                            ..default()
                        },
                        Lifetime {
                            timer: Timer::from_seconds(1.5, TimerMode::Once),
                        },
                        Projectile {
                            direction,
                            speed: 60.0,
                        },
                        Name::new("Bullet"),
                        PhysicsBundle::moving_entity().make_kinematic(),
                    ));
                });
            }
        }
    }
}

pub fn spawn_tower(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    tt: TowerType,
) -> Entity {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_translation(
            position,
        )))
        .insert(Name::new("Default Tower"))
        .insert(Tower {
            shooting_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            bullet_offset: Vec3::new(0.0, 1.2, 0.0),
            effects: vec![],
        })
        .insert(tt)
        .with_children(|commands| {
            commands.spawn(SceneBundle {
                scene: assets.tower_slice_a.clone(),
                transform: Transform::from_xyz(0.0, -0.4, 0.0),
                ..Default::default()
            });
        })
        .id()
}

fn tower_button_clicked(
    mut ev_tower_build_events: EventReader<TowerBuildEvent>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut particle_events: EventWriter<CreateParticleSystem>,
) {
    for event in ev_tower_build_events.iter() {
        let TowerBuildEvent::Dispatch { entity, kind, pos } = event;
        commands.entity(*entity).despawn_recursive();
        spawn_tower(&mut commands, &assets, *pos, *kind);
        particle_events.send(CreateParticleSystem {
            system: crate::graphics::ParticleSystemType::Landing,
            transform: Transform::from_translation(*pos),
        });
    }
}
