//! Template: pause. A global `Paused` flag the `pause` action toggles (controllers stop while
//! it is set); the `exit_to_menu` action leaves the level for the main menu. Copied into a
//! game's `assets/logic/`, it is the game's to change.

use xerxes_engine::prelude::*;

#[derive(Resource, Default, Debug, PartialEq)]
pub struct Paused(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<Paused>()
        .add_systems(Update, (toggle, exit_to_menu))
        .add_systems(OnExit(Flow::Level), unpause);
}

fn toggle(actions: Res<Actions>, mut paused: ResMut<Paused>) {
    if actions.just_pressed("pause") {
        paused.0 = !paused.0;
    }
}

fn exit_to_menu(actions: Res<Actions>, mut exit: MessageWriter<ExitToMenu>) {
    if actions.just_pressed("exit_to_menu") {
        exit.write(ExitToMenu);
    }
}

/// A level always starts unpaused.
fn unpause(mut paused: ResMut<Paused>) {
    paused.0 = false;
}
