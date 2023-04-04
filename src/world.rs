use bevy::{
    gltf::{Gltf, GltfMesh},
    pbr::NotShadowCaster,
    prelude::*,
};
use bevy_mod_picking::*;
use bevy_rapier3d::prelude::*;

use crate::{GameAssets, PhysicsBundle};

pub enum TowerBase {
    Bright(Vec3),
    Purple(Vec3),
    Bad(Vec3),
}

const MAP_TOWER_BASES: [TowerBase; 4] = [
    TowerBase::Bad(Vec3::new(0.3, -6.1, 12.0)),
    TowerBase::Bright(Vec3::new(-3.4, -6.1, 14.0)),
    TowerBase::Purple(Vec3::new(-5.0, -6.5, 10.6)),
    TowerBase::Bright(Vec3::new(8.5, -5.8, 14.5)),
];

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_basic_scene)
            .add_system(handle_map_spawn);
    }
}

fn handle_map_spawn(
    mut ev_asset: EventReader<AssetEvent<Gltf>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    assets: Res<GameAssets>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                if handle == assets.map() {
                    let default_collider_color =
                        materials.add(Color::NONE.into());
                    let selected_collider_color =
                        materials.add(Color::rgba(0.3, 0.9, 0.3, 0.9).into());
                    let map = assets_gltf.get(handle).unwrap();
                    let map_gltf_mesh =
                        assets_gltfmesh.get(&map.meshes[1]).unwrap();
                    let map_mesh =
                        meshes.get(&map_gltf_mesh.primitives[0].mesh).unwrap();

                    commands
                        .spawn(SceneBundle {
                            scene: map.scenes[0].clone(),
                            ..Default::default()
                        })
                        .insert(
                            PhysicsBundle::from_mesh(map_mesh).make_kinematic(),
                        )
                        .insert(Name::new("Map"))
                        .with_children(|commands| {
                            for (idx, tb) in MAP_TOWER_BASES.iter().enumerate()
                            {
                                let (handle, coords) = match tb {
                                    TowerBase::Bright(v) => (
                                        assets.scene(
                                            crate::Scenes::TowerBaseBright,
                                        ),
                                        v,
                                    ),
                                    TowerBase::Purple(v) => (
                                        assets.scene(
                                            crate::Scenes::TowerBasePurple,
                                        ),
                                        v,
                                    ),
                                    TowerBase::Bad(v) => (
                                        assets
                                            .scene(crate::Scenes::TowerBaseBad),
                                        v,
                                    ),
                                };

                                commands
                                    .spawn(SpatialBundle::from_transform(
                                        Transform::from_translation(*coords),
                                    ))
                                    .insert(Name::new(format!(
                                        "Tower_Base_{}",
                                        idx
                                    )))
                                    .insert(
                                        Collider::from_bevy_mesh(
                                            meshes
                                                .get(assets.get_capsule_shape())
                                                .unwrap(),
                                            &ComputedColliderShape::TriMesh,
                                        )
                                        .unwrap(),
                                    )
                                    .insert(assets.get_capsule_shape().clone())
                                    .insert(Highlighting {
                                        initial: default_collider_color.clone(),
                                        hovered: Some(
                                            selected_collider_color.clone(),
                                        ),
                                        pressed: Some(
                                            selected_collider_color.clone(),
                                        ),
                                        selected: Some(
                                            selected_collider_color.clone(),
                                        ),
                                    })
                                    .insert(default_collider_color.clone())
                                    .insert(NotShadowCaster)
                                    .insert(PickableBundle::default())
                                    .with_children(|commands| {
                                        commands
                                            .spawn(SceneBundle {
                                                scene: handle,
                                                transform: Transform::from_xyz(
                                                    0.0, -1.0, 0.0,
                                                ),
                                                ..Default::default()
                                            })
                                            .insert(Name::new(format!(
                                                "Tower_Base_Child_{}",
                                                idx
                                            )));
                                    });
                            }
                        });

                    commands
                        .spawn(SpatialBundle::from_transform(
                            Transform::from_xyz(0.3, 2.0, 9.0),
                        ))
                        .insert(PhysicsBundle::moving_entity(Vec3::new(
                            0.4, 0.4, 0.4,
                        )))
                        .insert(Name::new("Bouncy Capsule"))
                        .insert(assets.get_capsule_shape().clone())
                        .insert(selected_collider_color.clone());
                }
            }
            x => info!("Asset Event {:?} not handled", x),
        }
    }
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // set gravity
    rapier_config.gravity = -Vec3::Y;

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 4000.0,
                range: 10000.,
                radius: 10.,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(3.1, 10.0, 8.0),
            ..default()
        })
        .insert(Name::new("Sun"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(15.0).into()),
            material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
            transform: Transform::from_xyz(0., -15., 0.),
            ..default()
        })
        .insert(Name::new("Floor"));
}
