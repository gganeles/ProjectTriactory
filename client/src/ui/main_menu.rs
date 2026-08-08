//! The main menu screen: a "Start" button that moves the app into [`AppState::Game`], plus a
//! DEBUG map-settings picker (map type + player count) that tells the server to regenerate the
//! map before Start is pressed — see `set_map_config_ui`'s docs.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use lightyear::prelude::client::Client;
use lightyear::prelude::*;
use triactory_shared::AppState;
use triactory_shared::config::{MAX_PLAYERS, MIN_PLAYERS};
use triactory_shared::game::map::generation::MapType;
use triactory_shared::protocol::{MapChannel, SetMapConfig};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            main_menu_ui.run_if(in_state(AppState::MainMenu)),
        );
    }
}

const ALL_MAP_TYPES: [MapType; 6] = [
    MapType::Drylands,
    MapType::Lakes,
    MapType::Continents,
    MapType::Pangea,
    MapType::Archipelago,
    MapType::Waterworld,
];

/// DEBUG, easy to remove: the currently-selected map type/player count in the picker below,
/// persisted across frames via `Local` since it's only read/written by `main_menu_ui`. Remove
/// this struct, the picker UI in `main_menu_ui`, `SetMapConfig`
/// (`shared/src/protocol/mod.rs`), and `server/src/replication.rs`'s `set_map_config_on_request`
/// together to fully strip the feature.
struct DebugMapSettings {
    map_type: MapType,
    num_players: u8,
}

impl Default for DebugMapSettings {
    fn default() -> Self {
        Self {
            map_type: MapType::Continents,
            num_players: MIN_PLAYERS,
        }
    }
}

fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut settings: Local<DebugMapSettings>,
    client_sender: Option<Single<&mut MessageSender<SetMapConfig>, With<Client>>>,
) -> Result {
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

                ui.add_space(32.0);
                ui.separator();
                ui.label("Map settings (DEBUG)");
                egui::ComboBox::from_label("Map type")
                    .selected_text(format!("{:?}", settings.map_type))
                    .show_ui(ui, |ui| {
                        for map_type in ALL_MAP_TYPES {
                            ui.selectable_value(
                                &mut settings.map_type,
                                map_type,
                                format!("{map_type:?}"),
                            );
                        }
                    });
                ui.add(
                    egui::Slider::new(&mut settings.num_players, MIN_PLAYERS..=MAX_PLAYERS)
                        .text("Players"),
                );
                if ui.button("Apply map settings (DEBUG)").clicked()
                    && let Some(mut sender) = client_sender
                {
                    sender.send::<MapChannel>(SetMapConfig {
                        map_type: settings.map_type,
                        num_players: settings.num_players,
                    });
                }
            });
        });
    Ok(())
}
