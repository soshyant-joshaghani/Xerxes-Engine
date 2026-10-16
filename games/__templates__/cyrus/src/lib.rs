//! The game, shared by every platform: `main.rs` (Windows, macOS, Linux, web) and
//! `android.rs` (the Android APK) both call [`run`]. The project settings, scenes and logic
//! modules live in `assets/`; each playable game's code is a module here
//! (`pairidaeza`, `arashs_arrow`), plugged in by its `src/logic.rs`.

pub mod arashs_arrow;
pub mod logic;
pub mod pairidaeza;

xerxes_engine::include_assets!();

/// Starts the game: preloads its bundles, then the menu and its levels.
pub fn run() {
    xerxes_engine::launch_project(assets::project());
}
