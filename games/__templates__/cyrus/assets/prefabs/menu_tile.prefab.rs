//! A **tile of the main menu's grid**: one game, as a button that starts its level. The scene
//! names it, gives it its level (`LevelButton`) and adds its three lines as children; a game that
//! is not built yet replaces the two colours:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::menu_tile_prefab::prefab)
//!     .named("Qanat Tile")
//!     .with(LevelButton(1))
//!     .child(Object::new("Qanat Name").with(Text::new("Qanat")) /* ... */)
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const PANEL: Color = Color::srgb(0.090, 0.122, 0.157);
const AMBER: Color = Color::srgb(0.878, 0.659, 0.243);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Tile")
            .with(Button)
            .with(Node {
                width: px(196),
                height: px(58),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(px(10), px(6)),
                row_gap: px(2),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            })
            .with(BackgroundColor(PANEL))
            .with(BorderColor::all(AMBER)),
    }
}
