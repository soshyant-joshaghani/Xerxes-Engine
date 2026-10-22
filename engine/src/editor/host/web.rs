//! The editor on the web: Dioxus renders the page with a full-screen canvas, Bevy starts
//! inside it once the canvas is mounted, and the root UI is drawn on top.
//!
//! ```text
//! Dioxus host ─┬─ GameViewport <canvas>  ◀── Bevy App (BuildApp)
//!              └─ Root(UiPort)           ⇄  bridge<C, S>
//! ```

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use dioxus::prelude::*;

use super::{BuildApp, Root, page};
use crate::editor::bridge::bridge;

const CANVAS_ID: &str = "xerxes-canvas";

/// What the host component builds, handed over as a root context.
struct Setup<C, S> {
    app: BuildApp<C, S>,
    root: Root<C, S>,
}

impl<C, S> Clone for Setup<C, S> {
    fn clone(&self) -> Self {
        Self {
            app: self.app,
            root: self.root,
        }
    }
}

pub fn launch<C, S>(app: BuildApp<C, S>, root: Root<C, S>)
where
    C: Send + Sync + 'static,
    S: Clone + PartialEq + Default + Send + Sync + 'static,
{
    LaunchBuilder::new()
        .with_context(Setup { app, root })
        .launch(host::<C, S>);
}

fn host<C, S>() -> Element
where
    C: Send + Sync + 'static,
    S: Clone + PartialEq + Default + Send + Sync + 'static,
{
    let Setup { app, root } = use_context::<Setup<C, S>>();
    let (ui, runtime) = use_hook(|| bridge(S::default()));

    // The page title comes from the target's Dioxus.toml ([web.app] title).
    page(
        rsx! {
            GameViewport {
                id: CANVAS_ID,
                class: "xerxes-canvas",
                on_ready: move |_| run_once(|| app(runtime.clone(), Some(format!("#{CANVAS_ID}")))),
            }
        },
        root(ui.clone()),
    )
}

/// Builds and runs the Bevy app unless one already ran on this page. Bevy owns one winit
/// event loop per process and cannot re-attach to a new canvas. On wasm `run` hands the loop
/// to the browser and returns.
fn run_once(build: impl FnOnce() -> bevy::prelude::App) {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if !STARTED.swap(true, Ordering::SeqCst) {
        build().run();
    }
}

/// The canvas Bevy renders into. `on_ready` fires once, after the canvas is in the document
/// and focused (keyboard input goes to the focused canvas), which is when Bevy can attach to
/// it by `#id`. Keep it mounted for the life of the page.
#[component]
fn GameViewport(
    id: String,
    #[props(default)] class: String,
    on_ready: EventHandler<()>,
) -> Element {
    let fired = use_hook(|| Rc::new(Cell::new(false)));

    rsx! {
        canvas {
            id,
            class,
            tabindex: "0",
            oncontextmenu: move |evt| evt.prevent_default(),
            onmounted: move |evt| {
                let first = !fired.replace(true);
                async move {
                    if first {
                        let _ = evt.set_focus(true).await;
                        on_ready.call(());
                    }
                }
            },
        }
    }
}
