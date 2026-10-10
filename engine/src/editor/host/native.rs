//! The editor on Windows, macOS, Linux and Android: Bevy owns the only window (one event
//! loop, which is all NativeActivity allows) and Dioxus draws the UI over it.
//!
//! A [`DioxusDocument`] (Dioxus native's DOM, laid out and styled by Blitz) holds the root
//! component. Every frame the overlay forwards the window's mouse and touch input to it and
//! polls the VirtualDom; when something changed, Blitz paints the document on the CPU
//! (vello_cpu) into an image that a full-window Bevy UI node shows above the world.
//!
//! ```text
//! Bevy window ─┬─ world (cameras, meshes)
//!              └─ ImageNode ◀── vello_cpu ◀── Blitz layout ◀── DioxusDocument(Root(UiPort))
//!     input ─────────────────────────────────────────────────▶ (mouse, touch)
//! ```

use std::sync::Arc;

use anyrender::ImageRenderer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use bevy::asset::RenderAssetUsages;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{CursorMoved, PrimaryWindow};
use blitz_dom::{Document, DocumentConfig};
use blitz_paint::paint_scene;
use blitz_traits::events::{
    BlitzKeyEvent, BlitzMouseButtonEvent, KeyState, MouseEventButton, MouseEventButtons, UiEvent,
};
use blitz_traits::navigation::{NavigationOptions, NavigationProvider};
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::VirtualDom;
use dioxus_native_dom::DioxusDocument;

/// Builds the VirtualDom for the overlay (it is `!Send`, so it is made on the main thread).
pub type MakeDom = Arc<dyn Fn() -> VirtualDom + Send + Sync>;

/// Wheel lines to CSS pixels, like Blitz's own shell.
const LINE_HEIGHT: f64 = 20.0;

pub struct DioxusOverlayPlugin {
    make_dom: MakeDom,
}

impl DioxusOverlayPlugin {
    pub fn new(make_dom: MakeDom) -> Self {
        Self { make_dom }
    }
}

impl Plugin for DioxusOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverlayDom(self.make_dom.clone()))
            .add_systems(Startup, setup_overlay)
            .add_systems(Update, drive_overlay);
        #[cfg(target_os = "android")]
        app.add_systems(Update, soft_keyboard.after(drive_overlay));
        #[cfg(any(target_os = "android", target_os = "ios"))]
        app.add_systems(PreUpdate, phone_density);
        #[cfg(windows)]
        app.add_systems(Update, window_icon);
    }
}

#[derive(Resource)]
struct OverlayDom(MakeDom);

/// The Dioxus document and its paint target. Main thread only (the VirtualDom is `!Send`).
struct Overlay {
    doc: DioxusDocument,
    renderer: VelloCpuImageRenderer,
    pixels: Vec<u8>,
    image: Handle<Image>,
    /// Physical size and scale the document is laid out for.
    size: UVec2,
    scale: f32,
    /// Pointer position in logical pixels, and the buttons held.
    pointer: Vec2,
    buttons: MouseEventButtons,
    /// The touch acting as the mouse (Android).
    touch: Option<u64>,
    dirty: bool,
}

fn setup_overlay(world: &mut World) {
    let make_dom = world.resource::<OverlayDom>().0.clone();
    let mut doc = DioxusDocument::new(
        make_dom(),
        DocumentConfig {
            navigation_provider: Some(Arc::new(OpenInBrowser)),
            ..Default::default()
        },
    );
    doc.initial_build();

    let image = world
        .resource_mut::<Assets<Image>>()
        .add(blank_image(UVec2::ONE));
    // The overlay gets its own full-window UI camera, drawn after the world without clearing
    // it: the world's cameras may render into part of the window only (a viewport).
    world.spawn((
        Camera2d,
        Camera {
            order: 100,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        // UI only: keep world gizmos (drawn on layer 0) out of this 2D camera.
        bevy::camera::visibility::RenderLayers::layer(31),
    ));
    world.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        ImageNode::new(image.clone()),
        // Above any game or stage UI; the world still gets the pointer.
        GlobalZIndex(i32::MAX),
        Pickable::IGNORE,
    ));

    world.insert_non_send_resource(Overlay {
        doc,
        renderer: VelloCpuImageRenderer::new(1, 1),
        pixels: Vec::new(),
        image,
        size: UVec2::ZERO,
        scale: 0.0,
        pointer: Vec2::ZERO,
        buttons: MouseEventButtons::None,
        touch: None,
        dirty: true,
    });
}

fn blank_image(size: UVec2) -> Image {
    Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Windows: the window and taskbar show the exe's icon (resource 1, `windows/xerxes.rc`).
/// Runs until the window exists, then sets it once.
#[cfg(windows)]
fn window_icon(mut done: Local<bool>, window: Single<Entity, With<PrimaryWindow>>) {
    use winit::platform::windows::IconExtWindows;
    if *done {
        return;
    }
    bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        if let Some(winit_window) = windows.get_window(*window) {
            match winit::window::Icon::from_resource(1, None) {
                Ok(icon) => winit_window.set_window_icon(Some(icon)),
                Err(err) => bevy::log::warn!("window icon: {err}"),
            }
            *done = true;
        }
    });
}

/// Phones: at the system's scale a landscape phone is only ~850 logical pixels wide, too
/// narrow for an editor. Below `PHONE_WIDTH` the editor uses a denser scale (never below 1x)
/// so the window is that wide; the overlay and the Scene view follow the window's scale.
#[cfg(any(target_os = "android", target_os = "ios"))]
fn phone_density(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    const PHONE_WIDTH: f32 = 1100.0;
    let width = window.physical_width() as f32;
    if width <= 0.0 {
        return;
    }
    let base = window.resolution.base_scale_factor();
    let wanted = (width / base < PHONE_WIDTH).then(|| (width / PHONE_WIDTH).max(1.0));
    if window.resolution.scale_factor_override() != wanted {
        window.resolution.set_scale_factor_override(wanted);
    }
}

/// Android: the soft keyboard opens while a UI input has focus (winit shows it for
/// `ime_enabled`); physical keyboards elsewhere need nothing.
#[cfg(target_os = "android")]
fn soft_keyboard(focus: Res<super::UiFocus>, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    if focus.is_changed() && window.ime_enabled != focus.0 {
        window.ime_enabled = focus.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_overlay(
    mut overlay: NonSendMut<Overlay>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor: MessageReader<CursorMoved>,
    mut mouse: MessageReader<MouseButtonInput>,
    mut wheel: MessageReader<MouseWheel>,
    mut touches: MessageReader<TouchInput>,
    mut keys: MessageReader<KeyboardInput>,
    held: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<super::UiFocus>,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
) {
    let overlay = &mut *overlay;
    overlay.fit(
        UVec2::new(window.physical_width(), window.physical_height()),
        window.scale_factor(),
    );
    if overlay.size.min_element() == 0 {
        return;
    }

    let hovered = overlay.doc.get_hover_node_id();
    for moved in cursor.read() {
        overlay.move_to(moved.position);
    }
    for input in mouse.read() {
        let button = match input.button {
            MouseButton::Left => MouseEventButton::Main,
            MouseButton::Right => MouseEventButton::Secondary,
            _ => continue,
        };
        overlay.press(button, input.state == ButtonState::Pressed);
    }
    for scroll in wheel.read() {
        let unit = match scroll.unit {
            MouseScrollUnit::Line => LINE_HEIGHT,
            MouseScrollUnit::Pixel => 1.0,
        };
        overlay.scroll(scroll.x as f64 * unit, scroll.y as f64 * unit);
    }
    for touch in touches.read() {
        overlay.touch(touch);
    }
    // Keys go to the UI only while a text input has focus (then the Scene view ignores them);
    // otherwise they belong to the Scene view. Blitz also focuses clicked buttons and rows,
    // which must not take W/E/R away from the Scene view.
    let focused = overlay.text_input_focused();
    if focus.0 != focused {
        focus.0 = focused;
    }
    for key in keys.read() {
        if focused {
            overlay.key(key, &held);
        }
    }
    // Hover only changes the picture when the pointer crosses into another element.
    if overlay.doc.get_hover_node_id() != hovered {
        overlay.dirty = true;
    }

    // Run whatever the UI scheduled (bridge snapshots, clicks, finished fetches).
    for _ in 0..8 {
        if !overlay.doc.poll(None) {
            break;
        }
        overlay.dirty = true;
    }

    if overlay.dirty || overlay.doc.is_animating() {
        overlay.paint(time.elapsed_secs_f64());
        if let Some(image) = images.get_mut(&overlay.image) {
            let size = Extent3d {
                width: overlay.size.x,
                height: overlay.size.y,
                depth_or_array_layers: 1,
            };
            if image.texture_descriptor.size != size {
                image.resize(size);
            }
            image.data = Some(overlay.pixels.clone());
        }
    }
}

impl Overlay {
    /// Lays the document out for the window's current physical size and scale.
    fn fit(&mut self, size: UVec2, scale: f32) {
        if (size == self.size && scale == self.scale) || size.min_element() == 0 {
            return;
        }
        self.size = size;
        self.scale = scale;
        self.doc
            .set_viewport(Viewport::new(size.x, size.y, scale, ColorScheme::Dark));
        self.renderer.resize(size.x, size.y);
        self.dirty = true;
    }

    fn mouse_event(&self, button: MouseEventButton) -> BlitzMouseButtonEvent {
        BlitzMouseButtonEvent {
            x: self.pointer.x,
            y: self.pointer.y,
            button,
            buttons: self.buttons,
            mods: Default::default(),
        }
    }

    /// `position` is in logical pixels, like Blitz's CSS pixels.
    fn move_to(&mut self, position: Vec2) {
        self.pointer = position;
        let event = self.mouse_event(MouseEventButton::Main);
        self.doc.handle_ui_event(UiEvent::MouseMove(event));
    }

    fn press(&mut self, button: MouseEventButton, pressed: bool) {
        if pressed {
            self.buttons.insert(button.into());
        } else {
            self.buttons.remove(button.into());
        }
        let event = self.mouse_event(button);
        self.doc.handle_ui_event(if pressed {
            UiEvent::MouseDown(event)
        } else {
            UiEvent::MouseUp(event)
        });
        self.dirty = true;
    }

    fn scroll(&mut self, x: f64, y: f64) {
        let changed = match self.doc.get_hover_node_id() {
            Some(node) => self.doc.scroll_node_by_has_changed(node, x, y),
            None => self.doc.scroll_viewport_by_has_changed(x, y),
        };
        self.dirty |= changed;
    }

    fn key(&mut self, input: &KeyboardInput, held: &ButtonInput<KeyCode>) {
        use keyboard_types::{Code, Location, Modifiers};
        let key = match &input.logical_key {
            Key::Character(c) => keyboard_types::Key::Character(c.to_string()),
            Key::Space => keyboard_types::Key::Character(" ".into()),
            Key::Enter => keyboard_types::Key::Enter,
            Key::Tab => keyboard_types::Key::Tab,
            Key::Backspace => keyboard_types::Key::Backspace,
            Key::Delete => keyboard_types::Key::Delete,
            Key::Escape => keyboard_types::Key::Escape,
            Key::ArrowLeft => keyboard_types::Key::ArrowLeft,
            Key::ArrowRight => keyboard_types::Key::ArrowRight,
            Key::ArrowUp => keyboard_types::Key::ArrowUp,
            Key::ArrowDown => keyboard_types::Key::ArrowDown,
            Key::Home => keyboard_types::Key::Home,
            Key::End => keyboard_types::Key::End,
            _ => keyboard_types::Key::Unidentified,
        };
        // Bevy's KeyCode names are the W3C code names keyboard-types parses.
        let code = format!("{:?}", input.key_code)
            .parse::<Code>()
            .unwrap_or(Code::Unidentified);
        let mut modifiers = Modifiers::empty();
        let any = |keys: [KeyCode; 2]| held.any_pressed(keys);
        if any([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            modifiers.insert(Modifiers::SHIFT);
        }
        if any([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            modifiers.insert(Modifiers::CONTROL);
        }
        if any([KeyCode::AltLeft, KeyCode::AltRight]) {
            modifiers.insert(Modifiers::ALT);
        }
        if any([KeyCode::SuperLeft, KeyCode::SuperRight]) {
            modifiers.insert(Modifiers::META);
        }
        let pressed = input.state == ButtonState::Pressed;
        let event = BlitzKeyEvent {
            key,
            code,
            modifiers,
            location: Location::Standard,
            is_auto_repeating: input.repeat,
            is_composing: false,
            state: if pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            },
            text: if pressed { input.text.clone() } else { None },
        };
        self.doc.handle_ui_event(if pressed {
            UiEvent::KeyDown(event)
        } else {
            UiEvent::KeyUp(event)
        });
        self.dirty = true;
    }

    /// Whether the focused element takes typing (`<input>`, `<textarea>`).
    fn text_input_focused(&self) -> bool {
        self.doc
            .get_focussed_node_id()
            .and_then(|id| self.doc.get_node(id))
            .and_then(|node| node.element_data())
            .is_some_and(|el| matches!(el.name.local.as_ref(), "input" | "textarea"))
    }

    /// The first finger acts as the left mouse button (Android); other fingers are ignored.
    fn touch(&mut self, touch: &TouchInput) {
        match touch.phase {
            TouchPhase::Started if self.touch.is_none() => {
                self.touch = Some(touch.id);
                self.move_to(touch.position);
                self.press(MouseEventButton::Main, true);
            }
            TouchPhase::Moved if self.touch == Some(touch.id) => {
                // A drag scrolls what is under the finger, like the browser does.
                let delta = touch.position - self.pointer;
                self.move_to(touch.position);
                self.scroll(delta.x as f64, delta.y as f64);
            }
            TouchPhase::Ended | TouchPhase::Canceled if self.touch == Some(touch.id) => {
                self.touch = None;
                self.move_to(touch.position);
                self.press(MouseEventButton::Main, false);
            }
            _ => {}
        }
    }

    /// Resolves styles and layout and paints the document into `pixels` (straight-alpha sRGB).
    fn paint(&mut self, time: f64) {
        self.dirty = false;
        self.doc.resolve(time);
        let Self {
            doc,
            renderer,
            pixels,
            size,
            scale,
            ..
        } = self;
        renderer.render_to_vec(
            |scene| paint_scene(scene, doc, *scale as f64, size.x, size.y),
            pixels,
        );
        unpremultiply(pixels);
    }
}

/// vello_cpu writes premultiplied alpha; Bevy UI blends straight alpha.
fn unpremultiply(pixels: &mut [u8]) {
    for px in pixels.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a != 0 && a != 255 {
            for c in &mut px[..3] {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
}

/// Links (a game's Play link) open in the system browser.
struct OpenInBrowser;

impl NavigationProvider for OpenInBrowser {
    fn navigate_to(&self, options: NavigationOptions) {
        let url = options.url.to_string();
        #[cfg(target_os = "windows")]
        let opened = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn();
        #[cfg(target_os = "macos")]
        let opened = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
        let opened = std::process::Command::new("xdg-open").arg(&url).spawn();
        #[cfg(target_os = "android")]
        let opened: std::io::Result<()> = Err(std::io::Error::other(
            "no browser intent from NativeActivity yet",
        ));
        if let Err(err) = opened {
            warn!("could not open {url}: {err}");
        }
    }
}
