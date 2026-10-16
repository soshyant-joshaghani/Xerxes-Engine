//! The Project Manager: where the editor starts. Open a project, or create one from the engine's
//! starter or a template game (finished games are read-only: use one as the template of a new
//! project, and edit your own copy). Projects come from the project store (the disk
//! natively, the backend on the web), so the exe or APK works on any device by itself.
//! Scenes (2D or 3D) are not chosen here: they are added inside a project.

use dioxus::prelude::*;

use super::{Editor, Listing, ProjectEntry, Projects, Screen};
use crate::editor::project::{names, store};

/// The template the New project panel has selected (shared with the template rows on the left).
#[derive(Clone, Copy)]
struct TemplatePick(Signal<Option<String>>);

/// Opens a project in the editor.
pub fn open(editor: Editor, project: ProjectEntry) {
    let (mut open, mut screen) = (editor.project, editor.screen);
    open.set(Some(project));
    screen.set(Screen::Editor);
}

/// Reloads the project list.
pub async fn refresh(editor: Editor) {
    let mut projects = editor.projects;
    match store::list().await {
        Ok(listing) => projects.set(Projects::Ready(listing)),
        Err(err) => {
            bevy::log::warn!("editor: {err}");
            projects.set(Projects::Failed(err));
        }
    }
}

#[component]
pub fn Manager() -> Element {
    let editor = use_context::<Editor>();
    let projects = editor.projects.read().clone();
    let listing = match &projects {
        Projects::Ready(listing) => Some(listing.clone()),
        _ => None,
    };
    let picked = use_signal(|| None::<String>);
    use_context_provider(|| TemplatePick(picked));
    rsx! {
        div { class: "manager", "data-interactive": "true",
            div { class: "manager-head",
                div { class: "grow",
                    div { class: "manager-title", "Xerxes Engine" }
                    div { class: "muted", "Project Manager" }
                }
                button { onclick: move |_| { spawn(refresh(editor)); }, "Refresh" }
            }
            if let Projects::Failed(error) = &projects {
                div { class: "manager-banner",
                    div { class: "manager-name", "Cannot list projects" }
                    div { "{error}" }
                }
            }
            div { class: "manager-body",
                ProjectList { listing: listing.clone() }
                NewProjectPanel { listing }
            }
        }
    }
}

#[component]
fn ProjectList(listing: Option<Listing>) -> Element {
    let mut search = use_signal(String::new);
    let wanted = search.read().to_lowercase();
    let matches = |entry: &&ProjectEntry| {
        wanted.is_empty()
            || entry.name.contains(&wanted)
            || entry.title.to_lowercase().contains(&wanted)
    };
    let (games, templates): (Vec<ProjectEntry>, Vec<ProjectEntry>) = match &listing {
        Some(l) => (
            l.games.iter().filter(matches).cloned().collect(),
            l.templates.iter().filter(matches).cloned().collect(),
        ),
        None => (Vec::new(), Vec::new()),
    };
    let empty = listing
        .as_ref()
        .is_some_and(|l| l.games.is_empty() && l.templates.is_empty());
    rsx! {
        div { class: "manager-col",
            div { class: "manager-section", "Projects" }
            input {
                class: "num manager-input",
                value: "{search}",
                placeholder: "Search projects",
                oninput: move |e| search.set(e.value()),
            }
            div { class: "manager-list",
                if listing.is_none() {
                    div { class: "hint", "Loading projects…" }
                } else if empty {
                    div { class: "hint", "No projects yet. Create one on the right." }
                } else if games.is_empty() && templates.is_empty() {
                    div { class: "hint", "Nothing matches." }
                }
                if !games.is_empty() {
                    div { class: "muted", "Games" }
                }
                for entry in games {
                    ProjectRow { key: "{entry.id}", entry }
                }
                if !templates.is_empty() {
                    div { class: "muted", "Template games (read-only: make a project from one and edit your copy)" }
                }
                for entry in templates {
                    TemplateRow { key: "{entry.id}", entry }
                }
            }
        }
    }
}

#[component]
fn ProjectRow(entry: ProjectEntry) -> Element {
    let editor = use_context::<Editor>();
    let picked = entry.clone();
    rsx! {
        div { class: "manager-row", onclick: move |_| open(editor, picked.clone()),
            div { class: "grow",
                div { class: "manager-name", "{entry.title}" }
                div { class: "muted", "{entry.location}" }
                if let Some(text) = &entry.description {
                    div { class: "manager-desc", "{text}" }
                }
            }
            button { "Open" }
        }
    }
}

/// A finished game: it cannot be opened for editing, only used as the template of a new project.
#[component]
fn TemplateRow(entry: ProjectEntry) -> Element {
    let mut pick = use_context::<TemplatePick>().0;
    let name = entry.name.clone();
    rsx! {
        div { class: "manager-row", onclick: move |_| pick.set(Some(name.clone())),
            div { class: "grow",
                div { class: "manager-name", "{entry.title}" }
                div { class: "muted", "{entry.location}" }
                if let Some(text) = &entry.description {
                    div { class: "manager-desc", "{text}" }
                }
            }
            button { "Use as template" }
        }
    }
}

#[component]
fn NewProjectPanel(listing: Option<Listing>) -> Element {
    let editor = use_context::<Editor>();
    let mut name = use_signal(String::new);
    let mut touched = use_signal(|| false);
    let mut template = use_context::<TemplatePick>().0;
    let mut status = use_signal(|| None::<Result<String, String>>);
    let mut busy = use_signal(|| false);

    let starters = store::starters();
    let games: Vec<String> = listing
        .iter()
        .flat_map(|l| l.games.iter().map(|g| g.name.clone()))
        .collect();
    let template_names: Vec<String> = listing
        .iter()
        .flat_map(|l| l.templates.iter().map(|t| t.name.clone()))
        .collect();

    // Until the user types, a ready name, so Create works without typing (and without a
    // keyboard on a phone). The template defaults to the engine's `starter`.
    let current = if touched() {
        name.read().clone()
    } else {
        let taken: Vec<&str> = games
            .iter()
            .chain(&template_names)
            .map(String::as_str)
            .collect();
        names::unique("my-game", '-', &taken)
    };
    let chosen = template
        .read()
        .clone()
        .filter(|t| starters.iter().any(|s| &s.name == t))
        .or_else(|| {
            starters
                .iter()
                .find(|s| s.name == "starter")
                .or(starters.first())
                .map(|s| s.name.clone())
        });

    let problem = match &listing {
        None => Some("Loading projects…".to_string()),
        Some(_) => {
            let taken: Vec<&str> = games.iter().map(String::as_str).collect();
            let reserved: Vec<&str> = template_names.iter().map(String::as_str).collect();
            names::project_problem(&current, &taken, &reserved)
        }
    }
    .or_else(|| {
        chosen
            .is_none()
            .then(|| "This editor has no template games.".to_string())
    });
    let folder = listing
        .as_ref()
        .map(|l| l.new_projects_in.clone())
        .unwrap_or_default();

    let (wanted_name, wanted_template) = (current.clone(), chosen.clone());
    let create = move |_| {
        let wanted = wanted_name.clone();
        let Some(from) = wanted_template.clone() else {
            return;
        };
        if *busy.peek() {
            return;
        }
        busy.set(true);
        status.set(None);
        spawn(async move {
            let result = store::create(&wanted, &from).await;
            busy.set(false);
            match result {
                Ok(project) => {
                    touched.set(false);
                    editor.info(format!("created {} from {from}", project.location));
                    refresh(editor).await;
                    open(editor, project);
                }
                Err(err) => {
                    editor.error(format!("new project {wanted}: {err}"));
                    status.set(Some(Err(err)));
                }
            }
        });
    };

    rsx! {
        div { class: "manager-col narrow",
            div { class: "manager-section", "New project" }
            div { class: "muted", "Start from" }
            for starter in starters {
                div {
                    key: "{starter.name}",
                    class: if chosen.as_deref() == Some(starter.name.as_str()) { "manager-row sel" } else { "manager-row" },
                    onclick: {
                        let picked = starter.name.clone();
                        move |_| template.set(Some(picked.clone()))
                    },
                    div { class: "grow",
                        div { class: "manager-name", "{starter.title}" }
                        if let Some(text) = &starter.description {
                            div { class: "manager-desc", "{text}" }
                        }
                    }
                }
            }
            div { class: "muted", "Name" }
            input {
                class: "num manager-input",
                value: "{current}",
                placeholder: "my-game",
                disabled: busy(),
                oninput: move |e| {
                    touched.set(true);
                    name.set(names::normalize(&e.value()));
                },
            }
            if let Some(problem) = &problem {
                div { class: "manager-problem", "{problem}" }
            } else {
                div { class: "muted", "Creates {current} in {folder}, then opens it." }
            }
            div { class: "muted", "Scenes (2D or 3D) are added inside the project: Project panel, New Scene." }
            div { class: "manager-buttons",
                button {
                    class: "play grow",
                    disabled: busy() || problem.is_some(),
                    onclick: create,
                    if busy() { "Creating…" } else { "Create" }
                }
            }
            if let Some(Err(error)) = status() {
                div { class: "manager-problem", "Create failed: {error}" }
            }
        }
    }
}
