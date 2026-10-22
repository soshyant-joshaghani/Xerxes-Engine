//! A **HUD button**: the small frame every button in a band is built from. The scene names it,
//! gives it its action (a marker component of its own) and adds its label as a child; a primary
//! button replaces the background colour:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab)
//!     .named("Pause Button")
//!     .with(HudButton(Action::TogglePause))
//!     .with(BackgroundColor(PRIMARY))
//!     .child(Object::new("Pause Label").with(Text::new("Pause")) /* ... */)
//! ```
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("HUD Button")
            .with(Button)
            .with(Node {
                padding: UiRect::axes(px(10), px(6)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            })
            .with(BackgroundColor(GHOST)),
    }
}
