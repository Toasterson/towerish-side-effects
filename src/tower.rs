use bevy::{ecs::query::QuerySingleError, prelude::*, ui::FocusPolicy};
use bevy_mod_picking::*;
use strum::{Display as EnumDisplay, EnumIter, IntoEnumIterator};

use crate::GameAssets;

#[derive(Component)]
pub struct TowerUIRoot;

#[derive(Reflect, Component, EnumIter, EnumDisplay, Copy, Clone)]
pub enum TowerType {
    Test,
}

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(create_ui_on_selection)
            .add_system(tower_button_clicked);
    }
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

fn tower_button_clicked(
    interaction: Query<(&Interaction, &TowerType), Changed<Interaction>>,
    mut _commands: Commands,
    selection: Query<(Entity, &Selection, &Transform)>,
    _assets: Res<GameAssets>,
) {
    for (interaction, tower_type) in &interaction {
        if matches!(interaction, Interaction::Clicked) {
            for (_entity, selection, _transform) in &selection {
                if selection.selected() {
                    info!("Tower button {} clicked", tower_type.to_string());
                    //commands.entity(entity).despawn_recursive();
                    //crate::tower::spawn_tower(
                    //    &mut commands,
                    //    &assets,
                    //    transform.translation,
                    //    *tower_type,
                    //);
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
