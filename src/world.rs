use std::str::FromStr;

use crate::{
    pathmanager::{PathManager, PathManagerUpdate},
    GameAssets, Portal,
};
use bevy::{
    gltf::{Gltf, GltfNode},
    pbr::NotShadowCaster,
    prelude::*,
};
use bevy_mod_picking::*;
use bevy_rapier3d::prelude::*;
use std::fmt::Display;

#[derive(Reflect, Component)]
pub enum TowerBase {
    Bad(String),
    Normal(String),
    Super(String),
}

impl Display for TowerBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TowerBase::Bad(x) => write!(f, "{}", x),
            TowerBase::Normal(x) => write!(f, "{}", x),
            TowerBase::Super(x) => write!(f, "{}", x),
        }
    }
}

impl FromStr for TowerBase {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((_, other)) = s.split_once(",") {
            if let Some((t, extra)) = other.split_once(".") {
                match t {
                    "-1" => Ok(TowerBase::Bad(extra.to_owned())),
                    "0" => Ok(TowerBase::Normal(extra.to_owned())),
                    "1" => Ok(TowerBase::Super(extra.to_owned())),
                    x => {
                        Err(anyhow::anyhow!("unknwown tower type {} found", x))
                    }
                }
            } else {
                Err(anyhow::anyhow!("no . found in towerbase name"))
            }
        } else {
            Err(anyhow::anyhow!(
                "no , delimiting the type and towerbase found"
            ))
        }
    }
}

#[derive(Debug, Reflect, Component, PartialEq, Clone)]
pub struct Proxy {
    pub route_id: i32,
    pub node_id: i32,
    pub kind: ProxyKind,
    pub movement_type: MovementType,
    pub location: Vec3,
}

#[derive(Debug, Reflect, Component)]
pub struct Route {}

#[derive(Debug, Reflect, PartialEq, Eq, Clone)]
pub enum ProxyKind {
    Route,
    Portal,
}

impl Display for ProxyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyKind::Route => write!(f, "route"),
            ProxyKind::Portal => write!(f, "portal"),
        }
    }
}

#[derive(Debug, Reflect, PartialEq, Eq, Clone)]
pub enum MovementType {
    Walking,
    Falling,
}

impl FromStr for Proxy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //Proxy,kind,route_id,node_id,movement_type
        let parts = s.split(",").collect::<Vec<&str>>();
        Ok(Self {
            route_id: parts[2].parse()?,
            node_id: parts[3].parse()?,
            kind: match parts[1] {
                "route" => ProxyKind::Route,
                _ => ProxyKind::Route,
            },
            movement_type: match parts[4] {
                "walk" => MovementType::Walking,
                _ => MovementType::Falling,
            },
            location: Vec3::ZERO,
        })
    }
}

impl Display for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}_{}", self.kind, self.route_id, self.node_id)
    }
}

pub fn world_plugin(app: &mut App) {
    app.register_type::<Proxy>()
        .register_type::<TowerBase>()
        .register_type::<Route>()
        .add_startup_system(spawn_basic_scene)
        .add_system(handle_map_spawn);
}

fn handle_map_spawn(
    mut ev_asset: EventReader<AssetEvent<Gltf>>,
    mut ev_pathmanager_update: EventWriter<PathManagerUpdate>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    nodes: Res<Assets<GltfNode>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                if handle == assets.map() {
                    let map = assets_gltf.get(handle).unwrap();

                    commands
                        .spawn((
                            SceneBundle {
                                scene: map.default_scene.clone().unwrap(),
                                ..Default::default()
                            },
                            Name::new("Map"),
                            PathManager::new(),
                        ))
                        .with_children(|commands| {
                            for (name, node_handle) in map.named_nodes.iter() {
                                if name.to_lowercase().starts_with("tower") {
                                    let tower_base =
                                        TowerBase::from_str(name).unwrap();
                                    let node = nodes.get(node_handle).unwrap();
                                    commands.spawn((
                                        PbrBundle {
                                            mesh: assets
                                                .get_capsule_shape()
                                                .clone(),
                                            material: assets
                                                .default_collider_color
                                                .clone(),
                                            transform: node
                                                .transform
                                                .mul_transform(
                                                    Transform::from_xyz(
                                                        0.0, 1.0, 0.0,
                                                    ),
                                                )
                                                .with_scale(Vec3::new(
                                                    1.5, 1.5, 1.5,
                                                )),
                                            ..Default::default()
                                        },
                                        Name::new(format!(
                                            "Tower_Base_{}",
                                            tower_base
                                        )),
                                        Highlighting {
                                            initial: assets
                                                .default_collider_color
                                                .clone(),
                                            hovered: Some(
                                                assets
                                                    .tower_base_selected_color
                                                    .clone(),
                                            ),
                                            pressed: Some(
                                                assets
                                                    .tower_base_selected_color
                                                    .clone(),
                                            ),
                                            selected: Some(
                                                assets
                                                    .tower_base_selected_color
                                                    .clone(),
                                            ),
                                        },
                                        NotShadowCaster,
                                        PickableBundle::default(),
                                        tower_base,
                                    ));
                                } else if name
                                    .to_lowercase()
                                    .starts_with("proxy")
                                {
                                    let mut proxy =
                                        Proxy::from_str(name).unwrap();

                                    let node = nodes.get(node_handle).unwrap();
                                    proxy.location = node.transform.translation;

                                    if matches!(proxy.kind, ProxyKind::Route) {
                                        commands.spawn((
                                            SpatialBundle {
                                                transform: node.transform,
                                                ..Default::default()
                                            },
                                            Name::new(format!(
                                                "Proxy_{}",
                                                proxy
                                            )),
                                            proxy.clone(),
                                        ));

                                        ev_pathmanager_update.send(
                                            PathManagerUpdate::AddNode(proxy),
                                        );
                                    }
                                } else if name
                                    .to_lowercase()
                                    .starts_with("portal")
                                {
                                    let node = nodes.get(node_handle).unwrap();
                                    commands.spawn((
                                        SpatialBundle {
                                            transform: node.transform,
                                            ..Default::default()
                                        },
                                        Name::new("Portal"),
                                        Portal::new(),
                                    ));
                                }
                            }
                        });
                }
            }
            x => info!("Asset Event {:?} not handled", x),
        }
    }
}

fn spawn_basic_scene(
    mut commands: Commands,
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
}
