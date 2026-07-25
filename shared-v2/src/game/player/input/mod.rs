use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, _app: &mut App) {
        // stub — registers HeroInput, handles tap-to-move / joystick
    }
}
