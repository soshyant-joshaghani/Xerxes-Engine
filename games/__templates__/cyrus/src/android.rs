//! Android entry (NativeActivity): built as the `android` example by cargo-apk
//! (`xerxes-ctrl game dev <game> android`).

use bevy::prelude::bevy_main;

#[bevy_main]
fn main() {
    game::run();
}
