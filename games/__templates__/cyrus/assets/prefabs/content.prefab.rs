//! The **content** band: the gap between the header and the footer, which is where the game is
//! seen. It draws nothing of its own; it is there so the HUD can put something over the game (a
//! crosshair, a prompt) in exactly the rectangle the game fills. The engine's bands resize it as
//! the bars open and close.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Content")
            .with(ContentBand)
            .with(Node::default())
            .with(RectTransform {
                anchor_min: Vec2::new(0.0, 0.22),
                anchor_max: Vec2::new(1.0, 0.8),
                pivot: Vec2::new(0.5, 0.5),
                position: Vec2::new(0.0, 0.0),
                size: Vec2::new(0.0, 0.0),
            }),
    }
}
