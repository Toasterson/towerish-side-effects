use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_vfx_bag::{post_processing::lut::Lut, BevyVfxBagPlugin};

pub fn graphics_plugin(app: &mut App) {
    app.add_plugin(BevyVfxBagPlugin::default());
    app.add_plugin(HanabiPlugin);
    app.add_event::<CreateParticleSystem>();
    app.add_system(test_luts);
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

#[derive(Reflect, PartialEq)]
pub enum ParticleSystemType {
    Landing,
    MuzzleFlash,
    Impact,
}

#[derive(Resource, Reflect)]
pub struct CreateParticleSystem {
    pub system: ParticleSystemType,
    pub location: Vec3,
    pub orientation: Vec3,
}

fn particle_system_events(
    mut events: EventReader<CreateParticleSystem>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    for event in events.iter() {
        let texture_handle: Handle<Image> = asset_server.load("cloud.png");

        let mut color_gradient = Gradient::new();
        color_gradient.add_key(0.0, Vec4::splat(1.0));
        color_gradient.add_key(0.2, Vec4::splat(1.0));
        color_gradient.add_key(0.8, Vec4::new(1.0, 1.0, 1.0, 0.8));
        color_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

        let mut size_gradient = Gradient::new();
        size_gradient.add_key(0.0, Vec2::new(0.2, 0.2));
        size_gradient.add_key(1.0, Vec2::new(1., 1.));

        let effect = effects.add(
            EffectAsset {
                name: "Gradient".to_string(),
                // TODO: Figure out why no particle spawns if this is 1
                capacity: 400,
                spawner: Spawner::once(128.0.into(), true),
                ..Default::default()
            }
            .init(InitPositionCircleModifier {
                center: event.location,
                axis: event.orientation,
                radius: 0.4,
                dimension: ShapeDimension::Surface,
            })
            .init(InitVelocityCircleModifier {
                center: event.location,
                axis: event.orientation,
                speed: Value::Uniform((3.0, 4.0)),
            })
            .init(InitLifetimeModifier {
                lifetime: bevy_hanabi::Value::Uniform((0.1, 1.)),
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

        commands
            .spawn(ParticleEffectBundle::new(effect))
            .insert(Name::new("Part Effect"));
    }
}
