//! The game, shared by every platform: `main.rs` (Windows, macOS, Linux, web) and
//! `android.rs` (the Android APK) both call [`run`]. Everything the game is made of lives in
//! `assets/` (project settings, scenes, prefabs, art + metas), and its logic plugins are listed in
//! the project settings' `LOGIC`.

xerxes_engine::include_assets!();

/// Starts the game: preloads its bundles, then its start scene.
pub fn run() {
    xerxes_engine::launch_project(assets::project());
}
