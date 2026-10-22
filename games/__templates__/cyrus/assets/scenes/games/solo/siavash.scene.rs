//! Siavash (Solo / Score): an endless run on three lanes (slice 1). The knight, the camera, the
//! road and the HUD are here as plain data; the enemies are made at run time by
//! `src/siavash/play.rs`. Design: `__docs__/gdd/cyrus/solo-score-siavash.md`.
//!
//! Everything in it is a prefab of its own: the camera, the sun and the ground with this game's
//! pose on them, the knight, and — spawned while the run goes on, from the same prefabs — the
//! goons, the flyers and the road's chunks (`src/siavash/play.rs`).
//!
//! **The road runs toward +Z** — with +Y up, that is the front axis a glTF model is exported on —
//! so the camera stands behind the knight at -Z and sees his back, and the knight's model needs no
//! rotation of its own. The knight himself is a prefab (`prefabs/games/solo/siavash/knight`): the
//! test character's `.glb`.
//!
//! The screen is the engine's bands: a header (the HUD) over the content (the road). There is no
//! footer — the player drags on the content.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::siavash::{Banner, BannerText, DistanceText, RestartButton, Road};

const TEXT: Color = Color::srgb(0.914, 0.902, 0.863);
const DIM: Color = Color::srgb(0.596, 0.631, 0.675);

/// The chase camera's vertical field of view, in radians (about 31 degrees).
const LENS: f32 = 0.54;

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D3,
        objects: vec![
            // The camera, the sun and the ground are prefabs with this game's own pose: behind the
            // knight, lit from behind the camera, and the road's long desert.
            Object::prefab(crate::assets::prefabs::camera_prefab::prefab)
                .with(
                    Transform::from_xyz(0.0, 3.4, -7.0)
                        .looking_at(Vec3::new(0.0, 1.0, 3.0), Vec3::Y),
                )
                // A chase camera on a long lens: the knight reads at the far end of it, and the
                // road behind him does not race away as it would from a camera moved this close
                // on a wide one. The bands give this angle to the content gap, whichever bars are
                // open (`modules::ui::bands`), so it is the framing of the run on every screen.
                .with(Projection::Perspective(PerspectiveProjection {
                    fov: LENS,
                    ..default()
                })),
            Object::prefab(crate::assets::prefabs::sun_prefab::prefab)
                .with(Transform::from_xyz(4.0, 10.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y)),
            Object::prefab(crate::assets::prefabs::games::solo::siavash::ground_prefab::prefab),
            Object::prefab(crate::assets::prefabs::games::solo::siavash::knight_prefab::prefab),
            Object::new("Road").with(Road).with(Transform::default()),
            Object::new("HUD")
                .with(Canvas::overlay())
                .child(
                    Object::prefab(crate::assets::prefabs::header_prefab::prefab)
                        .child(
                            Object::new("Info")
                                .with(BandContent(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::MiddleLeft,
                                    Vec2::new(16.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    ..default()
                                })
                                .child(
                                    Object::new("Distance")
                                        .with(DistanceText)
                                        .with(Text::new("0 m   0 down"))
                                        .with(TextFont::from_font_size(22.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Controls Hint")
                                        .with(Text::new("A D or swipe side = lane   W or swipe up = jump   S or swipe down = roll / dive   |   Space or double tap shield   R"))
                                        .with(TextFont::from_font_size(13.0))
                                        .with(TextColor(DIM)),
                                ),
                        )
                        .child(
                            Object::new("Buttons")
                                .with(BandContent(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::MiddleRight,
                                    Vec2::new(-52.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                })
                                .child(
                                    Object::prefab(crate::assets::prefabs::hud_button_prefab::prefab)
                                        .named("Restart Button")
                                        .with(RestartButton)
                                        .child(
                                            Object::new("Restart Label")
                                                .with(Text::new("Restart"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(Object::prefab(
                                    crate::assets::prefabs::hud_back_button_prefab::prefab,
                                )),
                        ),
                )
                // The road is seen through the content band; the bands keep the camera inside it.
                .child(Object::prefab(
                    crate::assets::prefabs::content_prefab::prefab,
                ))
                .child(
                    Object::prefab(crate::assets::prefabs::banner_prefab::prefab)
                        .with(Banner)
                        .child(
                            Object::new("Banner Text")
                                .with(BannerText)
                                .with(Text::new(""))
                                .with(TextFont::from_font_size(28.0))
                                .with(TextColor(TEXT)),
                        ),
                ),
        ],
    }
}
