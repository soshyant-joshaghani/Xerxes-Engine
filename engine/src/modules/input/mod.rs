//! The input manager: named **actions** that a game reads, and the **bindings** that drive them
//! from the keyboard, the mouse, a gamepad and on-screen touch controls, the way Unity's Input
//! System and Unreal's Enhanced Input do.
//!
//! The actions are the contract: gameplay asks for `"move"` or `"jump"` and never for a key. Each
//! game's `<name>.project.rs` says how each physical device drives them, so Darius and Cyrus map
//! the same engine to their own controls.
//!
//! ```ignore
//! input: InputMap::new()
//!     // A 2D value: WASD or the arrows, the left stick, the on-screen stick named "move".
//!     .vector2("move", [
//!         Binding::wasd(),
//!         Binding::arrows(),
//!         Binding::PadStick(PadStick::Left),
//!         Binding::PadDpad,
//!         Binding::TouchStick("move"),
//!     ])
//!     // A 1D value: the wheel, or the triggers.
//!     .axis("zoom", [Binding::MouseWheel { scale: 1.0 }, Binding::KeyAxis {
//!         negative: KeyCode::KeyQ, positive: KeyCode::KeyE }])
//!     // A button: a key, a mouse button, a gamepad face button, an on-screen button.
//!     .button("jump", [
//!         Binding::Key(KeyCode::Space),
//!         Binding::Pad(GamepadButton::South),
//!         Binding::TouchButton("jump"),
//!     ])
//! ```
//!
//! Gameplay:
//!
//! ```ignore
//! fn steer(actions: Res<Actions>) {
//!     let wish = actions.vector("move");        // -1..1 on each axis, y up, from any device
//!     if actions.just_pressed("jump") { /* ... */ }
//! }
//! ```
//!
//! **Touch bindings** refer to on-screen controls by id: a `VirtualJoystick::new(..).source("move")`,
//! a `TouchPad::new().source("look")` or a `TouchButton("jump")` on a button. A phone has them;
//! on a desktop they simply read zero. **Mouse, keyboard and gamepads work on phones too**: the
//! mouse through the patched winit (`engine/winit`), keys as key events, and a gamepad through
//! the key events for its buttons and the patched winit's axes ([`android`]).
//!
//! [`ActiveDevice`] tells which device was used last, for prompts that follow the player's device.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bevy::input::gamepad::{Gamepad, GamepadAxis, GamepadButton};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use crate::modules::ui::{TouchButton, TouchPad, VirtualJoystick};

pub mod android;

/// A value of at least this size counts as the action being pressed (an analog trigger, a stick
/// pushed more than half way, a touch stick used as a D-pad).
pub const PRESS_THRESHOLD: f32 = 0.5;

/// The part of a gamepad stick's travel that reads as zero.
pub const PAD_DEAD_ZONE: f32 = 0.15;

/// The physical device a binding reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Device {
    #[default]
    Keyboard,
    Mouse,
    Gamepad,
    Touch,
}

/// What an action gives gameplay (Unity's Button, and Value of Axis or Vector2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionKind {
    /// Pressed or not (`value` is 0 or 1, or the trigger's travel).
    #[default]
    Button,
    /// One axis, -1 to 1.
    Axis,
    /// Two axes, y up, each -1 to 1 (a stick, WASD).
    Vector2,
}

/// A gamepad's analog stick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PadStick {
    Left,
    Right,
}

/// What can drive an action. Each variant reads one [`Device`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Binding {
    // -- keyboard
    /// One key, as a button (or 1 on the x axis).
    Key(KeyCode),
    /// Two keys as an axis: `positive` is +1, `negative` is -1.
    KeyAxis {
        negative: KeyCode,
        positive: KeyCode,
    },
    /// Four keys as a 2D value (WASD, the arrows).
    Keys {
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
    },
    // -- mouse
    Mouse(MouseButton),
    /// How far the mouse moved this frame (x right, y up), times `scale`; not clamped to 1.
    MouseMotion {
        scale: f32,
    },
    /// The wheel's vertical scroll this frame times `scale` (on the x axis); not clamped.
    MouseWheel {
        scale: f32,
    },
    // -- gamepad (any connected one)
    Pad(GamepadButton),
    PadAxis(GamepadAxis),
    PadStick(PadStick),
    /// The D-pad as a 2D value.
    PadDpad,
    // -- touch: on-screen controls, named by their `source` / id
    TouchButton(&'static str),
    TouchStick(&'static str),
    /// Four on-screen buttons (by `TouchButton` id) as a 2D value: an on-screen D-pad.
    TouchDpad {
        up: &'static str,
        down: &'static str,
        left: &'static str,
        right: &'static str,
    },
    /// A `TouchPad`'s movement this frame times `scale` (look); not clamped.
    TouchLook {
        id: &'static str,
        scale: f32,
    },
}

impl Binding {
    /// WASD as a 2D value.
    pub fn wasd() -> Self {
        Self::Keys {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
        }
    }

    /// The arrow keys as a 2D value.
    pub fn arrows() -> Self {
        Self::Keys {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
        }
    }

    /// Mouse movement as a look value, `sensitivity` units per pixel.
    pub fn mouse_look(sensitivity: f32) -> Self {
        Self::MouseMotion { scale: sensitivity }
    }

    /// Which device this binding reads.
    pub fn device(&self) -> Device {
        match self {
            Self::Key(_) | Self::KeyAxis { .. } | Self::Keys { .. } => Device::Keyboard,
            Self::Mouse(_) | Self::MouseMotion { .. } | Self::MouseWheel { .. } => Device::Mouse,
            Self::Pad(_) | Self::PadAxis(_) | Self::PadStick(_) | Self::PadDpad => Device::Gamepad,
            Self::TouchButton(_)
            | Self::TouchStick(_)
            | Self::TouchDpad { .. }
            | Self::TouchLook { .. } => Device::Touch,
        }
    }

    /// Short text for a settings screen or an input prompt (`W A S D`, `Pad South`, `Touch move`).
    pub fn label(&self) -> String {
        let key = |k: &KeyCode| format!("{k:?}").trim_start_matches("Key").to_string();
        match self {
            Self::Key(k) => key(k),
            Self::KeyAxis { negative, positive } => format!("{} {}", key(negative), key(positive)),
            Self::Keys {
                up,
                down,
                left,
                right,
            } => format!("{} {} {} {}", key(up), key(left), key(down), key(right)),
            Self::Mouse(b) => format!("Mouse {b:?}"),
            Self::MouseMotion { .. } => "Mouse move".into(),
            Self::MouseWheel { .. } => "Mouse wheel".into(),
            Self::Pad(b) => format!("Pad {b:?}"),
            Self::PadAxis(a) => format!("Pad {a:?}"),
            Self::PadStick(s) => format!("Pad {s:?} stick"),
            Self::PadDpad => "Pad D-pad".into(),
            Self::TouchButton(id) => format!("Touch {id}"),
            Self::TouchStick(id) => format!("Touch {id} stick"),
            Self::TouchDpad { .. } => "Touch D-pad".into(),
            Self::TouchLook { id, .. } => format!("Touch {id} look"),
        }
    }
}

/// One named action: its kind and the bindings that drive it, from any device.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDef {
    pub name: &'static str,
    pub kind: ActionKind,
    pub bindings: Vec<Binding>,
}

impl ActionDef {
    /// The bindings of one device (the keyboard's, the gamepad's).
    pub fn on(&self, device: Device) -> impl Iterator<Item = &Binding> {
        self.bindings.iter().filter(move |b| b.device() == device)
    }
}

/// Named actions and their bindings, in declaration order. This is a game's whole input config.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputMap {
    pub actions: Vec<ActionDef>,
}

impl InputMap {
    pub fn new() -> Self {
        Self::default()
    }

    fn with(
        mut self,
        name: &'static str,
        kind: ActionKind,
        bindings: impl IntoIterator<Item = Binding>,
    ) -> Self {
        self.actions.retain(|a| a.name != name);
        self.actions.push(ActionDef {
            name,
            kind,
            bindings: bindings.into_iter().collect(),
        });
        self
    }

    /// Adds (or replaces) a button action. Same as [`InputMap::button`].
    pub fn action(self, name: &'static str, bindings: impl IntoIterator<Item = Binding>) -> Self {
        self.with(name, ActionKind::Button, bindings)
    }

    pub fn button(self, name: &'static str, bindings: impl IntoIterator<Item = Binding>) -> Self {
        self.with(name, ActionKind::Button, bindings)
    }

    /// A 1D value (-1 to 1).
    pub fn axis(self, name: &'static str, bindings: impl IntoIterator<Item = Binding>) -> Self {
        self.with(name, ActionKind::Axis, bindings)
    }

    /// A 2D value (a move, a look).
    pub fn vector2(self, name: &'static str, bindings: impl IntoIterator<Item = Binding>) -> Self {
        self.with(name, ActionKind::Vector2, bindings)
    }

    pub fn get(&self, name: &str) -> Option<&ActionDef> {
        self.actions.iter().find(|a| a.name == name)
    }
}

/// One action's state this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ActionState {
    /// Button: 0 to 1. Axis: -1 to 1 in `x`. Vector2: both, y up. Look-type bindings are not
    /// clamped.
    pub value: Vec2,
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

/// The device the player used last.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActiveDevice(pub Device);

/// What a set of buttons did this frame.
#[derive(Debug, Clone)]
pub struct Held<T: Eq + Hash> {
    pub down: HashSet<T>,
    pub just_pressed: HashSet<T>,
    pub just_released: HashSet<T>,
}

impl<T: Eq + Hash> Default for Held<T> {
    fn default() -> Self {
        Self {
            down: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }
}

/// A frame of raw input from every device, gathered by [`update_actions`] and read by
/// [`Actions::update`]. Plain data, so the binding rules are tested without a window.
#[derive(Debug, Clone, Default)]
pub struct RawInput {
    pub keys: Held<KeyCode>,
    pub mouse_buttons: Held<MouseButton>,
    /// Pixels since the last frame, y down (as the OS reports it).
    pub mouse_motion: Vec2,
    /// The wheel this frame, in lines (notches).
    pub wheel: f32,
    pub pad_buttons: Held<GamepadButton>,
    /// Analog buttons (triggers) 0 to 1; a button missing here reads 0 or 1 from `pad_buttons`.
    pub pad_triggers: HashMap<GamepadButton, f32>,
    /// Stick axes -1 to 1, y up (before the dead zone).
    pub pad_axes: HashMap<GamepadAxis, f32>,
    /// On-screen controls by id.
    pub touch_sticks: HashMap<&'static str, Vec2>,
    pub touch_looks: HashMap<&'static str, Vec2>,
    pub touch_buttons: HashSet<&'static str>,
    /// The device this frame's input came from, when there was any.
    pub device: Option<Device>,
}

/// Every gamepad axis the actions read.
const PAD_AXES: [GamepadAxis; 6] = [
    GamepadAxis::LeftStickX,
    GamepadAxis::LeftStickY,
    GamepadAxis::LeftZ,
    GamepadAxis::RightStickX,
    GamepadAxis::RightStickY,
    GamepadAxis::RightZ,
];

/// Every gamepad button the actions read.
const PAD_BUTTONS: [GamepadButton; 19] = [
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::North,
    GamepadButton::West,
    GamepadButton::C,
    GamepadButton::Z,
    GamepadButton::LeftTrigger,
    GamepadButton::LeftTrigger2,
    GamepadButton::RightTrigger,
    GamepadButton::RightTrigger2,
    GamepadButton::Select,
    GamepadButton::Start,
    GamepadButton::Mode,
    GamepadButton::LeftThumb,
    GamepadButton::RightThumb,
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
];

/// A radial dead zone: zero inside `zone`, then rescaled so the edge of the dead zone is 0 and
/// the full push is 1.
pub fn dead_zone(value: Vec2, zone: f32) -> Vec2 {
    let length = value.length();
    if length <= zone {
        Vec2::ZERO
    } else {
        value / length * ((length - zone) / (1.0 - zone)).min(1.0)
    }
}

/// The actions' state this frame. Read it from gameplay systems.
#[derive(Resource, Debug, Default)]
pub struct Actions {
    map: InputMap,
    states: HashMap<&'static str, ActionState>,
    device: Device,
}

impl Actions {
    pub fn new(map: InputMap) -> Self {
        let states = map
            .actions
            .iter()
            .map(|a| (a.name, ActionState::default()))
            .collect();
        Self {
            map,
            states,
            device: Device::default(),
        }
    }

    /// The config these actions follow.
    pub fn map(&self) -> &InputMap {
        &self.map
    }

    /// The whole state of an action (all zero for an action the map does not have).
    pub fn state(&self, action: &str) -> ActionState {
        self.states.get(action).copied().unwrap_or_default()
    }

    pub fn pressed(&self, action: &str) -> bool {
        self.state(action).pressed
    }

    pub fn just_pressed(&self, action: &str) -> bool {
        self.state(action).just_pressed
    }

    pub fn just_released(&self, action: &str) -> bool {
        self.state(action).just_released
    }

    /// An action's value on its first axis: 0 to 1 for a button, -1 to 1 for an axis.
    pub fn value(&self, action: &str) -> f32 {
        self.state(action).value.x
    }

    /// An action's 2D value (y up).
    pub fn vector(&self, action: &str) -> Vec2 {
        self.state(action).value
    }

    /// -1 to 1 from a pair of actions (analog when they are: a trigger pair, a stick axis).
    pub fn axis(&self, negative: &str, positive: &str) -> f32 {
        (self.value(positive) - self.value(negative)).clamp(-1.0, 1.0)
    }

    /// The device the player used last (prompts follow it).
    pub fn device(&self) -> Device {
        self.device
    }

    /// Recomputes every action from a frame of raw input.
    pub fn update(&mut self, raw: &RawInput) {
        if let Some(device) = raw.device {
            self.device = device;
        }
        let pad_stick = |stick: PadStick| {
            let (x, y) = match stick {
                PadStick::Left => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
                PadStick::Right => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
            };
            let get = |axis| raw.pad_axes.get(&axis).copied().unwrap_or(0.0);
            dead_zone(Vec2::new(get(x), get(y)), PAD_DEAD_ZONE)
        };
        let pad_value = |button: GamepadButton| {
            raw.pad_triggers
                .get(&button)
                .copied()
                .unwrap_or_else(|| raw.pad_buttons.down.contains(&button) as u8 as f32)
        };
        let key = |k: &KeyCode| raw.keys.down.contains(k) as u8 as f32;

        for def in &self.map.actions {
            // Bounded sources add up and are clamped to a unit; motion-like ones (mouse, wheel,
            // touch look) are deltas and are added after.
            let mut bounded = Vec2::ZERO;
            let mut free = Vec2::ZERO;
            let mut digital_pressed = false;
            let mut digital_released = false;
            for binding in &def.bindings {
                match *binding {
                    Binding::Key(k) => {
                        bounded.x += key(&k);
                        digital_pressed |= raw.keys.just_pressed.contains(&k);
                        digital_released |= raw.keys.just_released.contains(&k);
                    }
                    Binding::KeyAxis { negative, positive } => {
                        bounded.x += key(&positive) - key(&negative);
                        for k in [negative, positive] {
                            digital_pressed |= raw.keys.just_pressed.contains(&k);
                            digital_released |= raw.keys.just_released.contains(&k);
                        }
                    }
                    Binding::Keys {
                        up,
                        down,
                        left,
                        right,
                    } => {
                        bounded += Vec2::new(key(&right) - key(&left), key(&up) - key(&down))
                            .normalize_or_zero();
                        for k in [up, down, left, right] {
                            digital_pressed |= raw.keys.just_pressed.contains(&k);
                            digital_released |= raw.keys.just_released.contains(&k);
                        }
                    }
                    Binding::Mouse(b) => {
                        bounded.x += raw.mouse_buttons.down.contains(&b) as u8 as f32;
                        digital_pressed |= raw.mouse_buttons.just_pressed.contains(&b);
                        digital_released |= raw.mouse_buttons.just_released.contains(&b);
                    }
                    Binding::MouseMotion { scale } => {
                        free += Vec2::new(raw.mouse_motion.x, -raw.mouse_motion.y) * scale;
                    }
                    Binding::MouseWheel { scale } => free.x += raw.wheel * scale,
                    Binding::Pad(b) => {
                        bounded.x += pad_value(b);
                        digital_pressed |= raw.pad_buttons.just_pressed.contains(&b);
                        digital_released |= raw.pad_buttons.just_released.contains(&b);
                    }
                    Binding::PadAxis(axis) => {
                        let v = raw.pad_axes.get(&axis).copied().unwrap_or(0.0);
                        bounded.x += dead_zone(Vec2::new(v, 0.0), PAD_DEAD_ZONE).x;
                    }
                    Binding::PadStick(stick) => bounded += pad_stick(stick),
                    Binding::PadDpad => {
                        let down = |b| raw.pad_buttons.down.contains(&b) as u8 as f32;
                        bounded += Vec2::new(
                            down(GamepadButton::DPadRight) - down(GamepadButton::DPadLeft),
                            down(GamepadButton::DPadUp) - down(GamepadButton::DPadDown),
                        );
                    }
                    Binding::TouchButton(id) => {
                        bounded.x += raw.touch_buttons.contains(id) as u8 as f32;
                    }
                    Binding::TouchStick(id) => {
                        bounded += raw.touch_sticks.get(id).copied().unwrap_or(Vec2::ZERO);
                    }
                    Binding::TouchDpad {
                        up,
                        down,
                        left,
                        right,
                    } => {
                        let held = |id| raw.touch_buttons.contains(id) as u8 as f32;
                        bounded += Vec2::new(held(right) - held(left), held(up) - held(down))
                            .normalize_or_zero();
                    }
                    Binding::TouchLook { id, scale } => {
                        free += raw.touch_looks.get(id).copied().unwrap_or(Vec2::ZERO) * scale;
                    }
                }
            }
            let sum = bounded.clamp_length_max(1.0) + free;
            let value = match def.kind {
                ActionKind::Button => Vec2::new(sum.length().min(1.0), 0.0),
                ActionKind::Axis => Vec2::new(sum.x, 0.0),
                ActionKind::Vector2 => sum,
            };
            let pressed = value.length() >= PRESS_THRESHOLD;
            let was = self.states.get(def.name).is_some_and(|s| s.pressed);
            self.states.insert(
                def.name,
                ActionState {
                    value,
                    pressed,
                    just_pressed: digital_pressed || (pressed && !was),
                    just_released: !pressed && (was || digital_released),
                },
            );
        }
    }
}

/// Adds [`Actions`] (from the game's [`InputMap`]) and [`ActiveDevice`], and updates them every
/// frame after Bevy's input and the touch controls have been read.
pub struct ActionsPlugin(pub InputMap);

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Actions::new(self.0.clone()))
            .init_resource::<ActiveDevice>()
            .add_systems(
                PreUpdate,
                update_actions
                    .after(bevy::input::InputSystems)
                    .after(bevy::ui::UiSystems::Focus)
                    .after(crate::modules::ui::TouchInputSet),
            );
    }
}

/// Reads every device into [`RawInput`] and updates [`Actions`].
#[allow(clippy::too_many_arguments)]
pub fn update_actions(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Option<Res<AccumulatedMouseMotion>>,
    scroll: Option<Res<AccumulatedMouseScroll>>,
    pads: Query<&Gamepad>,
    touches: Option<Res<Touches>>,
    sticks: Query<&VirtualJoystick>,
    looks: Query<&TouchPad>,
    buttons: Query<(&Interaction, &TouchButton)>,
    mut actions: ResMut<Actions>,
    mut active: ResMut<ActiveDevice>,
) {
    let mut raw = RawInput::default();

    // Keyboard. A gamepad's buttons also arrive as key events on Android; they are read below.
    let real = |k: &&KeyCode| !matches!(k, KeyCode::Unidentified(_));
    raw.keys.down = keys.get_pressed().filter(real).copied().collect();
    raw.keys.just_pressed = keys.get_just_pressed().filter(real).copied().collect();
    raw.keys.just_released = keys.get_just_released().filter(real).copied().collect();

    // Mouse.
    raw.mouse_buttons.down = mouse.get_pressed().copied().collect();
    raw.mouse_buttons.just_pressed = mouse.get_just_pressed().copied().collect();
    raw.mouse_buttons.just_released = mouse.get_just_released().copied().collect();
    raw.mouse_motion = motion.map_or(Vec2::ZERO, |m| m.delta);
    raw.wheel = scroll.map_or(0.0, |s| match s.unit {
        MouseScrollUnit::Line => s.delta.y,
        MouseScrollUnit::Pixel => s.delta.y * 0.01,
    });

    // Gamepads: every connected one at once, the strongest axis and any button wins.
    for pad in &pads {
        for button in PAD_BUTTONS {
            if pad.pressed(button) {
                raw.pad_buttons.down.insert(button);
            }
            if pad.just_pressed(button) {
                raw.pad_buttons.just_pressed.insert(button);
            }
            if pad.just_released(button) {
                raw.pad_buttons.just_released.insert(button);
            }
            if let Some(value) = pad.get(button) {
                let entry = raw.pad_triggers.entry(button).or_insert(0.0);
                *entry = entry.max(value);
            }
        }
        for axis in PAD_AXES {
            if let Some(value) = pad.get(axis) {
                let entry = raw.pad_axes.entry(axis).or_insert(0.0);
                if value.abs() > entry.abs() {
                    *entry = value;
                }
            }
        }
    }
    android::merge(&mut raw, &keys, android::axes());

    // Touch controls.
    for stick in &sticks {
        if !stick.source.is_empty() {
            let entry = raw.touch_sticks.entry(stick.source).or_insert(Vec2::ZERO);
            if stick.value.length() > entry.length() {
                *entry = stick.value;
            }
        }
    }
    for pad in &looks {
        if !pad.source.is_empty() {
            *raw.touch_looks.entry(pad.source).or_insert(Vec2::ZERO) += pad.delta;
        }
    }
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            raw.touch_buttons.insert(button.0);
        }
    }

    // The device used this frame: the last of these that did something.
    let mut device = None;
    if !raw.keys.just_pressed.is_empty() || !raw.keys.down.is_empty() {
        device = Some(Device::Keyboard);
    }
    if !raw.mouse_buttons.just_pressed.is_empty()
        || raw.mouse_motion != Vec2::ZERO
        || raw.wheel != 0.0
    {
        device = Some(Device::Mouse);
    }
    if !raw.pad_buttons.down.is_empty()
        || raw.pad_axes.values().any(|v| v.abs() > PAD_DEAD_ZONE + 0.1)
    {
        device = Some(Device::Gamepad);
    }
    if touches.is_some_and(|t| t.iter().next().is_some()) {
        device = Some(Device::Touch);
    }
    raw.device = device;
    if let Some(device) = device {
        if active.0 != device {
            active.0 = device;
        }
    }
    actions.update(&raw);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> InputMap {
        InputMap::new()
            .vector2(
                "move",
                [
                    Binding::wasd(),
                    Binding::arrows(),
                    Binding::PadStick(PadStick::Left),
                    Binding::PadDpad,
                    Binding::TouchStick("move"),
                    Binding::TouchDpad {
                        up: "dpad_up",
                        down: "dpad_down",
                        left: "dpad_left",
                        right: "dpad_right",
                    },
                ],
            )
            .button(
                "jump",
                [
                    Binding::Key(KeyCode::Space),
                    Binding::Pad(GamepadButton::South),
                    Binding::TouchButton("jump"),
                    Binding::Mouse(MouseButton::Left),
                ],
            )
            .axis(
                "zoom",
                [
                    Binding::MouseWheel { scale: 1.0 },
                    Binding::KeyAxis {
                        negative: KeyCode::KeyQ,
                        positive: KeyCode::KeyE,
                    },
                ],
            )
            .vector2(
                "look",
                [
                    Binding::mouse_look(0.5),
                    Binding::TouchLook {
                        id: "look",
                        scale: 2.0,
                    },
                ],
            )
            // The old way of writing a button still works.
            .action("pause", [Binding::Key(KeyCode::Escape)])
    }

    fn frame(actions: &mut Actions, build: impl FnOnce(&mut RawInput)) {
        let mut raw = RawInput::default();
        build(&mut raw);
        actions.update(&raw);
    }

    #[test]
    fn one_action_is_driven_by_every_device() {
        let mut a = Actions::new(map());

        frame(&mut a, |r| {
            r.keys.down.insert(KeyCode::KeyD);
        });
        assert_eq!(a.vector("move"), Vec2::X);

        frame(&mut a, |r| {
            r.pad_axes.insert(GamepadAxis::LeftStickY, -1.0);
        });
        assert_eq!(a.vector("move"), Vec2::NEG_Y);

        frame(&mut a, |r| {
            r.touch_sticks.insert("move", Vec2::new(0.0, 0.6));
        });
        assert!((a.vector("move") - Vec2::new(0.0, 0.6)).length() < 1e-5);

        // An on-screen D-pad: left and up held together is a unit diagonal.
        frame(&mut a, |r| {
            r.touch_buttons.insert("dpad_left");
            r.touch_buttons.insert("dpad_up");
        });
        let diagonal = Vec2::new(-1.0, 1.0).normalize();
        assert!((a.vector("move") - diagonal).length() < 1e-5);

        frame(&mut a, |r| {
            r.pad_buttons.down.insert(GamepadButton::DPadLeft);
        });
        assert_eq!(a.vector("move"), Vec2::NEG_X);
    }

    #[test]
    fn a_vector_is_clamped_so_two_devices_do_not_run_faster() {
        let mut a = Actions::new(map());
        frame(&mut a, |r| {
            r.keys.down.insert(KeyCode::KeyD);
            r.keys.down.insert(KeyCode::KeyW);
            r.touch_sticks.insert("move", Vec2::new(1.0, 1.0));
        });
        assert!(a.vector("move").length() <= 1.0 + 1e-5);
        assert!(a.pressed("move"));
    }

    #[test]
    fn the_dead_zone_is_removed_and_the_rest_rescaled() {
        let mut a = Actions::new(map());
        frame(&mut a, |r| {
            r.pad_axes.insert(GamepadAxis::LeftStickX, 0.1);
        });
        assert_eq!(a.vector("move"), Vec2::ZERO);
        frame(&mut a, |r| {
            r.pad_axes.insert(GamepadAxis::LeftStickX, 1.0);
        });
        assert!((a.vector("move").x - 1.0).abs() < 1e-5);
        frame(&mut a, |r| {
            r.pad_axes.insert(GamepadAxis::LeftStickX, 0.575);
        });
        assert!(
            (a.vector("move").x - 0.5).abs() < 1e-3,
            "{:?}",
            a.vector("move")
        );
    }

    #[test]
    fn buttons_press_and_release_across_devices() {
        let mut a = Actions::new(map());
        frame(&mut a, |r| {
            r.keys.down.insert(KeyCode::Space);
            r.keys.just_pressed.insert(KeyCode::Space);
        });
        assert!(a.pressed("jump") && a.just_pressed("jump") && !a.just_released("jump"));

        // Held on, no new edge.
        frame(&mut a, |r| {
            r.keys.down.insert(KeyCode::Space);
        });
        assert!(a.pressed("jump") && !a.just_pressed("jump"));

        // The key goes up while the gamepad button goes down: still held, pressed again.
        frame(&mut a, |r| {
            r.keys.just_released.insert(KeyCode::Space);
            r.pad_buttons.down.insert(GamepadButton::South);
            r.pad_buttons.just_pressed.insert(GamepadButton::South);
        });
        assert!(a.pressed("jump") && a.just_pressed("jump") && !a.just_released("jump"));

        frame(&mut a, |r| {
            r.pad_buttons.just_released.insert(GamepadButton::South);
        });
        assert!(!a.pressed("jump") && a.just_released("jump"));

        // An on-screen button works as a transition (it has no edge of its own).
        frame(&mut a, |r| {
            r.touch_buttons.insert("jump");
        });
        assert!(a.just_pressed("jump"));
        frame(&mut a, |_| {});
        assert!(a.just_released("jump"));
    }

    #[test]
    fn a_tap_shorter_than_a_frame_is_still_seen() {
        let mut a = Actions::new(map());
        frame(&mut a, |r| {
            r.keys.just_pressed.insert(KeyCode::Space);
            r.keys.just_released.insert(KeyCode::Space);
        });
        assert!(a.just_released("jump"));
    }

    #[test]
    fn a_trigger_is_analog_and_presses_past_half() {
        let m = InputMap::new().button("fire", [Binding::Pad(GamepadButton::RightTrigger2)]);
        let mut a = Actions::new(m);
        frame(&mut a, |r| {
            r.pad_triggers.insert(GamepadButton::RightTrigger2, 0.3);
        });
        assert!((a.value("fire") - 0.3).abs() < 1e-6 && !a.pressed("fire"));
        frame(&mut a, |r| {
            r.pad_triggers.insert(GamepadButton::RightTrigger2, 0.8);
        });
        assert!(a.pressed("fire"));
    }

    #[test]
    fn mouse_and_touch_look_are_deltas_not_clamped() {
        let mut a = Actions::new(map());
        frame(&mut a, |r| {
            r.mouse_motion = Vec2::new(10.0, 4.0);
        });
        // Mouse y is down on screen, y up in the action.
        assert_eq!(a.vector("look"), Vec2::new(5.0, -2.0));
        frame(&mut a, |r| {
            r.touch_looks.insert("look", Vec2::new(3.0, 1.0));
        });
        assert_eq!(a.vector("look"), Vec2::new(6.0, 2.0));
    }

    #[test]
    fn an_axis_combines_keys_and_the_wheel_and_axis_reads_a_pair() {
        let mut a = Actions::new(map());
        frame(&mut a, |r| {
            r.keys.down.insert(KeyCode::KeyQ);
        });
        assert_eq!(a.value("zoom"), -1.0);
        frame(&mut a, |r| {
            r.wheel = 3.0;
        });
        assert_eq!(a.value("zoom"), 3.0, "the wheel is a delta, not clamped");

        let m = InputMap::new()
            .button("left", [Binding::Key(KeyCode::KeyA)])
            .button("right", [Binding::Key(KeyCode::KeyD)]);
        let mut b = Actions::new(m);
        frame(&mut b, |r| {
            r.keys.down.insert(KeyCode::KeyA);
        });
        assert_eq!(b.axis("left", "right"), -1.0);
    }

    #[test]
    fn unknown_actions_read_zero_and_the_old_builder_is_a_button() {
        let a = Actions::new(map());
        assert!(!a.pressed("nope") && a.vector("nope") == Vec2::ZERO);
        assert_eq!(a.map().get("pause").unwrap().kind, ActionKind::Button);
    }

    #[test]
    fn bindings_know_their_device_and_a_device_can_be_listed() {
        let map = map();
        let jump = map.get("jump").unwrap();
        assert_eq!(jump.on(Device::Keyboard).count(), 1);
        assert_eq!(jump.on(Device::Gamepad).count(), 1);
        assert_eq!(jump.on(Device::Touch).count(), 1);
        assert_eq!(jump.on(Device::Mouse).count(), 1);
        assert_eq!(Binding::wasd().device(), Device::Keyboard);
        assert_eq!(Binding::wasd().label(), "W A S D");
    }

    #[test]
    fn the_active_device_follows_the_last_input() {
        let mut a = Actions::new(map());
        assert_eq!(a.device(), Device::Keyboard);
        frame(&mut a, |r| r.device = Some(Device::Gamepad));
        assert_eq!(a.device(), Device::Gamepad);
        // A quiet frame keeps it.
        frame(&mut a, |_| {});
        assert_eq!(a.device(), Device::Gamepad);
    }

    #[test]
    fn re_declaring_an_action_replaces_it() {
        let m = InputMap::new()
            .button("a", [Binding::Key(KeyCode::KeyA)])
            .axis("a", [Binding::Key(KeyCode::KeyB)]);
        assert_eq!(m.actions.len(), 1);
        assert_eq!(m.actions[0].kind, ActionKind::Axis);
    }
}
