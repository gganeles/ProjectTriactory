pub mod economy;
pub mod input;
pub mod tech;
pub mod vision;

use bevy::prelude::*;

/// Aggregates the player domain's sub-plugins. Also where `Hero` (marker), `TribeId`,
/// `TribeLeader`, `HeroTile` (current `TriCoord`), and `HeroKinematics` (progress along the
/// current edge crossing — the predicted movement state) will be registered once implemented —
/// core player/hero identity, not specific to any one sub-domain below.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            economy::EconomyPlugin,
            tech::TechPlugin,
            vision::VisionPlugin,
            input::InputPlugin,
        ));
    }
}
