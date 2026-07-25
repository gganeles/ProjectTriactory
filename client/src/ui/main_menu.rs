//! The main menu screen: just a "Start" button that moves the app into [`AppState::Game`].

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use triactory_shared::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            main_menu_ui.run_if(in_state(AppState::MainMenu)),
        );
    }
}

fn main_menu_ui(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("main_menu"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("Triactory").size(48.0));
                ui.add_space(24.0);
                if ui.button(egui::RichText::new("Start").size(24.0)).clicked() {
                    next_state.set(AppState::Game);
                }
            });
        });
    Ok(())
}
