//! The game, shared by every platform: `main.rs` (Windows, macOS, Linux, web) and
//! `android.rs` (the Android APK) both call [`run`]. The project settings, scenes and logic
//! modules live in `assets/`; each playable game's code is a module here
//! (`chapar`, `ahrimans_horde`, `jamshids_workshop`, `qanat`), plugged in by its
//! `src/logic.rs`.

pub mod chapar;
pub mod ahrimans_horde;
pub mod jamshids_workshop;
pub mod layout;
pub mod logic;
pub mod qanat;

xerxes_engine::include_assets!();

/// Starts the game: preloads its bundles, then the menu and its levels.
pub fn run() {
    xerxes_engine::launch_project(assets::project());
}
