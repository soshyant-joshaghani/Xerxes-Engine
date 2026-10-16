//! Small persistent text storage for games, the same on every platform: the browser's local
//! storage on the web, a file in the user's data folder natively, the app's own storage on
//! Android. A game keeps a best run, a ghost, a setting, a "played today" flag under a key of
//! its choosing (`"<game>.<what>"`).
//!
//! It is the platform piece of the save stage (PROGRESS 3.6): no schema, no serializer, no cloud;
//! those are built on top when savers and variables exist. Until then a game encodes its own
//! text.
//!
//! Reading and writing never fail a game: a missing or unwritable store means "nothing saved"
//! and the value lives for the session only (an in-memory copy backs every write, so a later
//! `load` in the same run still sees it). The editor's own preferences are a separate store
//! (`editor::prefs`, per user and not per game).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// The text saved under `key`, if any.
pub fn load(key: &str) -> Option<String> {
    if let Some(text) = session().lock().ok()?.get(key) {
        return Some(text.clone());
    }
    let text = imp::load(key)?;
    if let Ok(mut map) = session().lock() {
        map.insert(key.to_string(), text.clone());
    }
    Some(text)
}

/// Saves `text` under `key`. Returns whether it reached the platform's storage (the value is
/// kept for the session either way).
pub fn save(key: &str, text: &str) -> bool {
    if let Ok(mut map) = session().lock() {
        map.insert(key.to_string(), text.to_string());
    }
    imp::save(key, text)
}

/// Forgets `key`.
pub fn remove(key: &str) {
    if let Ok(mut map) = session().lock() {
        map.remove(key);
    }
    imp::remove(key);
}

/// What this session wrote or read, so a failed write still reads back.
fn session() -> &'static Mutex<HashMap<String, String>> {
    static SESSION: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    SESSION.get_or_init(Default::default)
}

/// A key as a file name: letters, digits, `-`, `_` and `.` stay, anything else becomes `_`.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn file_name(key: &str) -> String {
    let safe: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{safe}.txt")
}

#[cfg(target_arch = "wasm32")]
mod imp {
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn name(key: &str) -> String {
        format!("xerxes.data.{key}")
    }

    pub fn load(key: &str) -> Option<String> {
        storage()?.get_item(&name(key)).ok().flatten()
    }

    pub fn save(key: &str, text: &str) -> bool {
        storage().is_some_and(|s| s.set_item(&name(key), text).is_ok())
    }

    pub fn remove(key: &str) {
        if let Some(storage) = storage() {
            let _ = storage.remove_item(&name(key));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    /// `XERXES_DATA` wins (tests, a portable setup); then the platform's data folder.
    fn folder() -> Option<PathBuf> {
        if let Some(dir) = std::env::var_os("XERXES_DATA") {
            return Some(PathBuf::from(dir));
        }
        #[cfg(target_os = "android")]
        if let Some(dir) = bevy::android::ANDROID_APP
            .get()
            .and_then(|app| app.internal_data_path())
        {
            return Some(dir.join("data"));
        }
        if let Some(dir) = std::env::var_os("APPDATA") {
            return Some(PathBuf::from(dir).join("Xerxes").join("data"));
        }
        if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
            return Some(PathBuf::from(dir).join("xerxes"));
        }
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".local").join("share").join("xerxes"))
    }

    pub fn load(key: &str) -> Option<String> {
        std::fs::read_to_string(folder()?.join(super::file_name(key))).ok()
    }

    pub fn save(key: &str, text: &str) -> bool {
        let Some(dir) = folder() else {
            return false;
        };
        if std::fs::create_dir_all(&dir).is_err() {
            return false;
        }
        // Written beside and renamed over, so a crash never leaves half a file.
        let file = dir.join(super::file_name(key));
        let part = file.with_extension("part");
        std::fs::write(&part, text).is_ok() && std::fs::rename(&part, &file).is_ok()
    }

    pub fn remove(key: &str) {
        if let Some(dir) = folder() {
            let _ = std::fs::remove_file(dir.join(super::file_name(key)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_a_safe_file_name() {
        assert_eq!(file_name("darius.chapar.ghost"), "darius.chapar.ghost.txt");
        assert_eq!(file_name("../a b/c"), ".._a_b_c.txt");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn saved_text_reads_back_and_can_be_removed() {
        let dir = std::env::temp_dir().join(format!("xerxes-storage-test-{}", std::process::id()));
        // SAFETY: a test that owns this variable; no other test in this crate reads it.
        unsafe { std::env::set_var("XERXES_DATA", &dir) };
        let key = "test.storage.roundtrip";
        remove(key);
        assert_eq!(load(key), None);
        assert!(save(key, "one\ntwo"));
        assert_eq!(load(key).as_deref(), Some("one\ntwo"));
        // It is on disk, not only in this session's copy.
        assert_eq!(
            std::fs::read_to_string(dir.join(file_name(key))).unwrap(),
            "one\ntwo"
        );
        remove(key);
        assert_eq!(load(key), None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
