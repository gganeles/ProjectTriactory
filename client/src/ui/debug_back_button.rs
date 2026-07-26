//! DEBUG, easy to remove: a small corner button, visible during `AppState::Game`, that returns
//! to `AppState::MainMenu` and asks the server for a freshly-reseeded map (otherwise you'd see
//! the exact same map every time you hit Start again — see
//! `triactory_shared::protocol::RegenerateMap`'s docs). To remove this feature entirely: delete
//! this file, drop the `pub mod debug_back_button;` line in `ui/mod.rs`, drop this plugin from
//! `main.rs`, and revert the `RegenerateMap` message / `MapChannel` direction in
//! `shared/src/protocol/mod.rs` and `regenerate_map_on_request` in `server/src/replication.rs`.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use lightyear::prelude::client::Client;
use lightyear::prelude::*;
use triactory_shared::AppState;
use triactory_shared::protocol::{MapChannel, RegenerateMap};

use crate::world_model::RevealedTiles;

pub struct DebugBackButtonPlugin;

impl Plugin for DebugBackButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            debug_back_button_ui.run_if(in_state(AppState::Game)),
        );
    }
}

fn debug_back_button_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut revealed: ResMut<RevealedTiles>,
    mut client_sender: Single<&mut MessageSender<RegenerateMap>, With<Client>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("debug_back_button"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
        .show(ctx, |ui| {
            if ui.button("Back to Menu (DEBUG)").clicked() {
                next_state.set(AppState::MainMenu);
                // Wait for the server's fresh TilesRevealed rather than briefly re-showing the
                // old map on the next Start.
                revealed.tiles.clear();
                client_sender.send::<MapChannel>(RegenerateMap);
            }
        });
    Ok(())
}
