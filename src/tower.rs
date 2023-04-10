use std::time::Duration;

use bevy::{prelude::*, utils::FloatOrd};
use bevy_mod_picking::*;
use strum::{Display as EnumDisplay, EnumIter};

use crate::{
    graphics::CreateParticleSystem, Enemy, GameAssets, Lifetime, PhysicsBundle,
    Projectile,
};

#[derive(Component)]
pub struct TowerUIRoot;

#[derive(Component, Default, Clone)]
pub struct Tower {
    pub shooting_timer: Timer,
    pub bullet_offset: Vec3,
    pub upgrades: Vec<TowerUpgrades>,
    pub side_effects: Vec<TowerSideEffects>,
}

#[derive(Component, Clone)]
pub struct SideEffectBundle {
    pub side_effects: Vec<TowerSideEffects>,
}

impl SideEffectBundle {
    pub fn from_tower(tower: &Tower) -> Self {
        Self {
            side_effects: tower.side_effects.clone(),
        }
    }
}

#[derive(Debug, Reflect, Component, EnumIter, EnumDisplay, Copy, Clone)]
pub enum TowerType {
    Gun,
    Rocket,
    Sniper,
}

impl TowerType {
    pub fn get_price(&self, wave_multiplier: i32) -> f32 {
        let wave_multiplier = if wave_multiplier <= 0 {
            1
        } else {
            wave_multiplier
        };
        match self {
            TowerType::Gun => 500.0 * wave_multiplier as f32,
            TowerType::Rocket => 650.0 * wave_multiplier as f32,
            TowerType::Sniper => 600.0 * wave_multiplier as f32,
        }
    }
}

pub enum TowerBuildEvent {
    Dispatch {
        entity: Entity,
        kind: TowerType,
        pos: Vec3,
    },
    Upgrade {
        entity: Entity,
        effect: TowerUpgrades,
        side_effect: Option<TowerSideEffects>,
    },
}

#[derive(Debug, Reflect, Component, EnumIter, Copy, Clone, EnumDisplay)]
pub enum TowerUpgrades {
    BulletSpeedBuff(f32),
    ForceBuff(f32),
    ShootingSpeedBuff(f32),
    AOE(f32),
}

impl TowerUpgrades {
    pub fn get_price(&self, wave_multiplier: i32, force: i32) -> f32 {
        let wave_multiplier = if wave_multiplier <= 0 {
            1
        } else {
            wave_multiplier
        };
        match self {
            TowerUpgrades::BulletSpeedBuff(_) => {
                100. + (1.05 * (wave_multiplier as f32 * force as f32))
            }
            TowerUpgrades::ForceBuff(_) => {
                100. + (1.05 * (wave_multiplier as f32 * force as f32))
            }
            TowerUpgrades::AOE(_) => {
                200. + (2.05 * (wave_multiplier as f32 * force as f32))
            }
            TowerUpgrades::ShootingSpeedBuff(_) => {
                500. + (5.05 * (wave_multiplier as f32 * force as f32))
            }
        }
    }
    pub fn set_force(self, force: f32) -> Self {
        match self {
            TowerUpgrades::BulletSpeedBuff(_) => Self::BulletSpeedBuff(force),
            TowerUpgrades::ForceBuff(_) => Self::ForceBuff(force),
            TowerUpgrades::ShootingSpeedBuff(_) => {
                Self::ShootingSpeedBuff(force)
            }
            TowerUpgrades::AOE(_) => Self::AOE(force),
        }
    }
}

#[derive(Debug, Reflect, Component, EnumIter, Copy, Clone)]
pub enum TowerSideEffects {
    WeakShot(f32),
    HealShot(f32),
}

impl TowerSideEffects {
    pub fn get_weights(wave_multiplier: i32, force: i32) -> Vec<f32> {
        let wave_multiplier = if wave_multiplier <= 0 {
            1
        } else {
            wave_multiplier
        };

        vec![
            100.0,
            0.5 * wave_multiplier as f32 * force as f32,
            0.2 * wave_multiplier as f32 * force as f32,
        ]
    }
}

pub fn tower_plugin(app: &mut App) {
    app.add_event::<TowerBuildEvent>()
        .add_system(tower_build)
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

                let mut speed_mod = 0.;
                let mut force_mod = 0.;
                let mut aoe_mod = 0.;

                for upg in &tower.upgrades {
                    match upg {
                        TowerUpgrades::BulletSpeedBuff(v) => speed_mod += v,
                        TowerUpgrades::ForceBuff(v) => force_mod += v,
                        TowerUpgrades::AOE(v) => aoe_mod += v,
                        TowerUpgrades::ShootingSpeedBuff(_) => {}
                    }
                }

                let (direction, speed, force, lifetime, handle) =
                    match tower_type {
                        TowerType::Gun => (
                            Some(target.1.translation() - target_offset),
                            60.0 + speed_mod,
                            1.0 + force_mod,
                            Timer::from_seconds(1.5, TimerMode::Once),
                            assets.bullet_scene.clone(),
                        ),
                        TowerType::Rocket => (
                            None,
                            10.0 + speed_mod,
                            10.0 + force_mod,
                            Timer::from_seconds(10.0, TimerMode::Once),
                            assets.rocket_scene.clone(),
                        ),
                        TowerType::Sniper => (
                            None,
                            100.0 + speed_mod,
                            2.0 + force_mod,
                            Timer::from_seconds(9.0, TimerMode::Once),
                            assets.sniper_bullet_scene.clone(),
                        ),
                    };

                commands.entity(tower_ent).with_children(|commands| {
                    commands.spawn((
                        SceneBundle {
                            scene: handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                                .with_scale(Vec3::new(4.0, 4.0, 4.0))
                                .looking_at(-target.1.translation(), Vec3::Y),
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
                        SideEffectBundle::from_tower(&tower),
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
            PbrBundle {
                mesh: assets.get_capsule_shape().clone(),
                material: assets.default_collider_color.clone(),
                transform: Transform::from_translation(position),
                ..Default::default()
            },
            Name::new("Default Tower"),
            Tower {
                shooting_timer,
                bullet_offset: Vec3::new(0.0, 1.2, 0.0),
                upgrades: vec![],
                side_effects: vec![],
            },
            tt,
            PickableBundle::default(),
            Highlighting {
                initial: assets.default_collider_color.clone(),
                hovered: Some(assets.tower_base_selected_color.clone()),
                pressed: Some(assets.tower_base_selected_color.clone()),
                selected: Some(assets.tower_base_selected_color.clone()),
            },
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

fn tower_build(
    mut ev_tower_build_events: EventReader<TowerBuildEvent>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut particle_events: EventWriter<CreateParticleSystem>,
    mut towers: Query<&mut Tower>,
) {
    for event in ev_tower_build_events.iter() {
        match event {
            TowerBuildEvent::Dispatch { entity, kind, pos } => {
                commands.entity(*entity).despawn_recursive();
                spawn_tower(&mut commands, &assets, *pos, *kind);
                particle_events.send(CreateParticleSystem {
                    system: crate::graphics::ParticleSystemType::Landing,
                    transform: Transform::from_translation(*pos),
                });
            }
            TowerBuildEvent::Upgrade {
                entity,
                effect,
                side_effect,
            } => {
                if let Ok(mut tower) = towers.get_mut(*entity) {
                    match effect {
                        TowerUpgrades::ShootingSpeedBuff(v) => {
                            let speed_up = v / 10.0;
                            let duration = tower
                                .shooting_timer
                                .duration()
                                .clone()
                                .saturating_sub(Duration::from_secs_f32(
                                    speed_up,
                                ));
                            tower.shooting_timer.set_duration(duration);
                        }
                        effect => tower.upgrades.push(*effect),
                    }
                    if let Some(side_effect) = side_effect {
                        tower.side_effects.push(*side_effect);
                    }
                }
            }
        }
    }
}
