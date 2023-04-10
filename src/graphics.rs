use bevy::prelude::*;
use bevy_vfx_bag::{
    post_processing::{lut::Lut, wave::Wave},
    BevyVfxBagPlugin,
};

#[cfg(feature = "particles")]
use bevy_hanabi::prelude::*;

use crate::StateUpdateEvent;

pub fn graphics_plugin(app: &mut App) {
    app.add_event::<CreateParticleSystem>();
    #[cfg(feature = "particles")]
    {
        app.add_plugin(BevyVfxBagPlugin::default());
        app.add_plugin(HanabiPlugin);
        app.add_startup_system(setup_particle_systems);
        app.insert_resource(ParticleSystems {
            landing: Entity::PLACEHOLDER,
            muzzle_flash: Entity::PLACEHOLDER,
            impact: Entity::PLACEHOLDER,
            portal: Entity::PLACEHOLDER,
        });
    }

    app.add_system(test_luts)
        .add_system(health_loss_effects)
        .add_system(camera_effect_decay);

    #[cfg(feature = "particles")]
    app.add_system(particle_system_events);
}

// Cycle through some preset LUTs.
fn test_luts(
    mut choice: Local<usize>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<Entity, With<Camera>>,
) {
    let choice_now = if keyboard_input.just_pressed(KeyCode::Left) {
        choice.saturating_sub(1)
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        (*choice + 1).min(3)
    } else {
        *choice
    };
    if *choice != choice_now {
        let entity = query.single_mut();
        *choice = choice_now;
        match *choice {
            0 => {
                commands.get_or_spawn(entity).insert(Lut::neo());
                info!("Neo");
            }
            1 => {
                commands.get_or_spawn(entity).insert(Lut::arctic());
                info!("Arctic");
            }
            2 => {
                commands.get_or_spawn(entity).insert(Lut::slate());
                info!("Slate");
            }
            3 => {
                commands.get_or_spawn(entity).remove::<Lut>();
                info!("Disabled (default Bevy colors)");
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "particles")]
#[derive(Reflect, PartialEq)]
pub enum ParticleSystemType {
    Landing,
    MuzzleFlash,
    Impact,
}

#[cfg(feature = "particles")]
#[derive(Resource, Reflect)]
pub struct CreateParticleSystem {
    pub system: ParticleSystemType,
    pub transform: Transform,
}

#[cfg(feature = "particles")]
#[derive(Resource, Reflect)]
pub struct ParticleSystems {
    landing: Entity,
    muzzle_flash: Entity,
    impact: Entity,
    portal: Entity,
}

#[cfg(feature = "particles")]
fn setup_particle_systems(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut particle_systems: ResMut<ParticleSystems>,
) {
    let texture_handle: Handle<Image> = asset_server.load("cloud.png");

    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::splat(1.0));
    color_gradient.add_key(0.2, Vec4::new(1.0, 1.0, 1.0, 0.2));
    color_gradient.add_key(0.3, Vec4::new(1.0, 1.0, 1.0, 0.1));
    color_gradient.add_key(0.4, Vec4::new(1.0, 1.0, 1.0, 0.05));
    color_gradient.add_key(0.98, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec2::new(0.2, 0.2));
    size_gradient.add_key(1.0, Vec2::new(2.0, 2.0));

    let spawner = Spawner::once(300.0.into(), false);

    let effect = effects.add(
        EffectAsset {
            name: "Gradient".to_string(),
            capacity: 4000,
            spawner,
            ..Default::default()
        }
        .init(InitPositionCircleModifier {
            center: Vec3::ZERO,
            axis: Vec3::Y,
            radius: 0.4,
            dimension: ShapeDimension::Surface,
        })
        .init(InitVelocityCircleModifier {
            center: Vec3::ZERO,
            axis: Vec3::Y,
            speed: Value::Uniform((2.0, 3.5)),
        })
        .init(InitLifetimeModifier {
            lifetime: bevy_hanabi::Value::Uniform((0.4, 2.5)),
        })
        .update(LinearDragModifier { drag: 3. })
        .render(BillboardModifier {})
        .render(ParticleTextureModifier {
            texture: texture_handle.clone(),
        })
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
        }),
    );

    particle_systems.landing = commands
        .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("Part Effect"))
        .id();

    // Portal
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.3, Vec2::new(0.2, 0.02));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let effect1 = effects.add(
        EffectAsset {
            name: "portal".to_string(),
            capacity: 32768,
            spawner: Spawner::rate(5000.0.into()),
            ..Default::default()
        }
        .init(InitPositionCircleModifier {
            center: Vec3::ZERO,
            axis: Vec3::Z,
            radius: 4.,
            dimension: ShapeDimension::Surface,
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.6, 1.3)),
        })
        .update(LinearDragModifier { drag: 2. })
        .update(RadialAccelModifier::constant(Vec3::ZERO, -6.0))
        .update(TangentAccelModifier::constant(Vec3::ZERO, Vec3::Z, 30.))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
        })
        .render(OrientAlongVelocityModifier),
    );

    particle_systems.portal = commands
        .spawn((
            Name::new("portal"),
            ParticleEffectBundle {
                effect: ParticleEffect::new(effect1),
                transform: Transform::IDENTITY,
                ..Default::default()
            },
        ))
        .id();
}

#[cfg(feature = "particles")]
fn particle_system_events(
    mut events: EventReader<CreateParticleSystem>,
    systems: Res<ParticleSystems>,
    mut effect: Query<(
        &mut CompiledParticleEffect,
        &mut EffectSpawner,
        &mut Transform,
    )>,
) {
    for CreateParticleSystem {
        system,
        transform: new_transform,
    } in events.iter()
    {
        let Ok((_effect, mut spawner, mut transform)) = effect.get_mut(match system {
            ParticleSystemType::Landing => systems.landing,
            ParticleSystemType::MuzzleFlash => systems.muzzle_flash,
            ParticleSystemType::Impact => systems.impact,
        }) else {println!("ERROR 401"); return;};
        *transform = new_transform.clone();
        spawner.reset();
    }
}

fn health_loss_effects(
    mut events: EventReader<StateUpdateEvent>,
    mut commands: Commands,
    query: Query<Entity, With<Camera>>,
) {
    if events
        .iter()
        .filter(|w| match w {
            StateUpdateEvent::EnemyReachedPortal => true,
            _ => false,
        })
        .count()
        > 0
    {
        commands.get_or_spawn(query.single()).insert(Wave {
            waves_y: 0.,
            waves_x: 25.0,
            speed_y: 0.,
            speed_x: 10.,
            amplitude_y: 0.,
            amplitude_x: 0.10,
        });
    }
}

fn camera_effect_decay(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Wave), With<Camera>>,
) {
    for (entity, mut wave) in query.iter_mut() {
        wave.amplitude_x *= 0.9;
        wave.speed_x *= 0.95;
        if wave.amplitude_x < 0.001 {
            commands.entity(entity).remove::<Wave>();
        }
    }
}

pub fn portal_effect(
    mut effects: ResMut<Assets<EffectAsset>>,
) -> Handle<EffectAsset> {
    todo!()
}
