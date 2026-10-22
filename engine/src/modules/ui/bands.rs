//! **Bands**: the header / content / footer layout every Xerxes game screen is built from. The
//! window is one column of three rows — the HUD **header** at the top, the **content** (the game)
//! in the middle, the **controls footer** at the bottom — and the three never overlap.
//!
//! ```ignore
//! Object::new("HUD")
//!     .with(Canvas::overlay())
//!     .child(Object::prefab(header).child(/* title, buttons, ... */))
//!     .child(Object::prefab(content))   // the gap the game is seen through
//!     .child(Object::prefab(footer).child(/* the controls */))
//! ```
//!
//! A band is a node of a screen-space canvas, marked [`HeaderBand`] / [`ContentBand`] /
//! [`FooterBand`]; this module sizes it with a [`RectTransform`] from [`BandSettings`] and keeps
//! the game's camera inside the content:
//!
//! - a **2D** game camera ([`GameCamera2D`]) is zoomed out by `1 / content` and shifted into the
//!   gap, so the world fits between the bars (an orthographic projection only);
//! - a **3D** game camera ([`GameCamera3D`]) keeps its projection and is shifted with a
//!   [`SubCameraView`], so what it looks at sits in the middle of the gap.
//!
//! The HUD is a canvas of its own, drawn over the whole window by the UI camera, so moving the
//! game camera never moves the bars. A band a scene does not have is simply absent: Qanat has a
//! header and no footer, and the content grows into the footer's place.
//!
//! Collapsing a band ([`BandToggle`]) leaves a thin strip with its toggle, hides everything
//! marked [`BandContent`], and gives the room to the content.
//!
//! On Android the window draws edge-to-edge into the display cutout and under the navigation bar,
//! so every band is inset by the engine's [`SafeArea`] (zero on web and desktop). The strip the
//! inset leaves beside a bar is not a gap: a 2D camera clears to [`BandSettings::frame`], so the
//! dark HUD frame still reads as reaching the screen corners.
//!
//! The frame is the **letterbox a zoomed-out 2D camera leaves**, and nothing else. A 3D camera
//! fills the window, so it leaves no letterbox: it keeps its own [`Sky`], and a scene that gives
//! it none keeps Bevy's default. Painting a 3D sky in the frame colour made the bars and the sky
//! one dark shape, and collapsing the header then *looked* like it grew — the strip was small, but
//! the sky it uncovered was the same colour as the bar.

use bevy::camera::{Camera, ClearColorConfig, SubCameraView};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::modules::camera::{GameCamera2D, GameCamera3D, Sky};
use crate::modules::flow::Flow;
use crate::modules::safe_area::SafeArea;
use crate::modules::ui::RectTransform;

/// The three rows of the screen. [`Band::Content`] is the game's own row: it has no height of its
/// own and no toggle — it is what the header and the footer leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Header,
    Content,
    Footer,
}

/// How tall the bars are and what colour the frame around the game is.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BandSettings {
    /// Height of an open header, a fraction of the window height.
    pub header: f32,
    /// Height of an open footer (the controls row).
    pub footer: f32,
    /// Height of a collapsed band: a thin strip that still shows its toggle.
    pub collapsed: f32,
    /// The colour of the bars and of everything around the game, so the whole non-game area
    /// reads as one solid frame (the game camera clears to it).
    pub frame: Color,
}

impl Default for BandSettings {
    fn default() -> Self {
        Self {
            header: 0.20,
            // The content keeps the rest, so it is always the tallest row.
            footer: 0.22,
            collapsed: 0.05,
            frame: Color::srgb(0.047, 0.055, 0.071),
        }
    }
}

/// Which bands are collapsed. Reset to open when a level starts.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Bands {
    pub header_collapsed: bool,
    pub footer_collapsed: bool,
}

impl Bands {
    pub fn collapsed(&self, band: Band) -> bool {
        match band {
            Band::Header => self.header_collapsed,
            Band::Footer => self.footer_collapsed,
            Band::Content => false,
        }
    }

    fn toggle(&mut self, band: Band) {
        match band {
            Band::Header => self.header_collapsed = !self.header_collapsed,
            Band::Footer => self.footer_collapsed = !self.footer_collapsed,
            Band::Content => {}
        }
    }
}

/// The header band node (top row). Give it an opaque background in the scene.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HeaderBand;

/// The footer band node (bottom row). Omit it (Qanat) for a game with no controls row.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FooterBand;

/// The content band node: the gap between the bars, where the game is seen. A scene can leave it
/// out; it is there so the HUD can put something over the game (a banner, a crosshair) in exactly
/// the rectangle the game fills.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ContentBand;

/// A band's content, hidden while the band is collapsed (the toggle itself is not marked, so it
/// stays).
#[derive(Component, Debug, Clone, Copy)]
pub struct BandContent(pub Band);

/// A button that collapses / expands its band.
#[derive(Component, Debug, Clone, Copy)]
pub struct BandToggle(pub Band);

/// The toggle button's label: this module writes `-` (open) or `+` (collapsed) into it.
#[derive(Component, Debug, Clone, Copy)]
pub struct BandToggleLabel(pub Band);

/// The game camera as the scene authored it, captured before this module moves it, so every
/// shift and zoom is measured from what the scene meant (Qanat points its camera at the room, not
/// the origin) instead of from last frame's result.
#[derive(Component, Debug, Clone, Copy)]
pub struct CameraBase {
    pub translation: Vec3,
    /// The vertical field of view of a perspective camera, in radians (0 for any other).
    pub fov: f32,
}

/// Sizes the bands and keeps the game camera in the content. Added by [`super::UiPlugin`], so
/// every game has it; a scene without bands is left alone.
pub struct BandsPlugin;

impl Plugin for BandsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BandSettings>()
            .init_resource::<Bands>()
            .add_systems(OnEnter(Flow::Level), reset_bands)
            .add_systems(
                Update,
                (toggle_bands, capture_base, apply_bands)
                    .chain()
                    .run_if(in_state(Flow::Level)),
            );
    }
}

/// A fresh level starts with both bands open.
fn reset_bands(mut bands: ResMut<Bands>) {
    *bands = Bands::default();
}

/// Whether this module paints the camera's background, and in what colour.
///
/// Only a **2D** camera with bars around it: it is zoomed out by `1 / content`, so the game sits
/// in the gap and the rest of the window is letterbox, which the frame colour turns into one solid
/// HUD frame. A 3D camera fills the window — what is above the game there is the scene's sky, not
/// letterbox — so it keeps its own, and so does any camera a scene gave a [`Sky`].
fn frame_clear(flat: bool, has_bars: bool, has_sky: bool, frame: Color) -> Option<Color> {
    (flat && has_bars && !has_sky).then_some(frame)
}

/// Remembers where each game camera started (so the shift is measured from there) and letterboxes
/// a 2D game in the frame colour (see [`frame_clear`]).
#[allow(clippy::type_complexity)]
fn capture_base(
    settings: Res<BandSettings>,
    mut cameras: Query<
        (
            Entity,
            &Transform,
            Option<&Projection>,
            &mut Camera,
            Has<GameCamera2D>,
            Has<Sky>,
        ),
        (
            Or<(With<GameCamera2D>, With<GameCamera3D>)>,
            Without<CameraBase>,
        ),
    >,
    bars: Query<(), Or<(With<HeaderBand>, With<FooterBand>)>>,
    mut commands: Commands,
) {
    for (entity, transform, projection, mut camera, flat, has_sky) in &mut cameras {
        let fov = match projection {
            Some(Projection::Perspective(lens)) => lens.fov,
            _ => 0.0,
        };
        commands.entity(entity).insert(CameraBase {
            translation: transform.translation,
            fov,
        });
        if let Some(colour) = frame_clear(flat, !bars.is_empty(), has_sky, settings.frame) {
            camera.clear_color = ClearColorConfig::Custom(colour);
        }
    }
}

/// A press on a band's toggle collapses or expands it.
fn toggle_bands(
    toggles: Query<(&Interaction, &BandToggle), Changed<Interaction>>,
    mut bands: ResMut<Bands>,
) {
    for (interaction, BandToggle(band)) in &toggles {
        if *interaction == Interaction::Pressed {
            bands.toggle(*band);
        }
    }
}

/// The window rectangle a band fills, as anchor fractions: `[min_y, max_y]` of the height, inset
/// from the left and right edges by the safe area (fractions of the width).
fn band_rect(min_y: f32, max_y: f32, left: f32, right: f32) -> RectTransform {
    RectTransform {
        anchor_min: Vec2::new(left, min_y),
        anchor_max: Vec2::new(1.0 - right, max_y),
        pivot: Vec2::splat(0.5),
        position: Vec2::ZERO,
        size: Vec2::ZERO,
    }
}

/// How much of the window each row takes: `(top, bottom, content)`, every one a fraction of the
/// height, the safe-area insets included. A band that is not in the scene takes nothing.
fn rows(
    bands: &Bands,
    settings: &BandSettings,
    has_header: bool,
    has_footer: bool,
    inset_top: f32,
    inset_bottom: f32,
) -> (f32, f32, f32) {
    let height = |band: Band, present: bool, open: f32| {
        if !present {
            0.0
        } else if bands.collapsed(band) {
            settings.collapsed
        } else {
            open
        }
    };
    let top = inset_top + height(Band::Header, has_header, settings.header);
    let bottom = inset_bottom + height(Band::Footer, has_footer, settings.footer);
    (top, bottom, (1.0 - top - bottom).clamp(0.1, 1.0))
}

#[allow(clippy::type_complexity)]
fn apply_bands(
    bands: Res<Bands>,
    settings: Res<BandSettings>,
    safe: Res<SafeArea>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut headers: Query<&mut RectTransform, (With<HeaderBand>, Without<FooterBand>)>,
    mut footers: Query<&mut RectTransform, (With<FooterBand>, Without<HeaderBand>)>,
    mut contents: Query<
        &mut RectTransform,
        (With<ContentBand>, Without<HeaderBand>, Without<FooterBand>),
    >,
    mut hidden: Query<(&BandContent, &mut Node)>,
    mut labels: Query<(&BandToggleLabel, &mut Text)>,
    mut flat: Query<
        (&mut Projection, &mut Transform, &CameraBase),
        (With<GameCamera2D>, Without<GameCamera3D>),
    >,
    mut deep: Query<
        (&mut Camera, &mut Projection, &CameraBase),
        (With<GameCamera3D>, Without<GameCamera2D>),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (win_w, win_h) = (window.width().max(1.0), window.height().max(1.0));
    // The safe area (device-reserved edges, logical px) as fractions of the window. On Android a
    // camera cutout or the nav bar eats into one or more edges; off Android it is all zero.
    // Clamp each inset: a real nav bar / cutout is a small fraction of the edge, but the content
    // rect can report a transient, outsized value while the system UI is still settling on launch;
    // clamping keeps that from briefly collapsing the game or throwing the bars far inward.
    let side = |px: f32, win: f32| (px / win).clamp(0.0, 0.3);
    let (inset_left, inset_right) = (side(safe.left, win_w), side(safe.right, win_w));
    let (inset_top, inset_bottom) = (side(safe.top, win_h), side(safe.bottom, win_h));
    let (top, bottom, content) = rows(
        &bands,
        &settings,
        !headers.is_empty(),
        !footers.is_empty(),
        inset_top,
        inset_bottom,
    );

    // Each bar is inset from the reserved edges so its items stay clear of the nav bar / cutout.
    let place = |rect: &mut RectTransform, min_y: f32, max_y: f32| {
        let want = band_rect(min_y, max_y, inset_left, inset_right);
        if *rect != want {
            *rect = want;
        }
    };
    for mut rect in &mut headers {
        place(&mut rect, 1.0 - top, 1.0 - inset_top);
    }
    for mut rect in &mut footers {
        place(&mut rect, inset_bottom, bottom);
    }
    for mut rect in &mut contents {
        place(&mut rect, bottom, 1.0 - top);
    }
    for (BandContent(band), mut node) in &mut hidden {
        let display = if bands.collapsed(*band) {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (BandToggleLabel(band), mut text) in &mut labels {
        let wanted = if bands.collapsed(*band) { "+" } else { "-" };
        if text.0 != wanted {
            text.0 = wanted.into();
        }
    }

    // 2D: zoom the orthographic camera out by 1/content so the world fits the gap, and shift it
    // so the world sits in the gap (centred when top and bottom are equal; lower when the top
    // reaches further, e.g. Qanat's header-only layout).
    for (mut projection, mut transform, base) in &mut flat {
        if let Projection::Orthographic(ortho) = &mut *projection {
            let scale = 1.0 / content;
            if (ortho.scale - scale).abs() > f32::EPSILON {
                ortho.scale = scale;
            }
            let shift = (top - bottom) / 2.0 * ortho.area.height();
            let from = base.translation;
            let wanted = Vec3::new(from.x, from.y + shift, from.z);
            if transform.translation != wanted {
                transform.translation = wanted;
            }
        }
    }
    // 3D: the gap is the picture — [`gap_view`] and [`gap_fov`] between them frame the camera's
    // own angle in the content gap, wherever the bars leave it, exactly as the 2D zoom above
    // makes the gap frame the world. Collapsing a bar makes the game bigger; nothing slides.
    let (width, height) = (window.physical_width(), window.physical_height());
    for (mut camera, mut projection, base) in &mut deep {
        let want = gap_view(top, bottom, width, height);
        if camera.sub_camera_view != want {
            camera.viewport = None;
            camera.sub_camera_view = want;
        }
        if let Projection::Perspective(lens) = &mut *projection {
            let fov = gap_fov(base.fov, top, bottom, content);
            if (lens.fov - fov).abs() > 1e-4 {
                lens.fov = fov;
            }
        }
    }
}

/// The slice of the image the window shows, so that the camera's axis lands in the middle of the
/// **content gap** rather than the middle of the window.
///
/// The image is rendered as if the window were `|top - bottom|` of its height taller, and that
/// extra strip is cropped off the side the thicker bar is on: a header-only layout (Siavash) drops
/// a strip from the top, so everything the camera sees sits lower, in the gap under the bar.
///
/// `offset` is measured from the **bottom** of the full image (Bevy turns it into a top-down one
/// inside `get_clip_from_view_for_sub`), which is why a thick top bar asks for offset 0 and not
/// for the strip's height — the mirror image of that reads correctly and is wrong, and it showed
/// up as the game sliding down out of the gap when a bar was collapsed.
fn gap_view(top: f32, bottom: f32, width: u32, height: u32) -> Option<SubCameraView> {
    let lift = ((top - bottom).abs() * height as f32).round();
    (lift >= 1.0).then(|| SubCameraView {
        full_size: UVec2::new(width, height + lift as u32),
        size: UVec2::new(width, height),
        offset: Vec2::new(0.0, if top > bottom { 0.0 } else { lift }),
    })
}

/// The vertical field of view a 3D camera must be given so that the **content gap** frames the
/// angle the scene authored, `base_fov`.
///
/// Two things are undone at once. The gap is only `content` of the window, so the window must
/// frame `1 / content` of the angle — the same factor the 2D camera zooms out by. And the image is
/// rendered `|top - bottom|` taller than the window and cropped back (that is what moves the game
/// into the gap), so the angle has to cover the part that is thrown away as well.
///
/// With no bars both factors are 1 and the camera keeps the scene's own lens.
fn gap_fov(base_fov: f32, top: f32, bottom: f32, content: f32) -> f32 {
    let cropped = 1.0 + (top - bottom).abs();
    let half = (base_fov / 2.0).tan() * cropped / content.max(0.01);
    // A field of view can approach a half-turn but never reach it.
    (2.0 * half.atan()).min(std::f32::consts::PI * 0.98)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> BandSettings {
        BandSettings::default()
    }

    #[test]
    fn the_content_is_what_the_bars_leave() {
        let open = Bands::default();
        let (top, bottom, content) = rows(&open, &settings(), true, true, 0.0, 0.0);
        assert_eq!(top, 0.20);
        assert_eq!(bottom, 0.22);
        assert!((content - 0.58).abs() < 1e-6);
        assert!(content > top && content > bottom, "the game is the tallest");

        // A game with no footer (Qanat): the content takes the footer's place.
        let (_, bottom, content) = rows(&open, &settings(), true, false, 0.0, 0.0);
        assert_eq!(bottom, 0.0);
        assert!((content - 0.80).abs() < 1e-6);

        // Collapsed bands leave a strip each, and the safe area is on top of both.
        let shut = Bands {
            header_collapsed: true,
            footer_collapsed: true,
        };
        let (top, bottom, content) = rows(&shut, &settings(), true, true, 0.1, 0.0);
        assert!((top - 0.15).abs() < 1e-6, "0.1 inset + 0.05 strip");
        assert_eq!(bottom, 0.05);
        assert!((content - 0.80).abs() < 1e-6);
    }

    /// What the **content gap** actually shows, on the image plane, once Bevy has applied the
    /// sub-view and the widened lens: `(bottom, top)` of the gap in units of `tan(fov / 2)`.
    ///
    /// The arithmetic is `Projection::get_clip_from_view_for_sub`'s, repeated here on purpose —
    /// this is the one place that says what the bands promise a 3D scene, and a test that only
    /// asked the engine's own helpers what they computed would agree with any mistake in them.
    fn gap_shows(base_fov: f32, top: f32, bottom: f32) -> (f32, f32) {
        const HEIGHT: u32 = 1000;
        let content = 1.0 - top - bottom;
        let fov = gap_fov(base_fov, top, bottom, content);
        let view = gap_view(top, bottom, 1600, HEIGHT);

        // The window's own slice of the frustum.
        let half = (fov / 2.0).tan();
        let (window_low, window_high) = match view {
            None => (-half, half),
            Some(view) => {
                let full = view.full_size.y as f32;
                let sub = view.size.y as f32;
                // Bevy's y runs from the top of the full image down.
                let from_top = full - (view.offset.y + sub);
                (
                    -half + 2.0 * half * from_top / full,
                    -half + 2.0 * half * (from_top + sub) / full,
                )
            }
        };
        // The gap is the band the bars leave, measured up from the bottom of the window.
        let span = window_high - window_low;
        (window_low + span * bottom, window_low + span * (1.0 - top))
    }

    #[test]
    fn the_gap_frames_the_angle_the_scene_asked_for() {
        let base = std::f32::consts::FRAC_PI_4;
        let want = (base / 2.0).tan();

        // A header and a footer, open; a header alone (Siavash); that header collapsed; and no
        // bars at all. Whatever the bars do, the gap shows the camera's own field of view, and
        // shows it centred on the camera's axis: the picture neither shrinks nor slides.
        for (top, bottom) in [
            (0.20, 0.22),
            (0.20, 0.0),
            (0.05, 0.0),
            (0.0, 0.22),
            (0.0, 0.0),
        ] {
            let (low, high) = gap_shows(base, top, bottom);
            assert!(
                (high - want).abs() < 1e-3 && (low + want).abs() < 1e-3,
                "top {top}, bottom {bottom}: the gap framed {low}..{high}, not -{want}..{want}"
            );
        }

        // No bars at all: the scene's own lens, untouched.
        assert!((gap_fov(base, 0.0, 0.0, 1.0) - base).abs() < 1e-6);

        // Collapsing the header leaves a taller gap, so less of the world is spread over the
        // window: the same world drawn bigger, instead of sky uncovered above it.
        let open = gap_fov(base, 0.20, 0.0, 0.80);
        let shut = gap_fov(base, 0.05, 0.0, 0.95);
        assert!(shut < open, "a collapsed bar narrows the window's angle");
        assert!(shut > base, "the window still covers more than the gap");
    }

    #[test]
    fn only_a_letterboxed_2d_game_is_painted_in_the_frame_colour() {
        let frame = settings().frame;
        assert_eq!(frame_clear(true, true, false, frame), Some(frame));
        // A 3D camera fills the window: what is over the game is its sky, not letterbox. Painting
        // it in the frame colour made a collapsed header look taller than an open one.
        assert_eq!(frame_clear(false, true, false, frame), None);
        // A scene that named its own sky keeps it, 2D or 3D.
        assert_eq!(frame_clear(true, true, true, frame), None);
        // No bars, nothing to frame.
        assert_eq!(frame_clear(true, false, false, frame), None);
    }
}
