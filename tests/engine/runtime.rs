//! Headless: a bare `App` runs the schedules without a window or renderer.

use bevy::prelude::*;
use xerxes_engine::editor::bridge;
use xerxes_engine::editor::{UiBridge, UiBridgePlugin, UiCommand};

#[derive(Debug, Clone, PartialEq)]
enum Cmd {
    Add(u32),
}

#[derive(Resource, Default)]
struct Total(u32);

fn apply(mut commands: MessageReader<UiCommand<Cmd>>, mut total: ResMut<Total>) {
    for UiCommand(Cmd::Add(n)) in commands.read() {
        total.0 += n;
    }
}

fn publish(bridge: Res<UiBridge<Cmd, u32>>, total: Res<Total>) {
    bridge.publish(total.0);
}

fn app() -> (xerxes_engine::editor::UiPort<Cmd, u32>, App) {
    let (ui, runtime) = bridge::<Cmd, u32>(0);
    let mut app = App::new();
    app.add_plugins(UiBridgePlugin::new(runtime))
        .init_resource::<Total>()
        .add_systems(Update, (apply, publish).chain());
    (ui, app)
}

#[test]
fn ui_commands_reach_systems_and_snapshots_come_back() {
    let (ui, mut app) = app();
    ui.send(Cmd::Add(2));
    ui.send(Cmd::Add(3));
    app.update();

    assert_eq!(app.world().resource::<Total>().0, 5);
    assert_eq!(ui.latest(), 5);
}

#[test]
fn each_command_is_delivered_once() {
    let (ui, mut app) = app();
    ui.send(Cmd::Add(1));
    app.update();
    app.update();
    app.update();

    assert_eq!(ui.latest(), 1);
}

#[test]
fn primary_window_targets_the_host_canvas() {
    let plugin = xerxes_engine::modules::runtime::primary_window("Game", Some("#game".into()));
    let window = plugin.primary_window.expect("primary window");
    assert_eq!(window.title, "Game");
    assert_eq!(window.canvas.as_deref(), Some("#game"));
    assert!(window.fit_canvas_to_parent);
}
