use bevy::{
    ecs::{event::ManualEventIteratorWithId, query::QuerySingleError},
    prelude::*,
    ui::FocusPolicy,
};
use bevy_mod_picking::*;
use strum::{Display as EnumDisplay, EnumIter, IntoEnumIterator};

use crate::{graphics::CreateParticleSystem, GameAssets};

#[derive(Component)]
pub struct TowerUIRoot;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    pub shooting_timer: Timer,
    pub bullet_offset: Vec3,
}

#[derive(Reflect, Component, EnumIter, EnumDisplay, Copy, Clone)]
pub enum TowerType {
    Test,
}

pub fn tower_plugin(app: &mut App) {
    app.add_system(create_ui_on_selection)
        .add_system(tower_button_clicked);
}

fn create_ui_on_selection(
    mut commands: Commands,
    assets: Res<GameAssets>,
    selections: Query<&Selection>,
    root: Query<Entity, With<TowerUIRoot>>,
) {
    let at_least_one_selected =
        selections.iter().any(|selection| selection.selected());
    match root.get_single() {
        Ok(root) => {
            if !at_least_one_selected {
                commands.entity(root).despawn_recursive();
            }
        }
        Err(QuerySingleError::NoEntities(..)) => {
            if at_least_one_selected {
                create_ui(&mut commands, &assets);
            }
        }
        _ => unreachable!("too many ui tower roots!"),
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
            shooting_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            bullet_offset: Vec3::new(0.0, 1.2, 0.0),
        })
        .insert(tt)
        .with_children(|commands| {
            commands.spawn(PbrBundle {
                mesh: assets.get_capsule_shape().clone(),
                transform: Transform::from_xyz(0.0, -0.8, 0.0)
                    .with_scale(Vec3::new(1.5, 1.5, 1.5)),
                ..Default::default()
            });
        })
        .id()
}

fn tower_button_clicked(
    interaction: Query<(&Interaction, &TowerType), Changed<Interaction>>,
    mut commands: Commands,
    selection: Query<(Entity, &Selection, &Transform)>,
    assets: Res<GameAssets>,
    mut particle_events: EventWriter<CreateParticleSystem>,
) {
    for (interaction, tower_type) in &interaction {
        if matches!(interaction, Interaction::Clicked) {
            for (entity, selection, transform) in &selection {
                if selection.selected() {
                    commands.entity(entity).despawn_recursive();
                    spawn_tower(
                        &mut commands,
                        &assets,
                        transform.translation,
                        *tower_type,
                    );
                    particle_events.send(CreateParticleSystem {
                        system: crate::graphics::ParticleSystemType::Landing,
                        transform: *transform,
                    });
                }
            }
        }
    }
}

fn create_ui(commands: &mut Commands, assets: &GameAssets) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(TowerUIRoot)
        .with_children(|commands| {
            for tt in TowerType::iter() {
                commands
                    .spawn(ButtonBundle {
                        style: Style {
                            size: Size::new(
                                Val::Percent(15.0 * 9.0 / 16.0),
                                Val::Percent(15.0),
                            ),
                            align_self: AlignSelf::FlexEnd,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::all(Val::Percent(2.0)),
                            ..Default::default()
                        },
                        background_color: Color::rgba(
                            65.0 / 255.0,
                            123.0 / 255.0,
                            75.0 / 255.0,
                            96.0 / 255.0,
                        )
                        .into(),
                        ..Default::default()
                    })
                    .insert(tt)
                    .insert(Name::new(format!("Button {}", tt.to_string())))
                    .with_children(|commands| {
                        commands.spawn(TextBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Percent(100.0),
                                    Val::Percent(100.0),
                                ),
                                align_self: AlignSelf::Center,
                                margin: UiRect::all(Val::Auto),
                                ..Default::default()
                            },
                            text: Text::from_section(
                                tt.to_string(),
                                TextStyle {
                                    font: assets.font(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            ),
                            focus_policy: FocusPolicy::Pass,
                            ..Default::default()
                        });
                    });
            }
        });
}
