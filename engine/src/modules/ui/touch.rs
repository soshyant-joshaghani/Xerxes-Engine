//! Mobile inputs: a [`VirtualJoystick`] and a [`TouchPad`], UI elements you put on a canvas.
//!
//! ```ignore
//! // The lower-left of the screen is a joystick: it appears where the thumb lands.
//! Object::new("Move Stick")
//!     .with(VirtualJoystick::new(80.0).dead_zone(0.12))
//!     .with(RectTransform::anchored(Anchor::BottomLeft, Vec2::new(0.0, 0.0), Vec2::new(420.0, 420.0)))
//! // The right half takes swipes: aim, look.
//! Object::new("Look Pad")
//!     .with(TouchPad::new())
//!     .with(RectTransform::anchored(Anchor::BottomRight, Vec2::new(0.0, 0.0), Vec2::new(420.0, 420.0)))
//! ```
//!
//! The element's rectangle is the zone (the big box). A [`VirtualJoystick`] is *floating*: it
//! starts where the first touch lands inside the zone, the knob follows the finger up to
//! `radius`, and the output is analog (`value`, -1 to 1, y up). A [`TouchPad`] reports what a
//! finger does inside its zone: how far it moved this frame, and the direction from where it
//! landed. Several elements work at once with several fingers (move with one thumb, aim with
//! the other).
//!
//! **Through actions.** Give an element a `source` (`VirtualJoystick::new(80.0).source("move")`,
//! `TouchPad::new().source("look")`, or a [`TouchButton`]) and bind it in the game's `InputMap`
//! (`Binding::TouchStick("move")`, `Binding::TouchLook { id: "look", .. }`,
//! `Binding::TouchButton("jump")`): gameplay then reads one action (`actions.vector("move")`)
//! whichever device drives it. The components can still be read directly
//! (`Query<&VirtualJoystick>`). Touches are used on a phone; `with_mouse()` lets the left or
//! right mouse button drive an element too, for trying it on a desktop. Zones and the knob are
//! only drawn once a touch has been seen ([`TouchSeen`]), so a desktop player sees nothing.

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

/// Becomes true when the first touch happens: the zones and the stick's ring are drawn from
/// then on. The editor sets it so the zones show in the Scene view.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TouchSeen(pub bool);

/// The system set that reads the fingers into every stick and pad; actions are updated after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TouchInputSet;

/// An on-screen button: while its UI node is pressed (by a finger or the mouse), the action
/// bindings `Binding::TouchButton(id)` of the same id are held. Put it on a `Button`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[require(Interaction)]
pub struct TouchButton(pub &'static str);

/// The pointer that drives an element: a touch id, or a mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pointer {
    Touch(u64),
    MouseLeft,
    MouseRight,
}

/// An analog stick that floats to where the touch started.
#[derive(Component, Debug, Clone)]
#[require(Node = zone_node(), BackgroundColor = zone_color())]
pub struct VirtualJoystick {
    /// The farthest the knob goes from where the touch began, in pixels.
    pub radius: f32,
    /// The fraction of `radius` (0 to 1) inside which the output is zero.
    pub dead_zone: f32,
    /// The stick, -1 to 1 on each axis, y up. Zero when not held.
    pub value: Vec2,
    /// A finger is on it.
    pub active: bool,
    /// The id actions bind it by (`Binding::TouchStick(id)`). Empty: not bound to any action.
    pub source: &'static str,
    mouse: bool,
    pointer: Option<Pointer>,
    /// Where the touch began and where it is now, in the zone's pixels (y down).
    origin: Vec2,
    current: Vec2,
}

impl VirtualJoystick {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            dead_zone: 0.1,
            value: Vec2::ZERO,
            active: false,
            source: "",
            mouse: false,
            pointer: None,
            origin: Vec2::ZERO,
            current: Vec2::ZERO,
        }
    }

    pub fn dead_zone(mut self, fraction: f32) -> Self {
        self.dead_zone = fraction.clamp(0.0, 0.95);
        self
    }

    /// Lets the left **or** right mouse button drive it (desktop). Right-drag is useful when
    /// fire is bound to RMB: aim with the stick without also using left-click to move.
    pub fn with_mouse(mut self) -> Self {
        self.mouse = true;
        self
    }

    /// The id actions bind this stick by (`Binding::TouchStick("move")`).
    pub fn source(mut self, id: &'static str) -> Self {
        self.source = id;
        self
    }
}

/// A zone that reports what a finger does inside it: a swipe direction or a look delta.
#[derive(Component, Debug, Clone)]
#[require(Node = zone_node(), BackgroundColor = zone_color())]
pub struct TouchPad {
    /// How far the finger moved this frame, in pixels (y up).
    pub delta: Vec2,
    /// The unit direction from where the finger landed to where it is, once it has moved a
    /// few pixels; zero otherwise.
    pub direction: Vec2,
    /// Where the finger landed to where it is, in pixels (y up).
    pub offset: Vec2,
    /// A finger is on it.
    pub held: bool,
    /// The finger landed / lifted this frame.
    pub just_pressed: bool,
    pub just_released: bool,
    /// The id actions bind it by (`Binding::TouchLook { id, .. }`). Empty: not bound.
    pub source: &'static str,
    mouse: bool,
    pointer: Option<Pointer>,
    origin: Vec2,
    last: Vec2,
}

impl TouchPad {
    pub fn new() -> Self {
        Self {
            delta: Vec2::ZERO,
            direction: Vec2::ZERO,
            offset: Vec2::ZERO,
            held: false,
            just_pressed: false,
            just_released: false,
            source: "",
            mouse: false,
            pointer: None,
            origin: Vec2::ZERO,
            last: Vec2::ZERO,
        }
    }

    pub fn with_mouse(mut self) -> Self {
        self.mouse = true;
        self
    }

    /// The id actions bind this pad by (`Binding::TouchLook { id: "look", .. }`).
    pub fn source(mut self, id: &'static str) -> Self {
        self.source = id;
        self
    }
}

impl Default for TouchPad {
    fn default() -> Self {
        Self::new()
    }
}

fn zone_color() -> BackgroundColor {
    BackgroundColor(Color::NONE)
}

fn zone_node() -> Node {
    Node {
        width: px(360),
        height: px(360),
        ..default()
    }
}

/// The parts a stick draws (children of its zone).
#[derive(Component)]
pub(super) struct StickBase;

#[derive(Component)]
pub(super) struct StickKnob;

/// A finger's state this frame.
struct Finger {
    pointer: Pointer,
    /// Logical window pixels, y down.
    at: Vec2,
    pressed_now: bool,
}

fn fingers(
    touches: Option<&Touches>,
    mouse: Option<&ButtonInput<MouseButton>>,
    window: Option<&Window>,
) -> Vec<Finger> {
    let mut out = Vec::new();
    if let Some(touches) = touches {
        for touch in touches.iter() {
            out.push(Finger {
                pointer: Pointer::Touch(touch.id()),
                at: touch.position(),
                pressed_now: touches.just_pressed(touch.id()),
            });
        }
    }
    if let (Some(mouse), Some(window)) = (mouse, window) {
        if let Some(at) = window.cursor_position() {
            if mouse.pressed(MouseButton::Left) {
                out.push(Finger {
                    pointer: Pointer::MouseLeft,
                    at,
                    pressed_now: mouse.just_pressed(MouseButton::Left),
                });
            }
            if mouse.pressed(MouseButton::Right) {
                out.push(Finger {
                    pointer: Pointer::MouseRight,
                    at,
                    pressed_now: mouse.just_pressed(MouseButton::Right),
                });
            }
        }
    }
    out
}

/// A zone in logical window pixels: its top-left and its size.
fn zone_rect(computed: &ComputedNode, at: &UiGlobalTransform, scale: f32) -> (Vec2, Vec2) {
    let size = computed.size() / scale;
    (at.translation / scale - size / 2.0, size)
}

/// Reads the fingers into every stick and pad. A finger that lands in a zone belongs to it
/// until it lifts; one finger drives one element.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive(
    touches: Option<Res<Touches>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut seen: ResMut<TouchSeen>,
    mut sticks: Query<(
        Entity,
        &mut VirtualJoystick,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut pads: Query<(Entity, &mut TouchPad, &ComputedNode, &UiGlobalTransform)>,
) {
    let window = windows.single().ok();
    let fingers = fingers(touches.as_deref(), mouse.as_deref(), window);
    if touches.as_ref().is_some_and(|t| t.iter().next().is_some()) && !seen.0 {
        seen.0 = true;
    }
    let scale = window.map_or(1.0, |w| w.scale_factor()).max(0.01);
    let find = |pointer: Pointer| fingers.iter().find(|f| f.pointer == pointer);
    // Fingers already used this frame (by an element earlier in the loops).
    let mut claimed: Vec<Pointer> = Vec::new();

    for (_, mut stick, computed, at) in &mut sticks {
        let (top_left, size) = zone_rect(computed, at, scale);
        // Held: follow the finger, or let go.
        if let Some(pointer) = stick.pointer {
            match find(pointer) {
                Some(finger) => {
                    claimed.push(pointer);
                    stick.current = finger.at - top_left;
                }
                None => {
                    stick.pointer = None;
                    stick.active = false;
                }
            }
        }
        // Free: a finger that just landed inside takes it.
        if stick.pointer.is_none() {
            let landed = fingers.iter().find(|f| {
                f.pressed_now
                    && !claimed.contains(&f.pointer)
                    && (matches!(f.pointer, Pointer::Touch(_)) || stick.mouse)
                    && Rect::from_corners(top_left, top_left + size).contains(f.at)
            });
            if let Some(finger) = landed {
                claimed.push(finger.pointer);
                stick.pointer = Some(finger.pointer);
                stick.active = true;
                stick.origin = finger.at - top_left;
                stick.current = stick.origin;
            }
        }
        // The analog value: the offset over the radius, past the dead zone.
        let offset = if stick.active {
            stick.current - stick.origin
        } else {
            Vec2::ZERO
        };
        let radius = stick.radius.max(1.0);
        let length = (offset.length() / radius).min(1.0);
        let value = if length <= stick.dead_zone {
            Vec2::ZERO
        } else {
            let scaled = (length - stick.dead_zone) / (1.0 - stick.dead_zone);
            Vec2::new(offset.x, -offset.y).normalize_or_zero() * scaled
        };
        if stick.value != value {
            stick.value = value;
        }
    }

    for (_, mut pad, computed, at) in &mut pads {
        let (top_left, size) = zone_rect(computed, at, scale);
        pad.just_pressed = false;
        pad.just_released = false;
        pad.delta = Vec2::ZERO;
        if let Some(pointer) = pad.pointer {
            match find(pointer) {
                Some(finger) => {
                    claimed.push(pointer);
                    let now = finger.at - top_left;
                    let step = now - pad.last;
                    pad.delta = Vec2::new(step.x, -step.y);
                    pad.last = now;
                    let moved = now - pad.origin;
                    pad.offset = Vec2::new(moved.x, -moved.y);
                    pad.direction = if moved.length() >= 8.0 {
                        pad.offset.normalize_or_zero()
                    } else {
                        Vec2::ZERO
                    };
                }
                None => {
                    pad.pointer = None;
                    pad.held = false;
                    pad.just_released = true;
                    pad.direction = Vec2::ZERO;
                    pad.offset = Vec2::ZERO;
                }
            }
        }
        if pad.pointer.is_none() && !pad.just_released {
            let landed = fingers.iter().find(|f| {
                f.pressed_now
                    && !claimed.contains(&f.pointer)
                    && (matches!(f.pointer, Pointer::Touch(_)) || pad.mouse)
                    && Rect::from_corners(top_left, top_left + size).contains(f.at)
            });
            if let Some(finger) = landed {
                claimed.push(finger.pointer);
                pad.pointer = Some(finger.pointer);
                pad.held = true;
                pad.just_pressed = true;
                pad.origin = finger.at - top_left;
                pad.last = pad.origin;
            }
        }
    }
}

/// A stick draws a ring where the touch began and a knob where the finger is. Both are
/// children of the zone, hidden until a touch has been seen.
pub(super) fn build_sticks(
    mut commands: Commands,
    added: Query<(Entity, &VirtualJoystick), Added<VirtualJoystick>>,
) {
    for (entity, stick) in &added {
        let diameter = stick.radius * 2.0;
        commands.spawn((
            StickBase,
            Node {
                position_type: PositionType::Absolute,
                width: px(diameter),
                height: px(diameter),
                left: px(0),
                top: px(0),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::MAX,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.012)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
            ChildOf(entity),
        ));
        let knob = stick.radius * 0.8;
        commands.spawn((
            StickKnob,
            Node {
                position_type: PositionType::Absolute,
                width: px(knob),
                height: px(knob),
                left: px(0),
                top: px(0),
                border_radius: BorderRadius::MAX,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            ChildOf(entity),
        ));
    }
}

/// Places the ring and the knob; at rest (and once a touch has been seen) the ring sits at the
/// zone's centre as a hint.
pub(super) fn show_sticks(
    seen: Res<TouchSeen>,
    sticks: Query<(&VirtualJoystick, &Children, &ComputedNode)>,
    mut parts: Query<(&mut Node, Has<StickKnob>), Or<(With<StickBase>, With<StickKnob>)>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let scale = windows.single().map_or(1.0, |w| w.scale_factor()).max(0.01);
    for (stick, children, computed) in &sticks {
        let size = computed.size() / scale;
        let radius = stick.radius;
        let (base_at, knob_at, show_base, show_knob) = if stick.active {
            let offset = (stick.current - stick.origin).clamp_length_max(radius);
            (stick.origin, stick.origin + offset, true, true)
        } else {
            // At rest the ring is a hint in the zone's lower-left corner, where a thumb sits. It
            // shows once a touch has been seen, or always for a `with_mouse` stick (so a desktop
            // player can see where to click and drag it).
            let rest =
                Vec2::new(radius + 28.0, size.y - radius - 28.0).min(size / 2.0 + size / 4.0);
            (rest, rest, seen.0 || stick.mouse, false)
        };
        for child in children.iter() {
            let Ok((mut node, is_knob)) = parts.get_mut(child) else {
                continue;
            };
            let (at, show, half) = if is_knob {
                (knob_at, show_knob, radius * 0.4)
            } else {
                (base_at, show_base, radius)
            };
            let display = if show { Display::Flex } else { Display::None };
            if node.display != display {
                node.display = display;
            }
            let (left, top) = (px(at.x - half), px(at.y - half));
            if node.left != left {
                node.left = left;
            }
            if node.top != top {
                node.top = top;
            }
        }
    }
}

/// Zones are drawn faintly once a touch has been seen (a hair-thin frame, and almost no
/// fill), so the player sees where to put a thumb without the zone covering the game. (UI
/// blends in linear space: a 1% white fill already lifts a dark background by about ten
/// levels, so these alphas are small on purpose.)
pub(super) fn tint_zones(
    seen: Res<TouchSeen>,
    sticks: Query<&VirtualJoystick>,
    mut zones: Query<
        (&mut BackgroundColor, &mut Node, Option<&mut BorderColor>),
        Or<(With<VirtualJoystick>, With<TouchPad>)>,
    >,
    mut commands: Commands,
    entities: Query<
        Entity,
        (
            Or<(With<VirtualJoystick>, With<TouchPad>)>,
            Without<BorderColor>,
        ),
    >,
) {
    // Drawn once a touch has been seen, or always when a `with_mouse` stick is present (so a
    // desktop player sees the zone to click in).
    let show = seen.0 || sticks.iter().any(|stick| stick.mouse);
    let (fill, frame, width) = if show {
        (
            Color::srgba(1.0, 1.0, 1.0, 0.003),
            Color::srgba(1.0, 1.0, 1.0, 0.06),
            px(1),
        )
    } else {
        (Color::NONE, Color::NONE, px(0))
    };
    for entity in &entities {
        commands.entity(entity).insert(BorderColor::all(frame));
    }
    for (mut background, mut node, border) in &mut zones {
        if background.0 != fill {
            background.0 = fill;
        }
        let edge = UiRect::all(width);
        if node.border != edge {
            node.border = edge;
        }
        if let Some(mut border) = border {
            let wanted = BorderColor::all(frame);
            if *border != wanted {
                *border = wanted;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::touch::{TouchInput, TouchPhase};
    use bevy::math::Affine2;

    /// An app with the input plugin and the touch driver, and a 400 x 400 zone at the window's
    /// top-left corner holding `bundle`.
    fn app_with(bundle: impl Bundle) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin)
            .init_resource::<TouchSeen>()
            .add_systems(PreUpdate, drive.after(bevy::input::InputSystems));
        let window = app.world_mut().spawn_empty().id();
        let zone = app
            .world_mut()
            .spawn((
                bundle,
                ComputedNode {
                    size: Vec2::new(400.0, 400.0),
                    ..default()
                },
                UiGlobalTransform::from(Affine2::from_translation(Vec2::new(200.0, 200.0))),
            ))
            .id();
        (app, window, zone)
    }

    fn touch(app: &mut App, window: Entity, id: u64, phase: TouchPhase, x: f32, y: f32) {
        app.world_mut().write_message(TouchInput {
            phase,
            position: Vec2::new(x, y),
            window,
            force: None,
            id,
        });
        app.update();
    }

    #[test]
    fn a_joystick_floats_to_the_first_touch_and_is_analog() {
        let (mut app, window, zone) = app_with(VirtualJoystick::new(100.0).dead_zone(0.0));
        // Lands inside the zone, away from its centre: that is the stick's origin.
        touch(&mut app, window, 1, TouchPhase::Started, 100.0, 300.0);
        let stick = app.world().get::<VirtualJoystick>(zone).unwrap();
        assert!(stick.active);
        assert_eq!(stick.value, Vec2::ZERO, "no push yet");
        assert!(
            app.world().resource::<TouchSeen>().0,
            "the first touch is seen"
        );

        // Half a radius to the right: half the stick.
        touch(&mut app, window, 1, TouchPhase::Moved, 150.0, 300.0);
        let stick = app.world().get::<VirtualJoystick>(zone).unwrap();
        assert!(
            (stick.value - Vec2::new(0.5, 0.0)).length() < 1e-4,
            "{:?}",
            stick.value
        );

        // Far past the radius, upwards: clamped to 1, y is up.
        touch(&mut app, window, 1, TouchPhase::Moved, 100.0, 50.0);
        let stick = app.world().get::<VirtualJoystick>(zone).unwrap();
        assert!(
            (stick.value - Vec2::new(0.0, 1.0)).length() < 1e-4,
            "{:?}",
            stick.value
        );

        // Lifting lets go.
        touch(&mut app, window, 1, TouchPhase::Ended, 100.0, 50.0);
        let stick = app.world().get::<VirtualJoystick>(zone).unwrap();
        assert!(!stick.active);
        assert_eq!(stick.value, Vec2::ZERO);
    }

    #[test]
    fn a_touch_outside_the_zone_does_nothing_and_the_dead_zone_holds_the_stick() {
        let (mut app, window, zone) = app_with(VirtualJoystick::new(100.0).dead_zone(0.3));
        touch(&mut app, window, 1, TouchPhase::Started, 700.0, 300.0);
        assert!(!app.world().get::<VirtualJoystick>(zone).unwrap().active);
        touch(&mut app, window, 1, TouchPhase::Ended, 700.0, 300.0);

        touch(&mut app, window, 2, TouchPhase::Started, 100.0, 100.0);
        touch(&mut app, window, 2, TouchPhase::Moved, 120.0, 100.0);
        let stick = app.world().get::<VirtualJoystick>(zone).unwrap();
        assert!(
            stick.active && stick.value == Vec2::ZERO,
            "inside the dead zone"
        );
    }

    #[test]
    fn a_touch_pad_reports_deltas_and_the_direction_from_where_the_finger_landed() {
        let (mut app, window, zone) = app_with(TouchPad::new());
        touch(&mut app, window, 7, TouchPhase::Started, 200.0, 200.0);
        let pad = app.world().get::<TouchPad>(zone).unwrap();
        assert!(pad.held && pad.just_pressed);
        touch(&mut app, window, 7, TouchPhase::Moved, 230.0, 200.0);
        let pad = app.world().get::<TouchPad>(zone).unwrap();
        assert_eq!(pad.delta, Vec2::new(30.0, 0.0));
        assert_eq!(pad.direction, Vec2::X);
        // Up on the screen is y up.
        touch(&mut app, window, 7, TouchPhase::Moved, 230.0, 170.0);
        let pad = app.world().get::<TouchPad>(zone).unwrap();
        assert_eq!(pad.delta, Vec2::new(0.0, 30.0));
        touch(&mut app, window, 7, TouchPhase::Ended, 230.0, 170.0);
        let pad = app.world().get::<TouchPad>(zone).unwrap();
        assert!(!pad.held && pad.just_released);
        assert_eq!(pad.direction, Vec2::ZERO);
    }

    #[test]
    fn two_fingers_drive_two_elements() {
        let mut app = App::new();
        app.add_plugins(bevy::input::InputPlugin)
            .init_resource::<TouchSeen>()
            .add_systems(PreUpdate, drive.after(bevy::input::InputSystems));
        let window = app.world_mut().spawn_empty().id();
        let zone = |app: &mut App, x: f32| {
            app.world_mut()
                .spawn((
                    ComputedNode {
                        size: Vec2::new(400.0, 400.0),
                        ..default()
                    },
                    UiGlobalTransform::from(Affine2::from_translation(Vec2::new(x, 200.0))),
                ))
                .id()
        };
        let left = zone(&mut app, 200.0);
        let right = zone(&mut app, 800.0);
        app.world_mut()
            .entity_mut(left)
            .insert(VirtualJoystick::new(100.0));
        app.world_mut().entity_mut(right).insert(TouchPad::new());
        touch(&mut app, window, 1, TouchPhase::Started, 100.0, 200.0);
        touch(&mut app, window, 2, TouchPhase::Started, 700.0, 200.0);
        assert!(app.world().get::<VirtualJoystick>(left).unwrap().active);
        assert!(app.world().get::<TouchPad>(right).unwrap().held);
    }

    #[test]
    fn a_stick_value_is_analog_past_the_dead_zone() {
        // Same math as `drive`: offset over radius, dead zone removed and rescaled.
        let stick = VirtualJoystick::new(80.0).dead_zone(0.25);
        let length = (40.0f32 / stick.radius).min(1.0);
        assert_eq!(length, 0.5);
        let scaled = (length - stick.dead_zone) / (1.0 - stick.dead_zone);
        assert!((scaled - 1.0 / 3.0).abs() < 1e-6);
        assert!(stick.value == Vec2::ZERO && !stick.active);
    }

    #[test]
    fn dead_zone_is_clamped() {
        assert_eq!(VirtualJoystick::new(10.0).dead_zone(5.0).dead_zone, 0.95);
        assert_eq!(VirtualJoystick::new(10.0).dead_zone(-1.0).dead_zone, 0.0);
    }
}
