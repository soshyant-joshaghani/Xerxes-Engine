//! The game, shared by every platform: `main.rs` (Windows, macOS, Linux, web) and
//! `android.rs` (the Android APK) both call [`run`]. The project settings, scenes and logic
//! modules live in `assets/`; the top-down rules in `top_down` are plugged in by
//! `assets/logic/top_down.rs`.

pub mod top_down;

xerxes_engine::include_assets!();

/// Starts the game: preloads its bundles, then the menu and its levels.
pub fn run() {
    xerxes_engine::launch_project(assets::project());
}
