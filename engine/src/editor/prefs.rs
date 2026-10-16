//! The editor's own preferences (the saved layouts), kept per user, not per project: the
//! browser's local storage on the web, a file in the user's config folder natively (the app's
//! own storage on Android). Reading and writing never fail the editor: a missing or
//! unwritable store just means defaults and no saving.

/// The saved text for `key`, if any.
pub fn load(key: &str) -> Option<String> {
    imp::load(key)
}

/// Saves `text` under `key` (best effort).
pub fn save(key: &str, text: &str) {
    imp::save(key, text);
}

#[cfg(target_arch = "wasm32")]
mod imp {
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn name(key: &str) -> String {
        format!("xerxes.{key}")
    }

    pub fn load(key: &str) -> Option<String> {
        storage()?.get_item(&name(key)).ok().flatten()
    }

    pub fn save(key: &str, text: &str) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(&name(key), text);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::path::PathBuf;

    /// `XERXES_CONFIG` wins (tests, a portable setup); then the platform's config folder.
    fn folder() -> Option<PathBuf> {
        if let Some(dir) = std::env::var_os("XERXES_CONFIG") {
            return Some(PathBuf::from(dir));
        }
        #[cfg(target_os = "android")]
        if let Some(dir) = bevy::android::ANDROID_APP
            .get()
            .and_then(|app| app.internal_data_path())
        {
            return Some(dir.join("prefs"));
        }
        if let Some(dir) = std::env::var_os("APPDATA") {
            return Some(PathBuf::from(dir).join("Xerxes"));
        }
        if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(dir).join("xerxes"));
        }
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").join("xerxes"))
    }

    pub fn load(key: &str) -> Option<String> {
        std::fs::read_to_string(folder()?.join(format!("{key}.json"))).ok()
    }

    pub fn save(key: &str, text: &str) {
        let Some(dir) = folder() else { return };
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(format!("{key}.json")), text);
    }
}
