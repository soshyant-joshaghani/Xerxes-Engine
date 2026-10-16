//! Undo and redo for the open scene. Every edit is recorded as a step with the state before and
//! after it (snapshots: scenes are small text documents, so this is simpler and safer than
//! writing an inverse for each kind of edit). Plain data, no Bevy or Dioxus, so it is tested.
//!
//! Consecutive edits to the same thing (dragging a gizmo, typing in a field) carry the same
//! `merge` key and fold into one step, until [`History::seal`] is called (the pointer was let
//! go, another object was selected, an undo happened...). Like Renzora's undo stacks, with
//! Fyrox's idea that the history is a list the user can see and jump around in.

#[derive(Debug, Clone)]
struct Step<S> {
    label: String,
    merge: Option<String>,
    before: S,
    after: S,
}

#[derive(Debug, Clone)]
pub struct History<S> {
    steps: Vec<Step<S>>,
    /// How many steps are applied: the next undo reverts `steps[applied - 1]`.
    applied: usize,
    /// The next edit starts a new step even if it has the same merge key.
    sealed: bool,
    /// `applied` when the document was last saved (`None`: that state is gone).
    saved: Option<usize>,
    limit: usize,
}

impl<S: Clone> Default for History<S> {
    fn default() -> Self {
        Self::new(200)
    }
}

impl<S: Clone> History<S> {
    /// An empty history that keeps at most `limit` steps.
    pub fn new(limit: usize) -> Self {
        Self {
            steps: Vec::new(),
            applied: 0,
            sealed: true,
            saved: Some(0),
            limit: limit.max(1),
        }
    }

    /// Forgets everything (a different document was opened). The state now is the saved one.
    pub fn clear(&mut self) {
        *self = Self::new(self.limit);
    }

    /// Records an edit: the state before it and after it. Steps that were undone are dropped.
    pub fn record(&mut self, label: &str, merge: Option<&str>, before: S, after: S) {
        if self.applied < self.steps.len() {
            self.steps.truncate(self.applied);
            if self.saved.is_some_and(|s| s > self.applied) {
                self.saved = None;
            }
        }
        let folds = !self.sealed
            && merge.is_some()
            && self
                .steps
                .last()
                .is_some_and(|top| top.merge.as_deref() == merge);
        if folds {
            if let Some(top) = self.steps.last_mut() {
                // The step keeps the label of its first edit ("Rename Ground", not what the
                // name became while typing).
                top.after = after;
            }
        } else {
            self.steps.push(Step {
                label: label.to_string(),
                merge: merge.map(str::to_string),
                before,
                after,
            });
            self.applied += 1;
            if self.steps.len() > self.limit {
                self.steps.remove(0);
                self.applied -= 1;
                self.saved = self.saved.and_then(|s| s.checked_sub(1));
            }
        }
        self.sealed = false;
    }

    /// The next edit starts a new step.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn can_undo(&self) -> bool {
        self.applied > 0
    }

    pub fn can_redo(&self) -> bool {
        self.applied < self.steps.len()
    }

    /// What the next undo would revert.
    pub fn undo_label(&self) -> Option<&str> {
        self.applied
            .checked_sub(1)
            .map(|i| self.steps[i].label.as_str())
    }

    /// What the next redo would apply again.
    pub fn redo_label(&self) -> Option<&str> {
        self.steps.get(self.applied).map(|s| s.label.as_str())
    }

    /// Reverts one step; returns the state to show.
    pub fn undo(&mut self) -> Option<&S> {
        let index = self.applied.checked_sub(1)?;
        self.applied = index;
        self.sealed = true;
        Some(&self.steps[index].before)
    }

    /// Applies the next step again; returns the state to show.
    pub fn redo(&mut self) -> Option<&S> {
        let step = self.steps.get(self.applied)?;
        self.applied += 1;
        self.sealed = true;
        Some(&step.after)
    }

    /// Goes to the state after `target` steps (0 = before the first); returns it, or `None`
    /// when already there.
    pub fn jump(&mut self, target: usize) -> Option<S> {
        let target = target.min(self.steps.len());
        if target == self.applied || self.steps.is_empty() {
            return None;
        }
        self.applied = target;
        self.sealed = true;
        Some(match target {
            0 => self.steps[0].before.clone(),
            n => self.steps[n - 1].after.clone(),
        })
    }

    /// The labels of all steps, oldest first, and how many are applied.
    pub fn labels(&self) -> (Vec<&str>, usize) {
        (
            self.steps.iter().map(|s| s.label.as_str()).collect(),
            self.applied,
        )
    }

    /// The document was saved in the current state.
    pub fn mark_saved(&mut self) {
        self.saved = Some(self.applied);
        self.sealed = true;
    }

    /// The current state is the one that was saved.
    pub fn is_clean(&self) -> bool {
        self.saved == Some(self.applied)
    }
}
