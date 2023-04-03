use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_mod_picking::*;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use crate::GameAssets;

pub enum TowerBase {
    Bright(Vec3),
    Purple(Vec3),
    Bad(Vec3),
}

const MAP_TOWER_BASES: [TowerBase; 4] = [
    TowerBase::Bad(Vec3::new(10.8182, -14.5673, -6.77553)),
    TowerBase::Bright(Vec3::new(2.6328, -6.97, -0.693552)),
    TowerBase::Purple(Vec3::new(-2.14896, -1.25817, -4.63252)),
    TowerBase::Purple(Vec3::new(-4.76274, -11.2879, -7.37368)),
];

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_startup_system(spawn_basic_scene);
    }
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    assets: Res<GameAssets>,
) {
    // set gravity
    rapier_config.gravity = Vec3::ZERO;

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1600.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(-5.0, 60.0, -5.0),
            ..default()
        })
        .insert(Name::new("Sun"));

    let default_collider_color =
        materials.add(Color::rgba(0.3, 0.5, 0.3, 0.3).into());
    let selected_collider_color =
        materials.add(Color::rgba(0.3, 0.9, 0.3, 0.9).into());

    commands
        .spawn(SceneBundle {
            scene: assets.scene(crate::Scenes::Map),
            ..Default::default()
        })
        .insert(Name::new("Map"))
        .with_children(|commands| {
            for (idx, tb) in MAP_TOWER_BASES.iter().enumerate() {
                let (handle, coords) = match tb {
                    TowerBase::Bright(v) => {
                        (assets.scene(crate::Scenes::TowerBaseBright), v)
                    }
                    TowerBase::Purple(v) => {
                        (assets.scene(crate::Scenes::TowerBasePurple), v)
                    }
                    TowerBase::Bad(v) => {
                        (assets.scene(crate::Scenes::TowerBaseBad), v)
                    }
                };

                commands
                    .spawn(SpatialBundle::from_transform(Transform::from_xyz(
                        coords.x,
                        coords.y + 0.8,
                        coords.z,
                    )))
                    .insert(Name::new(format!("Tower_Base_{}", idx)))
                    .insert(meshes.add(shape::Capsule::default().into()))
                    .insert(Highlighting {
                        initial: default_collider_color.clone(),
                        hovered: Some(selected_collider_color.clone()),
                        pressed: Some(selected_collider_color.clone()),
                        selected: Some(selected_collider_color.clone()),
                    })
                    .insert(default_collider_color.clone())
                    .insert(NotShadowCaster)
                    .insert(PickableBundle::default())
                    .with_children(|commands| {
                        commands.spawn(SceneBundle {
                            scene: handle,
                            transform: Transform::from_xyz(
                                coords.x, coords.y, coords.z,
                            ),
                            ..Default::default()
                        });
                    });
            }
        });
}
