use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_picking::Selection;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::time::Duration;
use strum::IntoEnumIterator;

use crate::{
    Tower, TowerBuildEvent, TowerSideEffects, TowerType, TowerUpgrades,
};

#[derive(Default, Resource)]
struct UiState {
    game_state: GameState,
    wave_timer: Timer,
    enemies_killed: i32,
    money_in_bank: f32,
    health: f32,
    waves_finished: i32,
    force_number: String,
}

pub enum StateUpdateEvent {
    EnemyKilled(f32),
    EnemyReachedPortal,
    StartWave {
        time_of_wave: f32,
        spawn_interval: f32,
    },
    GameWon,
    GameLost,
    EndWave,
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
    entity: Option<(Entity, GlobalTransform, Option<Tower>, Option<TowerType>)>,
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
            StateUpdateEvent::StartWave { time_of_wave, .. } => {
                ui_state
                    .wave_timer
                    .set_duration(Duration::from_secs_f32(*time_of_wave));
                ui_state.wave_timer.unpause();
                ui_state.game_state = GameState::RunningWave;
            }
            StateUpdateEvent::GameWon => {
                ui_state.game_state = GameState::GameWon;
                ui_state.wave_timer.pause();
            }
            StateUpdateEvent::GameLost => {
                ui_state.game_state = GameState::GameLost;
                ui_state.wave_timer.pause();
            }
            StateUpdateEvent::EndWave => {
                ui_state.game_state = GameState::TowerUpgrade;
                ui_state.wave_timer.pause();
                ui_state.waves_finished += 1;
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

fn get_side_effect(
    wave_multiplier: i32,
    force: i32,
) -> Option<TowerSideEffects> {
    let mut rng = thread_rng();
    let weights = WeightedIndex::new(TowerSideEffects::get_weights(
        wave_multiplier,
        force,
    ))
    .unwrap();
    let options = [
        None,
        Some(TowerSideEffects::WeakShot(force as f32)),
        Some(TowerSideEffects::HealShot(force as f32)),
    ];
    options[weights.sample(&mut rng)]
}

fn main_game_screen(
    selections: Query<(
        Entity,
        &Selection,
        &GlobalTransform,
        Option<&TowerType>,
        Option<&Tower>,
    )>,
    mut ui_state: ResMut<UiState>,
    mut egui_ctx: EguiContexts,
    mut ev_tower_build_writer: EventWriter<TowerBuildEvent>,
    mut ev_state_update_writer: EventWriter<StateUpdateEvent>,
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
        for (entity, selection, transform, tower_type, tower) in &selections {
            if selection.selected() {
                info!("Selected entity {:?}", entity);
                current_selection.entity = Some((
                    entity,
                    transform.clone(),
                    tower.clone().map(|t| t.clone()),
                    tower_type.clone().map(|tt| tt.clone()),
                ));
            }
        }
    }
    egui::TopBottomPanel::bottom("bottom_panel")
        .max_height(300.0)
        .min_height(300.0)
        .show(ctx, |ui| {
            ui.heading("Towering side effect");
            if matches!(ui_state.game_state, GameState::RunningWave) {
                    ui.label("Wave running");
                    ui.label(format!(
                        "Remaining wave time: {:.2}",
                        ui_state.wave_timer.remaining_secs()
                    ));
                }

            match ui_state.game_state {
                GameState::TowerUpgrade | GameState::RunningWave => {
                    if !matches!(ui_state.game_state, GameState::RunningWave) {
                        ui.horizontal(|ui| {
                            ui.allocate_ui(egui::Vec2::new(30.0, 30.0), |ui| {
                                if ui.button("Run wave!").clicked() {
                                    ev_state_update_writer.send(
                                        StateUpdateEvent::StartWave {
                                            time_of_wave: 40.0,
                                            spawn_interval: 1.5,
                                        },
                                    );
                                }
                            });
                        });
                    }

                    ui.horizontal(|ui| {
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
                        if let Some((entity, transform, tower, _tower_type)) =
                            current_selection.entity.clone()
                        {
                            ui.separator();

                            if let Some(tower) = tower {
                                ui.label(format!(
                                    "Upgrade option for tower {:#?}",
                                    entity
                                ));
                                ui.horizontal_top(|ui| {
                                    if tower.upgrades.len() > 0 {
                                        ui.label(format!("Existing upgrades: {:?}", tower.upgrades));
                                        ui.allocate_space(egui::Vec2::new(10.0, 10.0));
                                        ui.label(format!("Side effects: {:?}", tower.side_effects));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        for upgrade_option in TowerUpgrades::iter() {
                                            let price = upgrade_option
                                                .get_price(ui_state.waves_finished, ui_state.force_number.parse().unwrap_or(1));

                                            if ui_state.money_in_bank >= price {

                                                ui.allocate_ui(egui::Vec2::new(100.0, 30.0), |ui|{
                                                    if ui
                                                        .button(egui::RichText::new(
                                                            upgrade_option.to_string(),
                                                        ))
                                                        .clicked()
                                                    {
                                                        let force: i32 = ui_state.force_number.parse().unwrap_or(1);
                                                        info!("Fired upgrade event");
                                                        ui_state.money_in_bank -= price;
                                                        ev_tower_build_writer.send(
                                                            TowerBuildEvent::Upgrade {
                                                                entity,
                                                                effect: upgrade_option.set_force(force as f32),
                                                                side_effect: get_side_effect(ui_state.waves_finished, force),
                                                            },
                                                        );
                                                        current_selection.entity = None;
                                                    }
                                                    ui.label(format!("Cost: {:.2}", price));
                                                });
                                            } else  {
                                                ui.label(format!("Not enough money for upgrade: {} need {:.2}", upgrade_option, price));
                                            }
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Force of the upgrade");
                                        ui.text_edit_singleline(&mut ui_state.force_number);
                                    });
                                });
                            } else {
                                ui.label(format!(
                                    "Build options for {:#?}",
                                    entity
                                ));
                                for build_option in TowerType::iter() {
                                    let price = build_option
                                        .get_price(ui_state.waves_finished);

                                    if ui_state.money_in_bank >= price {
                                        if ui
                                            .button(egui::RichText::new(
                                                build_option.to_string(),
                                            ))
                                            .clicked()
                                        {
                                            info!("Fired build event");
                                            ui_state.money_in_bank -= price;
                                            ev_tower_build_writer.send(
                                                TowerBuildEvent::Dispatch {
                                                    entity,
                                                    kind: build_option,
                                                    pos: transform
                                                        .translation(),
                                                },
                                            );
                                            current_selection.entity = None;
                                        }
                                    } else {
                                        ui.label(format!(
                                            "{}: Not enough money need {:.2}",
                                            build_option.to_string(),
                                            price,
                                        ));
                                    }
                                }
                            }
                            ui.separator();
                        }
                    });
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
