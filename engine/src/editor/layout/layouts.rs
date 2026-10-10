//! The saved layouts: every workspace the user has (Blender's tabs along the top) and which one
//! is showing. This is what is written to the editor's preferences, so loading it must never
//! fail the editor: a damaged or empty file gives the default layouts.

use serde::{Deserialize, Serialize};

use super::presets;
use super::tree::{LayoutError, Workspace};

/// The longest name a layout tab can have.
pub const NAME_LIMIT: usize = 24;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layouts {
    pub workspaces: Vec<Workspace>,
    #[serde(default)]
    pub active: usize,
}

impl Default for Layouts {
    fn default() -> Self {
        Self {
            workspaces: presets::defaults(),
            active: 0,
        }
    }
}

impl Layouts {
    pub fn active_workspace(&self) -> &Workspace {
        &self.workspaces[self.active.min(self.workspaces.len() - 1)]
    }

    pub fn active_mut(&mut self) -> &mut Workspace {
        let last = self.workspaces.len() - 1;
        &mut self.workspaces[self.active.min(last)]
    }

    pub fn select(&mut self, index: usize) {
        if index < self.workspaces.len() {
            self.active = index;
        }
    }

    /// A name no tab has yet: `name`, else `name 2`, `name 3`...
    fn unique(&self, name: &str) -> String {
        let taken = |n: &str| self.workspaces.iter().any(|w| w.name == n);
        if !taken(name) {
            return name.to_string();
        }
        (2..)
            .map(|n| format!("{name} {n}"))
            .find(|n| !taken(n))
            .expect("an unused number exists")
    }

    /// Adds a workspace and shows it. Returns its index.
    pub fn add(&mut self, mut workspace: Workspace) -> usize {
        workspace.name = self.unique(&workspace.name);
        self.workspaces.push(workspace);
        self.active = self.workspaces.len() - 1;
        self.active
    }

    /// Puts a workspace first and shows it (the layout a small screen starts with).
    pub fn prefer(&mut self, mut workspace: Workspace) {
        workspace.name = self.unique(&workspace.name);
        self.workspaces.insert(0, workspace);
        self.active = 0;
    }

    /// Adds a copy of workspace `index` and shows it.
    pub fn duplicate(&mut self, index: usize) -> Option<usize> {
        let copy = self.workspaces.get(index)?.clone();
        Some(self.add(copy))
    }

    /// Moves workspace `from` to position `to`; the one on screen stays the one on screen.
    pub fn move_tab(&mut self, from: usize, to: usize) {
        let len = self.workspaces.len();
        if from >= len || to >= len || from == to {
            return;
        }
        let shown = self.active;
        let workspace = self.workspaces.remove(from);
        self.workspaces.insert(to, workspace);
        self.active = if shown == from {
            to
        } else if from < shown && to >= shown {
            shown - 1
        } else if from > shown && to <= shown {
            shown + 1
        } else {
            shown
        };
    }

    /// Removes workspace `index` (the last one cannot be removed).
    pub fn remove(&mut self, index: usize) -> Result<(), LayoutError> {
        if self.workspaces.len() <= 1 {
            return Err(LayoutError::LastWorkspace);
        }
        if index >= self.workspaces.len() {
            return Ok(());
        }
        self.workspaces.remove(index);
        if self.active > index || self.active >= self.workspaces.len() {
            self.active = self.active.saturating_sub(1);
        }
        Ok(())
    }

    /// Renames workspace `index` (as it is typed, so it may be empty for a moment).
    pub fn rename(&mut self, index: usize, name: &str) {
        if let Some(workspace) = self.workspaces.get_mut(index) {
            workspace.name = name.chars().take(NAME_LIMIT).collect();
        }
    }

    /// Gives workspace `index` its shipped layout back (a user-made one gets the default).
    pub fn reset_workspace(&mut self, index: usize) {
        if let Some(workspace) = self.workspaces.get_mut(index) {
            let name = workspace.name.clone();
            *workspace = presets::by_name(&name).unwrap_or_else(presets::layout);
            workspace.name = name;
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Reads saved layouts; `None` when the text is not a usable file.
    pub fn from_json(text: &str) -> Option<Layouts> {
        let mut layouts: Layouts = serde_json::from_str(text).ok()?;
        if layouts.workspaces.is_empty() {
            return None;
        }
        for workspace in &mut layouts.workspaces {
            workspace.repair();
            if workspace.name.trim().is_empty() {
                workspace.name = "Layout".into();
            }
        }
        layouts.active = layouts.active.min(layouts.workspaces.len() - 1);
        Some(layouts)
    }
}
