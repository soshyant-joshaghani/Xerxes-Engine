//! The Game Preview: Play, Build and Publish for the open project. A job is the pipeline
//! (`game dev | build | publish <project> <platform>`) running on the machine that has the
//! project: through the backend on the web, in a process of its own in the native editor
//! (when it lives in the Xerxes repo). Its output is shown as it comes. A web job that serves
//! the game is shown in a frame on the web, and offered as a link in the native editor (which
//! has no frames). A finished build or publish says where its result went.

use dioxus::prelude::*;

use super::Editor;
use crate::editor::layout::EditorKind;
use crate::editor::prefs;
use crate::editor::project::jobs::{self, JobRequest, JobStatus};

pub const PLATFORMS: [(&str, &str); 3] = [
    ("web", "Web"),
    ("windows", "Windows"),
    ("android", "Android"),
];

/// Whether a job is running.
pub fn running(editor: Editor) -> bool {
    editor.job.read().as_ref().is_some_and(JobStatus::running)
}

/// Starts `command` (`dev`, `build` or `publish`) for the open project on the chosen platform,
/// and follows it until it ends.
pub fn run(editor: Editor, command: &'static str) {
    let Some(project) = editor.project.peek().clone() else {
        return;
    };
    if editor.job.peek().as_ref().is_some_and(JobStatus::running) {
        return;
    }
    super::show_editor(editor, EditorKind::GamePreview);
    let request = JobRequest {
        command: command.into(),
        target: project.name.clone(),
        platform: editor.platform.peek().clone(),
        template: project.kind == "template",
    };
    // Editor-level work: it must go on when the button that started it is gone.
    dioxus::core::spawn_forever(async move {
        let (mut job, mut note) = (editor.job, editor.job_note);
        note.set(None);
        let first = match jobs::api::start(&request).await {
            Ok(first) => first,
            Err(err) => {
                note.set(Some(err));
                return;
            }
        };
        let id = first.id;
        // So a reloaded page can pick the job up again.
        prefs::save("job", &id.to_string());
        job.set(Some(first));
        follow(editor, id).await;
    });
}

/// Keeps the job's status up to date (once a second) until it ends or another job takes over.
async fn follow(editor: Editor, id: u64) {
    let (mut job, mut note) = (editor.job, editor.job_note);
    loop {
        jobs::api::pause(1000).await;
        if job.peek().as_ref().map(|j| j.id) != Some(id) {
            break;
        }
        match jobs::api::status(id).await {
            Ok(next) => {
                let again = next.running();
                job.set(Some(next));
                if !again {
                    break;
                }
            }
            Err(err) => {
                note.set(Some(err));
                break;
            }
        }
    }
}

/// After a page reload: picks up the job that is still running for this project.
pub fn resume(editor: Editor) {
    let Some(project) = editor.project.peek().clone() else {
        return;
    };
    let Some(id) = prefs::load("job").and_then(|text| text.trim().parse::<u64>().ok()) else {
        return;
    };
    if editor.job.peek().is_some() {
        return;
    }
    dioxus::core::spawn_forever(async move {
        let mut job = editor.job;
        let Ok(found) = jobs::api::status(id).await else {
            return;
        };
        if found.running() && found.target() == Some(project.name.as_str()) {
            job.set(Some(found));
            follow(editor, id).await;
        }
    });
}

/// Stops the running job (and what it started: the dev server, the game).
pub fn stop(editor: Editor) {
    let Some(id) = editor
        .job
        .peek()
        .as_ref()
        .filter(|j| j.running())
        .map(|j| j.id)
    else {
        return;
    };
    dioxus::core::spawn_forever(async move {
        let mut note = editor.job_note;
        if let Err(err) = jobs::api::stop(id).await {
            note.set(Some(err));
        }
    });
}

#[component]
pub fn GamePreviewPanel() -> Element {
    let editor = use_context::<Editor>();
    let mut show_log = use_signal(|| false);
    let job = editor.job.read().clone();
    let note = editor.job_note.read().clone();
    let platform = editor.platform.read().clone();
    let busy = job.as_ref().is_some_and(JobStatus::running);
    let has_project = editor.project.read().is_some();
    // The address of a running web job (a dev server, or a built site served for a look).
    let url = job
        .as_ref()
        .filter(|j| j.running() && j.args.last().is_some_and(|p| p == "web"))
        .and_then(JobStatus::preview_url);
    let output = job
        .as_ref()
        .filter(|j| !j.running())
        .and_then(JobStatus::output);
    let summary = match &job {
        Some(j) if j.running() => format!("{}: running", j.title()),
        Some(j) => format!("{}: {}", j.title(), j.status),
        None => "Nothing has run yet.".to_string(),
    };
    // Frames exist on the web page, not in the native editor.
    let framed = url
        .clone()
        .filter(|_| !show_log() && cfg!(target_arch = "wasm32"));
    rsx! {
        div { class: "actions",
            for (id , label) in PLATFORMS {
                button {
                    key: "{id}",
                    class: if platform == id { "on" } else { "" },
                    disabled: busy,
                    onclick: move |_| {
                        let mut platform = editor.platform;
                        platform.set(id.to_string());
                    },
                    "{label}"
                }
            }
            div { class: "sep" }
            if busy {
                button { onclick: move |_| stop(editor), "Stop" }
            } else {
                button {
                    class: "play",
                    disabled: !has_project,
                    onclick: move |_| run(editor, "dev"),
                    "Play"
                }
            }
            button { disabled: busy || !has_project, onclick: move |_| run(editor, "build"), "Build" }
            button { disabled: busy || !has_project, onclick: move |_| run(editor, "publish"), "Publish" }
            button {
                class: if show_log() { "on" } else { "" },
                onclick: move |_| show_log.set(!show_log()),
                "Log"
            }
            span { class: "muted", "{summary}" }
            if let Some(path) = &output {
                span { class: "muted", "result: {path}" }
            }
            if let Some(address) = url.clone() {
                button { onclick: move |_| jobs::open_in_browser(&address), "Open in browser" }
                span { class: "muted", "{url.clone().unwrap_or_default()}" }
            }
        }
        if let Some(note) = note {
            div { class: "manager-banner", "{note}" }
        }
        if let Some(url) = framed {
            iframe { class: "preview-frame", src: "{url}" }
        } else {
            div { class: "body",
                match &job {
                    Some(j) if !j.log.is_empty() => rsx! {
                        for (index , line) in j.log.iter().enumerate() {
                            div { key: "{index}", class: "console-line", "{jobs::strip_ansi(line)}" }
                        }
                    },
                    Some(_) => rsx! { div { class: "hint", "Waiting for output..." } },
                    None => rsx! {
                        div { class: "hint",
                            "Play runs the open project on the platform above (it builds first, which takes a while the first time). On the web editor the web game shows here; the native editor offers it in your browser. Windows and Android open the game on the machine that runs the editor's pipeline. Build prepares the dev build; Publish makes the release build in dist/."
                        }
                    },
                }
            }
        }
    }
}
