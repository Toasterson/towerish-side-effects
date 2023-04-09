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
    Gun,
    Rocket,
    Sniper,
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
    mut towers: Query<(Entity, &mut Tower, &TowerType, &GlobalTransform)>,
    targets: Query<(Entity, &GlobalTransform), With<Enemy>>,
    time: Res<Time>,
) {
    for (tower_ent, mut tower, tower_type, transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if tower.shooting_timer.just_finished() {
            let bullet_spawn = transform.translation() + tower.bullet_offset;

            let target_offset = transform.translation();

            let target = targets.iter().min_by_key(|target_transform| {
                FloatOrd(Vec3::distance(
                    target_transform.1.translation(),
                    bullet_spawn,
                ))
            });

            if let Some(target) = target {
                debug!("Shooting at target at: {}", target.1.translation());

                let (direction, speed, force, lifetime) = match tower_type {
                    TowerType::Gun => (
                        Some(target.1.translation() - target_offset),
                        60.0,
                        1.0,
                        Timer::from_seconds(1.5, TimerMode::Once),
                    ),
                    TowerType::Rocket => (
                        None,
                        10.0,
                        10.0,
                        Timer::from_seconds(10.0, TimerMode::Once),
                    ),
                    TowerType::Sniper => (
                        None,
                        100.0,
                        2.0,
                        Timer::from_seconds(9.0, TimerMode::Once),
                    ),
                };

                commands.entity(tower_ent).with_children(|commands| {
                    commands.spawn((
                        SceneBundle {
                            scene: assets.bullet_scene.clone(),
                            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                                .with_scale(Vec3::new(4.0, 4.0, 4.0)),
                            ..default()
                        },
                        Lifetime { timer: lifetime },
                        Projectile {
                            direction,
                            speed,
                            force,
                            target: match tower_type {
                                TowerType::Gun => None,
                                TowerType::Rocket => Some(target.0),
                                TowerType::Sniper => Some(target.0),
                            },
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
    let shooting_timer = match tt {
        TowerType::Gun => Timer::from_seconds(0.2, TimerMode::Repeating),
        TowerType::Rocket => Timer::from_seconds(1.5, TimerMode::Repeating),
        TowerType::Sniper => Timer::from_seconds(0.8, TimerMode::Repeating),
    };
    commands
        .spawn((
            SpatialBundle::from_transform(Transform::from_translation(
                position,
            )),
            Name::new("Default Tower"),
            Tower {
                shooting_timer,
                bullet_offset: Vec3::new(0.0, 1.2, 0.0),
                effects: vec![],
            },
            tt,
        ))
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
