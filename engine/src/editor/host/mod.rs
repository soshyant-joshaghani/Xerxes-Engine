//! Where the editor's Dioxus UI runs.
//!
//! - `web`     Dioxus renders the page (DOM); Bevy starts inside its canvas once it is mounted
//! - `native`  Bevy owns the window; Blitz (Dioxus native's renderer) lays out and paints the
//!   same components into a texture shown over the world, with mouse and touch forwarded
//!
//! Both render the same page ([`page`]: one stylesheet, the stage and HUD layers) around the
//! same root component, so the UI is written once and lays out the same on every target. The
//! only difference is the viewport layer: the web page holds Bevy's `<canvas>`; natively it is
//! empty because Bevy draws the world under the UI.

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use bevy::prelude::App;
use dioxus::prelude::*;
use futures_util::StreamExt;

#[cfg(not(target_arch = "wasm32"))]
use super::bridge::bridge;
use super::bridge::{RuntimePort, UiPort};

/// Whether a UI text input has keyboard focus: then the Scene view leaves the keyboard alone.
/// On the web the DOM routes keys to the focused input; natively the overlay sets this.
#[derive(bevy::prelude::Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct UiFocus(pub bool);

/// The root component, given the UI end of the bridge. Return a component
/// (`rsx! { Panel { port } }`) so its hooks get their own scope.
pub type Root<C, S> = fn(UiPort<C, S>) -> Element;

/// Builds the editor's Bevy app from the bridge's runtime end and the web canvas selector.
pub type BuildApp<C, S> = fn(RuntimePort<C, S>, Option<String>) -> App;

/// The host page's stylesheet, shared by both hosts. Everything the UI relies on is spelled
/// out here rather than left to a renderer's defaults (Blitz is not a browser):
/// - panels take the pointer only where marked `data-interactive` (Blitz has no
///   `pointer-events`, so the native host forwards input to the whole document; the web
///   host must opt the same areas in, or the wheel falls through to the canvas);
/// - no scrollbars anywhere (Blitz does not paint them); lists scroll with the wheel or a
///   touch drag on every platform.
const HOST_STYLE: &str = r#"
html, body, #main { margin: 0; height: 100%; overflow: hidden; background: transparent; }
.xerxes-stage { position: absolute; inset: 0; }
.xerxes-canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; outline: none; }
.xerxes-hud { position: absolute; inset: 0; pointer-events: none; }
.xerxes-hud button, .xerxes-hud a, .xerxes-hud input, .xerxes-hud [data-interactive] { pointer-events: auto; }
.xerxes-hud button { text-align: center; }
.xerxes-hud * { scrollbar-width: none; }
.xerxes-hud *::-webkit-scrollbar { display: none; }
"#;

/// The host page: the viewport layer under the HUD layer, which holds the root UI.
fn page(viewport: Element, hud: Element) -> Element {
    rsx! {
        style { {HOST_STYLE} }
        div { class: "xerxes-stage",
            {viewport}
            div { class: "xerxes-hud", {hud} }
        }
    }
}

/// The native VirtualDom's root: the host page with an empty viewport layer.
#[cfg(not(target_arch = "wasm32"))]
fn native_page<C: 'static, S: Clone + 'static>(
    (port, root): (UiPort<C, S>, Root<C, S>),
) -> Element {
    page(rsx! {}, root(port))
}

/// Runs the Bevy app `app` builds, with `root` as its Dioxus UI.
pub fn launch<C, S>(app: BuildApp<C, S>, root: Root<C, S>)
where
    C: Send + Sync + 'static,
    S: Clone + PartialEq + Default + Send + Sync + 'static,
{
    #[cfg(target_arch = "wasm32")]
    web::launch(app, root);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let (ui, runtime) = bridge(S::default());
        let mut app = app(runtime, None);
        app.add_plugins(native::DioxusOverlayPlugin::new(Arc::new(move || {
            VirtualDom::new_with_props(native_page::<C, S>, (ui.clone(), root))
        })));
        app.run();
    }
}

/// The runtime's latest snapshot, updated whenever it publishes a new one.
pub fn use_runtime_snapshot<C, S>(port: &UiPort<C, S>) -> ReadSignal<S>
where
    C: 'static,
    S: Clone + 'static,
{
    let mut snapshot = use_signal(|| port.latest());
    let port = port.clone();
    use_future(move || {
        let port = port.clone();
        async move {
            // Subscribe before re-reading `latest` so nothing published in between is lost.
            let mut updates = port.subscribe();
            snapshot.set(port.latest());
            while let Some(next) = updates.next().await {
                snapshot.set(next);
            }
        }
    });
    snapshot.into()
}
