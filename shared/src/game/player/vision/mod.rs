use bevy::prelude::*;

pub struct VisionPlugin;

impl Plugin for VisionPlugin {
    fn build(&self, _app: &mut App) {
        // stub — registers VisionSource { range } component (heroes reveal VISION_RANGE around themselves)
    }
}
