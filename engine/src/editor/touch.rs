//! Whether the editor uses touch sizes (bigger targets for a finger): `auto` decides from the
//! device (Android; the web when the main pointer is coarse; elsewhere the first finger that
//! touches the screen), `on` and `off` are the user's choice (Preferences). The flag itself is
//! `layout::touch()`, one for the whole editor.

use super::layout::{set_touch, set_touch_auto};
use super::prefs;

/// The choices, and the one the user has made (`auto` until they choose).
pub const MODES: [(&str, &str); 3] = [("auto", "Auto"), ("on", "On"), ("off", "Off")];

pub fn mode() -> String {
    prefs::load("touch")
        .map(|m| m.trim().to_string())
        .filter(|m| MODES.iter().any(|(id, _)| id == m))
        .unwrap_or_else(|| "auto".to_string())
}

/// Whether the device looks like a touch device (the `auto` choice's starting point).
pub fn detect() -> bool {
    #[cfg(target_os = "android")]
    {
        true
    }
    #[cfg(all(target_arch = "wasm32", not(target_os = "android")))]
    {
        web_sys::window()
            .and_then(|w| w.match_media("(pointer: coarse)").ok().flatten())
            .is_some_and(|query| query.matches())
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        // Dev/QA hook: `XERXES_TOUCH=1` starts a desktop editor with touch sizes.
        std::env::var_os("XERXES_TOUCH").is_some()
    }
}

/// Puts a choice in effect (and keeps it).
pub fn apply(mode: &str, save: bool) {
    match mode {
        "on" => {
            set_touch_auto(false);
            set_touch(true);
        }
        "off" => {
            set_touch_auto(false);
            set_touch(false);
        }
        _ => {
            set_touch_auto(true);
            set_touch(detect());
        }
    }
    if save {
        prefs::save("touch", mode);
    }
}

/// At start: the user's choice.
pub fn init() {
    apply(&mode(), false);
}
