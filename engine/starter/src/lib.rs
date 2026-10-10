//! The game, shared by every platform: `main.rs` (Windows, macOS, Linux, web) and
//! `android.rs` (the Android APK) both call [`run`]. Everything the game is made of lives in
//! `assets/` (project settings, scenes, prefabs, logic modules, art + metas).

xerxes_engine::include_assets!();

/// Starts the game: preloads its bundles, then its start scene.
pub fn run() {
    xerxes_engine::launch_project(assets::project());
}
