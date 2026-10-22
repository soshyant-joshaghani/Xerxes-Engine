use futures_executor::block_on;
use futures_util::StreamExt;
use xerxes_engine::editor::bridge;

#[derive(Debug, Clone, PartialEq)]
enum Cmd {
    Pause,
    Resume,
}

#[test]
fn commands_drain_in_order_and_only_once() {
    let (ui, runtime) = bridge::<Cmd, u32>(0);
    ui.send(Cmd::Pause);
    ui.clone().send(Cmd::Resume);

    assert_eq!(runtime.drain_commands(), vec![Cmd::Pause, Cmd::Resume]);
    assert!(runtime.drain_commands().is_empty());
}

#[test]
fn latest_starts_at_initial_and_follows_publish() {
    let (ui, runtime) = bridge::<Cmd, u32>(7);
    assert_eq!(ui.latest(), 7);

    assert!(runtime.publish(8));
    assert_eq!(ui.latest(), 8);
}

#[test]
fn unchanged_snapshot_is_not_pushed() {
    let (ui, runtime) = bridge::<Cmd, u32>(1);
    let mut updates = ui.subscribe();

    assert!(!runtime.publish(1));
    assert!(runtime.publish(2));
    assert!(!runtime.publish(2));
    assert!(runtime.publish(3));
    drop(runtime);

    // Every RuntimePort is gone, but the UiPort keeps the senders alive: read exactly two.
    let got = block_on(async { vec![updates.next().await, updates.next().await] });
    assert_eq!(got, vec![Some(2), Some(3)]);
    assert!(updates.try_recv().is_err(), "no third update queued");
}

#[test]
fn dropped_subscriber_is_pruned_without_affecting_others() {
    let (ui, runtime) = bridge::<Cmd, u32>(0);
    let dropped = ui.subscribe();
    let mut kept = ui.subscribe();
    drop(dropped);

    assert!(runtime.publish(5));
    assert_eq!(block_on(kept.next()), Some(5));
}

#[test]
fn ports_compare_by_bridge_identity() {
    let (a, _) = bridge::<Cmd, u32>(0);
    let (b, _) = bridge::<Cmd, u32>(0);
    assert!(a == a.clone());
    assert!(a != b);
}

#[test]
fn runtime_port_crosses_threads() {
    let (ui, runtime) = bridge::<Cmd, u32>(0);
    ui.send(Cmd::Pause);
    let drained = std::thread::spawn(move || {
        runtime.publish(9);
        runtime.drain_commands()
    })
    .join()
    .unwrap();

    assert_eq!(drained, vec![Cmd::Pause]);
    assert_eq!(ui.latest(), 9);
}
