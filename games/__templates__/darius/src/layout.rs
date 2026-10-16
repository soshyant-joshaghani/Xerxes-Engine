//! The on-screen layout shared by Darius's games: the window is one column of three rows — the
//! HUD **header** at the top, the **game** in the middle, the **controls** in the footer — and
//! the three never overlap. One camera does everything (the scene's own), like a normal 2D game.
//!
//! Each band is a node of the `HUD` overlay canvas, marked [`HeaderBand`] / [`FooterBand`] in the
//! scene; it has an **opaque** background and its children are anchored inside it with
//! `RectTransform`. The HUD is drawn full-window by the single camera. To keep the world out from
//! under the bars, this module **zooms and shifts the camera** so the game fits the gap between
//! them: it reads how tall the header and the footer are (a fraction of the window; a thin strip
//! when collapsed, nothing when the band is absent — Qanat has no footer) and scales the
//! orthographic projection by `1 / middle` and offsets it into the gap. Collapsing a band grows
//! the middle, and the game grows with it.
//!
//! A game marks its world camera with the engine's [`GameCamera2D`]. Applied on every platform.
//!
//! On Android the window draws edge-to-edge into the display cutout and under the navigation bar,
//! so the bands and the camera gap are inset by the engine's [`SafeArea`] (the device-reserved
//! edges, in logical px, `0` on web and desktop): each bar pulls in from the reserved edge so its
//! items never fall under the nav bar or the cutout. The strip the inset leaves beside a bar is not
//! a gap — the camera clears to the bars' own colour (see [`LayoutPlugin`]), so the dark HUD frame
//! still reads as reaching the screen corners. Off Android `SafeArea` is `0` and nothing changes.

use bevy::camera::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// The dark colour of the bars and the background behind the game, so the whole non-game area reads
/// as one solid frame.
const FRAME: Color = Color::srgb(0.047, 0.055, 0.071);
use xerxes_engine::modules::camera::GameCamera2D;
use xerxes_engine::modules::flow::Flow;
use xerxes_engine::modules::safe_area::SafeArea;
use xerxes_engine::modules::ui::RectTransform;

/// Height of an open header band, a fraction of the window height.
pub const HEADER: f32 = 0.20;
/// Height of an open footer (controls) band. The middle — the game — keeps the rest, so it is the
/// tallest row.
pub const FOOTER: f32 = 0.22;
/// Height of a collapsed band: a thin strip that still shows its toggle.
pub const COLLAPSED: f32 = 0.05;

/// The two bands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    Header,
    Footer,
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
        }
    }
}

/// The camera's own position, captured before this module shifts it, so the shift is always from
/// the scene's intended centre (Qanat points its camera at the room, not the origin).
#[derive(Component, Debug, Clone, Copy)]
pub struct CameraBase(pub Vec3);

/// The header band node (top row). Give it an opaque background in the scene.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HeaderBand;

/// The footer band node (bottom row). Omit it (Qanat) for a game with no controls row.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FooterBand;

/// A band's content, hidden when the band collapses (the toggle itself is not marked, so it
/// stays).
#[derive(Component, Debug, Clone, Copy)]
pub struct BandContent(pub Band);

/// A button that collapses / expands its band.
#[derive(Component, Debug, Clone, Copy)]
pub struct BandToggle(pub Band);

/// The toggle button's label: this module writes `-` (open) or `+` (collapsed) into it.
#[derive(Component, Debug, Clone, Copy)]
pub struct BandToggleLabel(pub Band);

pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        // The camera clears to the bars' colour, so the letterbox beside the game and the bars read
        // as one solid frame to the screen corners (not the engine's default grey). Set both the
        // global default and the game camera's own config (`capture_base`), so the background is the
        // frame colour from the camera's first render, whatever the shared resource is.
        app.insert_resource(ClearColor(FRAME))
            .init_resource::<Bands>()
            .add_systems(OnEnter(Flow::Level), reset_bands)
            .add_systems(
                Update,
                (toggle_bands, capture_base, apply_layout)
                    .chain()
                    .run_if(in_state(Flow::Level)),
            );
    }
}

/// The window rectangle a band fills, as anchor fractions: `[min_y, max_y]` of the height, inset
/// from the left and right edges by the safe area (`left`/`right`, fractions of the width).
fn band_rect(min_y: f32, max_y: f32, left: f32, right: f32) -> RectTransform {
    RectTransform {
        anchor_min: Vec2::new(left, min_y),
        anchor_max: Vec2::new(1.0 - right, max_y),
        pivot: Vec2::splat(0.5),
        position: Vec2::ZERO,
        size: Vec2::ZERO,
    }
}

/// A fresh level starts with both bands open.
fn reset_bands(mut bands: ResMut<Bands>) {
    *bands = Bands::default();
}

/// Remembers where each game camera started (so the shift is measured from there), and clears it to
/// the frame colour explicitly, so the background never flashes the engine's default grey.
fn capture_base(
    mut cameras: Query<(&Transform, &mut Camera, Entity), Added<GameCamera2D>>,
    mut commands: Commands,
) {
    for (transform, mut camera, entity) in &mut cameras {
        commands
            .entity(entity)
            .insert(CameraBase(transform.translation));
        camera.clear_color = ClearColorConfig::Custom(FRAME);
    }
}

/// A press on a band's toggle collapses or expands it.
fn toggle_bands(
    toggles: Query<(&Interaction, &BandToggle), Changed<Interaction>>,
    mut bands: ResMut<Bands>,
) {
    for (interaction, BandToggle(band)) in &toggles {
        if *interaction == Interaction::Pressed {
            match band {
                Band::Header => bands.header_collapsed = !bands.header_collapsed,
                Band::Footer => bands.footer_collapsed = !bands.footer_collapsed,
            }
        }
    }
}

/// Sizes the bands and fits the game into the gap between them. The header and the footer each take
/// their fraction of the height (a thin strip when collapsed, nothing when absent); the game keeps
/// the middle, zoomed to fill it and shifted into it, and is the tallest.
#[allow(clippy::type_complexity)]
fn apply_layout(
    bands: Res<Bands>,
    safe: Res<SafeArea>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut headers: Query<&mut RectTransform, (With<HeaderBand>, Without<FooterBand>)>,
    mut footers: Query<&mut RectTransform, (With<FooterBand>, Without<HeaderBand>)>,
    mut contents: Query<(&BandContent, &mut Node)>,
    mut labels: Query<(&BandToggleLabel, &mut Text)>,
    mut cameras: Query<(&mut Projection, &mut Transform, &CameraBase), With<GameCamera2D>>,
) {
    let frac = |band: Band, present: bool| {
        if !present {
            0.0
        } else if bands.collapsed(band) {
            COLLAPSED
        } else {
            match band {
                Band::Header => HEADER,
                Band::Footer => FOOTER,
            }
        }
    };
    let header = frac(Band::Header, !headers.is_empty());
    let footer = frac(Band::Footer, !footers.is_empty());

    // The safe area (device-reserved edges, logical px) as fractions of the window. On Android a
    // camera cutout or the nav bar eats into one or more edges; off Android it is all zero.
    let Ok(window) = windows.single() else {
        return;
    };
    let (win_w, win_h) = (window.width().max(1.0), window.height().max(1.0));
    // Clamp each inset: a real nav bar / cutout is a small fraction of the edge, but the content
    // rect can report a transient, outsized value while the system UI is still settling on launch;
    // clamping keeps that from briefly collapsing the game or throwing the bars far inward.
    let side = |px: f32, win: f32| (px / win).clamp(0.0, 0.3);
    let (inset_left, inset_right) = (side(safe.left, win_w), side(safe.right, win_w));
    let (inset_top, inset_bottom) = (side(safe.top, win_h), side(safe.bottom, win_h));

    // Each bar is inset from the reserved edges so its items stay clear of the nav bar / cutout.
    // The strip the inset leaves beside a bar shows the camera's clear colour, which is the bar's
    // own colour (set in `LayoutPlugin`), so the dark frame still reads as reaching the corners.
    for mut rect in &mut headers {
        let want = band_rect(
            1.0 - inset_top - header,
            1.0 - inset_top,
            inset_left,
            inset_right,
        );
        if *rect != want {
            *rect = want;
        }
    }
    for mut rect in &mut footers {
        let want = band_rect(inset_bottom, inset_bottom + footer, inset_left, inset_right);
        if *rect != want {
            *rect = want;
        }
    }
    for (BandContent(band), mut node) in &mut contents {
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

    // The middle is what the bars and the safe-area edges leave. Each band's reach is its own
    // fraction plus the inset above/below it. Zoom the camera out by 1/middle so the game fills the
    // gap, and shift it so the game sits in it (centred when top and bottom are equal; lower when
    // the top reaches further, e.g. Qanat's header-only layout).
    let top = inset_top + header;
    let bottom = inset_bottom + footer;
    let middle = (1.0 - top - bottom).clamp(0.1, 1.0);
    for (mut projection, mut transform, CameraBase(base)) in &mut cameras {
        if let Projection::Orthographic(ortho) = &mut *projection {
            let scale = 1.0 / middle;
            if (ortho.scale - scale).abs() > f32::EPSILON {
                ortho.scale = scale;
            }
            let shift = (top - bottom) / 2.0 * ortho.area.height();
            let wanted = Vec3::new(base.x, base.y + shift, base.z);
            if transform.translation != wanted {
                transform.translation = wanted;
            }
        }
    }
}
