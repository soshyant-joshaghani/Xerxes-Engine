//! The Preferences editor: how the editor behaves for this user, kept in the browser or the
//! user's config folder (not in a project). Touch sizes (Auto, On, Off) and the keymap: every
//! action with the keys that run it, which can be changed, with a warning when two actions
//! share a key. A change is in effect at once.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::keymap;
use crate::editor::protocol::Action;
use crate::editor::touch;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
fn KeyRow(action: Action, changed: Signal<u32>) -> Element {
    let current = keymap::effective_with(action, &keymap::overrides()).join(", ");
    let own = keymap::overrides().contains_key(action.id());
    let mut draft = use_signal(|| current.clone());
    let mut problem = use_signal(|| None::<String>);
    let edited = *draft.read() != current;
    let apply = move |_| {
        let typed = draft.peek().clone();
        match keymap::parse_list(&typed) {
            Err(why) => problem.set(Some(why)),
            Ok(list) => {
                problem.set(None);
                let mut all = keymap::overrides();
                if list == keymap::effective_with(action, &Default::default()) {
                    all.remove(action.id());
                } else {
                    all.insert(action.id().to_string(), list.clone());
                }
                keymap::set_overrides(all);
                draft.set(list.join(", "));
                changed.set(changed() + 1);
            }
        }
    };
    let reset = move |_| {
        let mut all = keymap::overrides();
        all.remove(action.id());
        let list = keymap::effective_with(action, &Default::default());
        keymap::set_overrides(all);
        draft.set(list.join(", "));
        problem.set(None);
        changed.set(changed() + 1);
    };
    rsx! {
        div { class: "field",
            span { class: "label", "{action.title()}" }
            div { class: "value",
                input {
                    class: "num",
                    value: "{draft}",
                    placeholder: "no key",
                    oninput: move |e| draft.set(e.value()),
                }
                button { class: "dim", disabled: !edited, onclick: apply, "Set" }
                button { class: "dim", disabled: !own, onclick: reset, "Reset" }
            }
        }
        if let Some(why) = problem() {
            div { class: "field", span { class: "manager-problem", "{why}" } }
        }
    }
}

#[component]
pub fn PreferencesPanel() -> Element {
    let editor = use_context::<Editor>();
    let changed = use_signal(|| 0u32);
    let _ = changed();
    let mut touch_mode = use_signal(touch::mode);
    let clashes = keymap::conflicts_with(&keymap::overrides());
    rsx! {
        div { class: "body",
            div { class: "section",
                div { class: "section-head", "Interface" }
                div { class: "field",
                    span { class: "label", "Touch sizes" }
                    div { class: "value",
                        for (id , label) in touch::MODES {
                            button {
                                key: "{id}",
                                class: if touch_mode() == id { "on" } else { "" },
                                onclick: move |_| {
                                    touch::apply(id, true);
                                    touch_mode.set(id.to_string());
                                },
                                "{label}"
                            }
                        }
                    }
                }
                div { class: "field",
                    span { class: "hint", "Bigger buttons and grab zones for a finger. Auto decides from the device." }
                }
            }
            div { class: "section",
                div { class: "section-head", "Keymap" }
                for action in Action::ALL {
                    KeyRow { key: "{action.id()}", action, changed }
                }
                for (first , second , chord) in clashes {
                    div { key: "{chord}", class: "field",
                        span { class: "manager-problem", "{first.title()} and {second.title()} both use {chord}." }
                    }
                }
                div { class: "field",
                    span { class: "hint", "Keys separated by commas (Ctrl+Shift+Z, Ctrl+Y). W E R X F and the camera keys belong to the Viewport." }
                }
            }
            div { class: "section",
                div { class: "section-head", "Layouts" }
                div { class: "field",
                    span { class: "label", "This project" }
                    div { class: "value",
                        button {
                            onclick: move |_| {
                                let mut layouts = editor.layouts;
                                layouts.set(crate::editor::layout::Layouts::default());
                            },
                            "Reset all layouts"
                        }
                    }
                }
            }
            div { class: "section",
                div { class: "section-head", "About" }
                div { class: "field",
                    span { class: "label", "Xerxes Engine" }
                    span { class: "value muted", "{VERSION}" }
                }
            }
        }
    }
}
