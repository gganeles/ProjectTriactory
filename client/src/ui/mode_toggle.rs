//! Temporary debug HUD: a small corner button to flip between `GameMode::BuildMode` and
//! `GameMode::PlayMode` while `PlayMode` has no real trigger (no joystick/build UI yet) —
//! otherwise the state would exist with no way to reach it. Expected to be replaced by whatever
//! real UI ends up deciding the mode (e.g. a build-menu "Play" button).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use triactory_shared::{AppState, GameMode};

pub struct ModeTogglePlugin;

impl Plugin for ModeTogglePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            mode_toggle_ui.run_if(in_state(AppState::Game)),
        );
    }
}

fn mode_toggle_ui(
    mut contexts: EguiContexts,
    mode: Res<State<GameMode>>,
    mut next_mode: ResMut<NextState<GameMode>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("mode_toggle"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 12.0))
        .show(ctx, |ui| {
            ui.label(format!("Mode: {:?}", mode.get()));
            let next = match mode.get() {
                GameMode::BuildMode => GameMode::PlayMode,
                GameMode::PlayMode => GameMode::BuildMode,
            };
            if ui.button(format!("Switch to {next:?}")).clicked() {
                next_mode.set(next);
            }
        });
    Ok(())
}
