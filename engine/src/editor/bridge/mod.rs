//! UI ↔ runtime bridge.
//!
//! The UI never touches the game world. It sends commands (`C`) and observes
//! snapshots (`S`); the runtime drains commands once per frame and publishes a
//! snapshot when something the UI shows has changed.
//!
//! ```text
//! UiPort ──commands──▶ RuntimePort      (queue, drained by the runtime each frame)
//! UiPort ◀─snapshots── RuntimePort      (latest value + push to subscribers)
//! ```
//!
//! The two ports are separate types so each side can only move data in its own
//! direction. Both are cheap to clone and `Send + Sync` when `C` and `S` are, so the
//! runtime port can live in an ECS resource on any thread.

pub mod protocol;
pub mod view;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

/// Creates a connected pair of ports, starting from `initial` as the snapshot.
pub fn bridge<C, S: Clone>(initial: S) -> (UiPort<C, S>, RuntimePort<C, S>) {
    let shared = Arc::new(Shared {
        commands: Mutex::new(VecDeque::new()),
        snapshot: Mutex::new(Snapshot {
            latest: initial,
            subscribers: Vec::new(),
        }),
    });
    (
        UiPort {
            shared: shared.clone(),
        },
        RuntimePort { shared },
    )
}

struct Shared<C, S> {
    commands: Mutex<VecDeque<C>>,
    snapshot: Mutex<Snapshot<S>>,
}

struct Snapshot<S> {
    latest: S,
    subscribers: Vec<UnboundedSender<S>>,
}

/// A poisoned lock only means another thread panicked mid-push; the data is still a
/// valid queue or snapshot, so keep going instead of cascading the panic into the UI.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The UI side: send commands, read or subscribe to snapshots.
pub struct UiPort<C, S> {
    shared: Arc<Shared<C, S>>,
}

impl<C, S> Clone for UiPort<C, S> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<C, S> PartialEq for UiPort<C, S> {
    /// Ports are equal when they belong to the same bridge (lets UI frameworks skip re-renders).
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl<C, S: Clone> UiPort<C, S> {
    /// Queues a command for the runtime's next frame.
    pub fn send(&self, command: C) {
        lock(&self.shared.commands).push_back(command);
    }

    /// The most recently published snapshot.
    pub fn latest(&self) -> S {
        lock(&self.shared.snapshot).latest.clone()
    }

    /// A stream of every snapshot published from now on. Dropping it unsubscribes.
    pub fn subscribe(&self) -> UnboundedReceiver<S> {
        let (tx, rx) = unbounded();
        lock(&self.shared.snapshot).subscribers.push(tx);
        rx
    }
}

/// The runtime side: drain commands, publish snapshots.
pub struct RuntimePort<C, S> {
    shared: Arc<Shared<C, S>>,
}

impl<C, S> Clone for RuntimePort<C, S> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<C, S: Clone + PartialEq> RuntimePort<C, S> {
    /// Takes every queued command, oldest first.
    pub fn drain_commands(&self) -> Vec<C> {
        lock(&self.shared.commands).drain(..).collect()
    }

    /// Stores `snapshot` and pushes it to subscribers. Returns `false` (and notifies
    /// nobody) when it equals the previous one, so the runtime can call this every frame.
    pub fn publish(&self, snapshot: S) -> bool {
        let mut state = lock(&self.shared.snapshot);
        if state.latest == snapshot {
            return false;
        }
        state
            .subscribers
            .retain(|tx| tx.unbounded_send(snapshot.clone()).is_ok());
        state.latest = snapshot;
        true
    }
}
