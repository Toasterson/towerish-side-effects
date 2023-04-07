use bevy::{prelude::*, ui::FocusPolicy};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_picking::{PickingPlugin, Selection};
use strum::IntoEnumIterator;

use crate::{TowerBuildEvent, TowerType};

#[derive(Default, Resource)]
struct UiState {
    game_state: GameState,
}

#[derive(Default)]
enum GameState {
    RunningWave,
    #[default]
    TowerUpgrade,
}

pub fn ui_plugin(app: &mut App) {
    app.init_resource::<UiState>()
        .add_plugin(EguiPlugin)
        .add_startup_system(configure_ui)
        .add_system(main_game_screen);
}

#[derive(Default)]
struct CurrentSelection {
    entity: Option<(Entity, GlobalTransform)>,
}

fn main_game_screen(
    selections: Query<(Entity, &Selection, &GlobalTransform)>,
    mut ui_state: ResMut<UiState>,
    mut egui_ctx: EguiContexts,
    mut ev_tower_build_writer: EventWriter<TowerBuildEvent>,
    mut current_selection: Local<CurrentSelection>,
) {
    let ctx = egui_ctx.ctx_mut();
    if !ctx.wants_pointer_input() {
        for (entity, selection, tranform) in &selections {
            if selection.selected() {
                current_selection.entity = Some((entity, tranform.clone()));
            }
        }
    }
    egui::TopBottomPanel::bottom("bottom_panel")
        .max_height(200.0)
        .min_height(30.0)
        .show(ctx, |ui| {
            ui.heading("Towering side effect");
            ui.vertical(|ui| {
                if let Some((entity, transform)) = current_selection.entity {
                    ui.label(format!("Build options for {:#?}", entity));
                    for build_option in TowerType::iter() {
                        if ui
                            .button(egui::RichText::new(
                                build_option.to_string(),
                            ))
                            .clicked()
                        {
                            info!("Fired build event");
                            ev_tower_build_writer.send(
                                TowerBuildEvent::Dispatch {
                                    entity,
                                    kind: TowerType::Test,
                                    pos: transform.translation(),
                                },
                            );
                            current_selection.entity = None;
                        }
                    }
                    ui.separator();
                }
            });
        });
}

fn configure_ui(
    mut commands: Commands,
    mut egui_ctx: EguiContexts,
    window: Query<Entity, With<Window>>,
) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
    commands
        .entity(window.single())
        .insert(FocusPolicy::Pass)
        .insert(bevy_mod_picking::NoDeselect)
        .insert(bevy_mod_picking::PickingBlocker);
}
