//! The bridge's Bevy end. The editor UI never touches the world: commands arrive as
//! [`UiCommand`] messages in `PreUpdate`, systems publish snapshots via [`UiBridge`].

use std::ops::Deref;

use bevy::prelude::*;

use crate::editor::bridge::RuntimePort;

/// A command sent from the UI, delivered once in the frame after it was sent.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct UiCommand<C: Send + Sync + 'static>(pub C);

/// The runtime end of the bridge. Publish snapshots with `bridge.publish(snapshot)`:
/// it is a no-op when nothing changed, so a system can call it every frame.
#[derive(Resource)]
pub struct UiBridge<C: Send + Sync + 'static, S: Send + Sync + 'static>(RuntimePort<C, S>);

impl<C: Send + Sync + 'static, S: Send + Sync + 'static> Deref for UiBridge<C, S> {
    type Target = RuntimePort<C, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Installs [`UiBridge`] and forwards UI commands into [`UiCommand`] messages.
pub struct UiBridgePlugin<C, S> {
    port: RuntimePort<C, S>,
}

impl<C, S> UiBridgePlugin<C, S> {
    pub fn new(port: RuntimePort<C, S>) -> Self {
        Self { port }
    }
}

impl<C, S> Plugin for UiBridgePlugin<C, S>
where
    C: Send + Sync + 'static,
    S: Clone + PartialEq + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(UiBridge(self.port.clone()))
            .add_message::<UiCommand<C>>()
            .add_systems(PreUpdate, forward_ui_commands::<C, S>);
    }
}

fn forward_ui_commands<C, S>(bridge: Res<UiBridge<C, S>>, mut out: MessageWriter<UiCommand<C>>)
where
    C: Send + Sync + 'static,
    S: Clone + PartialEq + Send + Sync + 'static,
{
    out.write_batch(bridge.drain_commands().into_iter().map(UiCommand));
}
