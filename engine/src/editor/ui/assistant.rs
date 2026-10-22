//! The Assistant editor: an AI chat beside the scene, for building a game, extending the engine
//! and working on the backend. Not wired yet: the panel is the finished place for it (messages,
//! suggestions, an input) so a model can be connected later without touching the layout.
//! See __plans__/PROGRESS.md (stage 2i).

use dioxus::prelude::*;

/// What the Assistant is for, shown as starting points.
const SUGGESTIONS: [(&str, &str); 4] = [
    ("Game", "Add a dash ability to the player"),
    ("Scene", "Lay out a main menu with Play, Options and Quit"),
    ("Engine", "Add a Slider widget style to the UI kit"),
    ("Backend", "Add a leaderboard route and table to the API"),
];

#[component]
pub fn AssistantPanel() -> Element {
    rsx! {
        div { class: "chat",
            div { class: "chat-log",
                div { class: "chat-badge", "Coming soon" }
                div { class: "chat-msg bot",
                    div { class: "chat-who", "Assistant" }
                    "I will help you build your game, extend the engine for what it needs, and work on the backend (API, rooms, database). It is not connected to a model yet."
                }
                div { class: "chat-msg bot",
                    div { class: "chat-who", "Assistant" }
                    "When it is, it will read this project (scenes, logic, assets) and write code that follows the engine's rules. Things you will be able to ask:"
                }
                for (area, text) in SUGGESTIONS {
                    div { key: "{area}", class: "chat-suggest",
                        span { class: "chat-area", "{area}" }
                        "{text}"
                    }
                }
            }
            div { class: "chat-input",
                input {
                    class: "num",
                    disabled: true,
                    placeholder: "Ask the Assistant (coming soon)",
                }
                button { disabled: true, "Send" }
            }
        }
    }
}
