//! The meta of `MD_CHR_TPS.glb`, the third-person test character.
//!
//! It ships in the **`siavash`** bundle, not in `core`: the bundle a file names is what makes a
//! web build progressive. Nothing downloads it until a level that draws it is asked for, and then
//! the flow's Loading step has it in memory before the level starts (the engine reads what a
//! scene's objects load, so there is no list to keep in step). Hovering Siavash's menu tile
//! fetches it while the player is still choosing.
//!
//! A glTF model faces **+Z** with **+Y** up. Siavash runs toward +Z, so the knight needs no
//! rotation of its own: the camera sits behind him and sees his back.

use xerxes_engine::prelude::*;

pub fn meta() -> AssetMeta {
    AssetMeta::mesh().bundle("siavash")
}
