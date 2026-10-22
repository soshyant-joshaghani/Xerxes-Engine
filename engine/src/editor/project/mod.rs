//! The open project, as the editor sees it: where projects live (`store`: the disk natively,
//! the backend on the web through `client`), the `.rs` asset codec (`codec`), and the naming
//! rules the editor checks as you type (`names`), and the one rule that turns a prefab file into
//! the path a scene writes for it ([`prefab_ref`]).

pub mod browse;
#[cfg(target_arch = "wasm32")]
pub mod client;
pub mod codec;
pub mod jobs;
pub mod names;
pub mod settings_text;
pub mod store;
pub mod table;

/// The path a scene writes for a prefab file (`prefabs/coin.prefab.rs` →
/// `crate::assets::prefabs::coin_prefab::prefab`), by the same rule `build.rs` names the modules
/// it generates from `assets/`. Nested folders come through as they are
/// (`prefabs/games/solo/qanat/cell.prefab.rs` →
/// `crate::assets::prefabs::games::solo::qanat::cell_prefab::prefab`).
pub fn prefab_ref(path: &str) -> String {
    let module = xerxes_build::module_path(path);
    format!(
        "crate::assets::{}::prefab",
        module.trim_start_matches("self::")
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_prefab_file_becomes_the_path_a_scene_writes() {
        assert_eq!(
            super::prefab_ref("prefabs/coin.prefab.rs"),
            "crate::assets::prefabs::coin_prefab::prefab"
        );
        assert_eq!(
            super::prefab_ref("prefabs/games/solo/qanat/cell.prefab.rs"),
            "crate::assets::prefabs::games::solo::qanat::cell_prefab::prefab"
        );
    }
}
