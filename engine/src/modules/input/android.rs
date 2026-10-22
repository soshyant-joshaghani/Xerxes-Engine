//! Gamepads on Android. Bevy's gamepad backend (gilrs) has none there, so a controller reaches
//! the engine two other ways, and [`merge`] folds both into the same [`RawInput`] the other
//! platforms fill, so one action map covers every device on every platform:
//!
//! - its **buttons** arrive as key events with no key name (`KeyCode::Unidentified(Android(96))`
//!   is the A button), which [`button`] names;
//! - its **sticks, triggers and hat** are motion events that stock winit would report as a phantom
//!   touch; the vendored winit (`engine/vendors/winit`, patch marked "Xerxes patch") keeps them and hands
//!   them out as [`axes`].
//!
//! The D-pad also arrives as arrow keys, so it works as the keyboard's arrows too.

use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy::input::keyboard::{KeyCode, NativeKeyCode};
use bevy::prelude::*;

use super::RawInput;

/// The latest gamepad axes: `[left X, left Y, right X, right Y, left trigger, right trigger,
/// hat X, hat Y]`, in Android's convention (y down). All zero off Android.
pub fn axes() -> [f32; 8] {
    #[cfg(target_os = "android")]
    {
        winit::platform::android::gamepad_axes()
    }
    #[cfg(not(target_os = "android"))]
    {
        [0.0; 8]
    }
}

/// The gamepad button an Android key code is (`AKEYCODE_BUTTON_A` = 96 and so on).
pub fn button(code: u32) -> Option<GamepadButton> {
    Some(match code {
        96 => GamepadButton::South,
        97 => GamepadButton::East,
        98 => GamepadButton::C,
        99 => GamepadButton::West,
        100 => GamepadButton::North,
        101 => GamepadButton::Z,
        102 => GamepadButton::LeftTrigger,
        103 => GamepadButton::RightTrigger,
        104 => GamepadButton::LeftTrigger2,
        105 => GamepadButton::RightTrigger2,
        106 => GamepadButton::LeftThumb,
        107 => GamepadButton::RightThumb,
        108 => GamepadButton::Start,
        109 => GamepadButton::Select,
        110 => GamepadButton::Mode,
        _ => return None,
    })
}

/// Adds Android's gamepad buttons (from the key events) and `axes` (from [`axes`]) to `raw`.
/// Does nothing when there is none, so it is safe on every platform.
pub fn merge(raw: &mut RawInput, keys: &ButtonInput<KeyCode>, axes: [f32; 8]) {
    let named = |key: &KeyCode| match key {
        KeyCode::Unidentified(NativeKeyCode::Android(code)) => button(*code),
        _ => None,
    };
    raw.pad_buttons
        .down
        .extend(keys.get_pressed().filter_map(named));
    raw.pad_buttons
        .just_pressed
        .extend(keys.get_just_pressed().filter_map(named));
    raw.pad_buttons
        .just_released
        .extend(keys.get_just_released().filter_map(named));

    let [x, y, z, rz, left, right, hat_x, hat_y] = axes;
    let mut stronger = |axis: GamepadAxis, value: f32| {
        if value != 0.0 {
            let entry = raw.pad_axes.entry(axis).or_insert(0.0);
            if value.abs() > entry.abs() {
                *entry = value;
            }
        }
    };
    // Android's y is down; actions are y up.
    stronger(GamepadAxis::LeftStickX, x);
    stronger(GamepadAxis::LeftStickY, -y);
    stronger(GamepadAxis::RightStickX, z);
    stronger(GamepadAxis::RightStickY, -rz);
    for (button, value) in [
        (GamepadButton::LeftTrigger2, left),
        (GamepadButton::RightTrigger2, right),
    ] {
        if value > 0.0 {
            let entry = raw.pad_triggers.entry(button).or_insert(0.0);
            *entry = entry.max(value);
        }
    }
    for (pressed, button) in [
        (hat_x > 0.5, GamepadButton::DPadRight),
        (hat_x < -0.5, GamepadButton::DPadLeft),
        (hat_y < -0.5, GamepadButton::DPadUp),
        (hat_y > 0.5, GamepadButton::DPadDown),
    ] {
        if pressed {
            raw.pad_buttons.down.insert(button);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_key_codes_name_the_gamepad_buttons() {
        assert_eq!(button(96), Some(GamepadButton::South));
        assert_eq!(button(100), Some(GamepadButton::North));
        assert_eq!(button(108), Some(GamepadButton::Start));
        assert_eq!(button(29), None, "a letter key is not a gamepad button");
    }

    #[test]
    fn key_events_and_axes_become_the_same_raw_input_as_other_platforms() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Unidentified(NativeKeyCode::Android(96)));
        keys.press(KeyCode::KeyW);
        let mut raw = RawInput::default();
        // Left stick pushed right and up (Android y is -1 up), right trigger half, hat left.
        merge(&mut raw, &keys, [0.8, -0.6, 0.0, 0.0, 0.0, 0.5, -1.0, 0.0]);

        assert!(raw.pad_buttons.down.contains(&GamepadButton::South));
        assert!(raw.pad_buttons.just_pressed.contains(&GamepadButton::South));
        assert!(raw.pad_buttons.down.contains(&GamepadButton::DPadLeft));
        assert_eq!(raw.pad_axes[&GamepadAxis::LeftStickX], 0.8);
        assert_eq!(raw.pad_axes[&GamepadAxis::LeftStickY], 0.6);
        assert_eq!(raw.pad_triggers[&GamepadButton::RightTrigger2], 0.5);
        assert!(
            raw.keys.down.is_empty(),
            "a keyboard key is left to the keyboard"
        );
    }
}
