use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_picking::Selection;
use strum::IntoEnumIterator;

use crate::{TowerBuildEvent, TowerType};

#[derive(Default, Resource)]
pub struct UiState {
    pub game_state: GameState,
    pub wave_timer: Timer,
    enemies_killed: i32,
    money_in_bank: f32,
    health: f32,
}

pub enum StateUpdateEvent {
    EnemyKilled(f32),
    EnemyReachedPortal,
}

#[derive(Default)]
pub enum GameState {
    RunningWave,
    #[default]
    TowerUpgrade,
    GameWon,
    GameLost,
}

pub fn ui_plugin(app: &mut App) {
    app.init_resource::<UiState>()
        .add_event::<StateUpdateEvent>()
        .add_plugin(EguiPlugin)
        .add_startup_system(configure_ui)
        .add_startup_system(configure_ui_state)
        .add_system(main_game_screen)
        .add_system(stat_window)
        .add_system(state_update_handler);
}

#[derive(Default)]
struct CurrentSelection {
    entity: Option<(Entity, GlobalTransform)>,
}

fn state_update_handler(
    mut ev_state_update: EventReader<StateUpdateEvent>,
    mut ui_state: ResMut<UiState>,
) {
    for event in ev_state_update.iter() {
        match event {
            StateUpdateEvent::EnemyKilled(reward) => {
                ui_state.enemies_killed += 1;
                ui_state.money_in_bank += reward;
            }
            StateUpdateEvent::EnemyReachedPortal => {
                ui_state.health -= 1.;
                if ui_state.health < 0. {
                    ui_state.game_state = GameState::GameLost;
                }
            }
        }
    }
}

fn stat_window(ui_state: Res<UiState>, mut egui_ctx: EguiContexts) {
    let ctx = egui_ctx.ctx_mut();
    egui::Window::new("Statistics")
        .interactable(false)
        .default_size(egui::Vec2::new(30.0, 20.0))
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Money in bank {:.2}",
                    ui_state.money_in_bank
                ));
            });
            ui.horizontal(|ui| {
                ui.label(format!("Enemies killed {}", ui_state.enemies_killed));
            });
            ui.horizontal(|ui| {
                ui.label(format!("Health: {:.0}", ui_state.health));
            });
        });
}

fn main_game_screen(
    selections: Query<(Entity, &Selection, &GlobalTransform)>,
    mut ui_state: ResMut<UiState>,
    mut egui_ctx: EguiContexts,
    mut ev_tower_build_writer: EventWriter<TowerBuildEvent>,
    mut current_selection: Local<CurrentSelection>,
    time: Res<Time>,
) {
    if matches!(ui_state.game_state, GameState::RunningWave) {
        ui_state.wave_timer.tick(time.delta());
        if ui_state.wave_timer.just_finished() {
            ui_state.game_state = GameState::TowerUpgrade;
            ui_state.wave_timer.pause();
        }
    }
    let ctx = egui_ctx.ctx_mut();
    if !ctx.wants_pointer_input() {
        for (entity, selection, transform) in &selections {
            if selection.selected() {
                current_selection.entity = Some((entity, transform.clone()));
            }
        }
    }
    egui::TopBottomPanel::bottom("bottom_panel")
        .max_height(200.0)
        .min_height(30.0)
        .show(ctx, |ui| {
            ui.heading("Towering side effect");

            match ui_state.game_state {
                GameState::TowerUpgrade => {
                    ui.horizontal(|ui| {
                        ui.allocate_ui(egui::Vec2::new(30.0, 30.0), |ui| {
                            if ui.button("Run wave!").clicked() {
                                ui_state.game_state = GameState::RunningWave;
                                ui_state.wave_timer.reset();
                                ui_state.wave_timer.unpause();
                            }
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.separator();
                        ui.allocate_ui(egui::Vec2::new(30.0, 30.0), |ui| {
                            if ui_state.money_in_bank >= 100.0 {
                                if ui.button("Heal").clicked() {
                                    ui_state.money_in_bank -= 100.0;
                                    ui_state.health += 5.;
                                }
                            } else {
                                ui.label("Not enough money to heal");
                            }
                        });
                    });

                    ui.vertical(|ui| {
                        if let Some((entity, transform)) =
                            current_selection.entity
                        {
                            ui.separator();
                            ui.label(format!(
                                "Build options for {:#?}",
                                entity
                            ));
                            for build_option in TowerType::iter() {
                                if ui_state.money_in_bank >= 500.0 {
                                    if ui
                                        .button(egui::RichText::new(
                                            build_option.to_string(),
                                        ))
                                        .clicked()
                                    {
                                        info!("Fired build event");
                                        ui_state.money_in_bank -= 500.0;
                                        ev_tower_build_writer.send(
                                            TowerBuildEvent::Dispatch {
                                                entity,
                                                kind: TowerType::Test,
                                                pos: transform.translation(),
                                            },
                                        );
                                        current_selection.entity = None;
                                    }
                                } else {
                                    ui.label(format!(
                                        "{}: Not enough money",
                                        build_option.to_string()
                                    ));
                                }
                            }
                            ui.separator();
                        }
                    });
                }
                GameState::RunningWave => {
                    ui.label("Wave running");
                    ui.label(format!(
                        "Remaining wave time: {:.2}",
                        ui_state.wave_timer.remaining_secs()
                    ));
                }
                GameState::GameWon => {
                    ui.label("You won");
                }
                GameState::GameLost => {
                    ui.label("You Lost");
                }
            }
        });
}

fn configure_ui(mut egui_ctx: EguiContexts) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn configure_ui_state(mut ui_state: ResMut<UiState>) {
    ui_state.wave_timer = Timer::from_seconds(40.0, TimerMode::Once);
    ui_state.money_in_bank = 1000.0;
    ui_state.health = 5.;
}
