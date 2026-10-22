//! The **header** band: the opaque bar across the top of a game screen, with its collapse toggle
//! in the corner. A scene adds its own content as children of the instance:
//!
//! ```ignore
//! Object::prefab(crate::assets::prefabs::header_prefab::prefab)
//!     .child(Object::new("Info").with(BandContent(Band::Header)) /* ... */)
//! ```
//!
//! The engine's bands (`modules::ui::bands`) size it, hide the `BandContent` children while it is
//! collapsed and keep the game camera in the gap below. The `RectTransform` here is the open
//! header's rectangle, so the editor's Scene view shows the bar where the game puts it.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const PANEL: Color = Color::srgb(0.047, 0.055, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Header")
            .with(HeaderBand)
            .with(Node::default())
            // Above the banner, so the bar stays solid and clickable while the banner only dims
            // the game in the middle.
            .with(ZIndex(1))
            .with(RectTransform {
                anchor_min: Vec2::new(0.0, 0.8),
                anchor_max: Vec2::new(1.0, 1.0),
                pivot: Vec2::new(0.5, 0.5),
                position: Vec2::new(0.0, 0.0),
                size: Vec2::new(0.0, 0.0),
            })
            .with(BackgroundColor(PANEL))
            .child(
                Object::new("Header Toggle")
                    .with(Button)
                    .with(BandToggle(Band::Header))
                    // Top-right corner, 10px in and 8px down. The engine folds the pivot into the
                    // layout box, so the hit-box matches where it draws.
                    .with(RectTransform::anchored(
                        Anchor::TopRight,
                        Vec2::new(-10.0, -8.0),
                        Vec2::new(36.0, 28.0),
                    ))
                    .with(Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    })
                    .with(BackgroundColor(GHOST))
                    .child(
                        Object::new("Header Toggle Label")
                            .with(BandToggleLabel(Band::Header))
                            .with(Text::new("-"))
                            .with(TextFont::from_font_size(16.0))
                            .with(TextColor(TEXT)),
                    ),
            ),
    }
}
