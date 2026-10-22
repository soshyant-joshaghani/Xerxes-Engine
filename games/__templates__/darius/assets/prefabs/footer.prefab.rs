//! The **footer** band: the opaque controls bar across the bottom of a game screen, with its
//! collapse toggle in the corner. A scene adds the controls as children of the instance; a game
//! played with the pointer alone (Qanat) leaves the footer out, and the content grows into it.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const PANEL: Color = Color::srgb(0.047, 0.055, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Footer")
            .with(FooterBand)
            .with(Node::default())
            .with(ZIndex(1))
            .with(RectTransform {
                anchor_min: Vec2::new(0.0, 0.0),
                anchor_max: Vec2::new(1.0, 0.22),
                pivot: Vec2::new(0.5, 0.5),
                position: Vec2::new(0.0, 0.0),
                size: Vec2::new(0.0, 0.0),
            })
            .with(BackgroundColor(PANEL))
            .child(
                Object::new("Footer Toggle")
                    .with(Button)
                    .with(BandToggle(Band::Footer))
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
                        Object::new("Footer Toggle Label")
                            .with(BandToggleLabel(Band::Footer))
                            .with(Text::new("-"))
                            .with(TextFont::from_font_size(16.0))
                            .with(TextColor(TEXT)),
                    ),
            ),
    }
}
