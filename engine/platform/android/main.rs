//! The Xerxes engine app on Android (NativeActivity). Built as the `android` example:
//! `cargo apk build --example android --features android` (`xerxes-ctrl engine run android` / `engine publish android`).

use bevy::prelude::bevy_main;

#[bevy_main]
fn main() {
    xerxes_engine::editor::launch();
}
