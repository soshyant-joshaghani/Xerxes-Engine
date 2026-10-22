//! **Siavash**, the knight who runs: the test character's glTF model (`MD_CHR_TPS.glb`), his feet
//! on the road, plus the ring that lights up while his shield is on.
//!
//! The road runs toward **+Z**, which is also the front axis of a glTF model, so the knight needs
//! no rotation: he faces the way he runs and the camera, behind him, sees his back.
//!
//! The model comes in at the size it is played at (a 2 m character; its bounds reach higher
//! because of the hat and the other pieces he wears), so it needs no fitting. It sits on a child
//! of its own all the same: `src/siavash` writes the **root's** transform every frame (the lane,
//! and the press down for a roll), so the art's transform stays the author's to set in the editor.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::siavash::{Knight, ShieldRing};

/// The shield's ring, the one piece of feedback the model itself cannot give.
const SHIELD: Color = Color::srgb(1.0, 0.78, 0.2);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Knight")
            .with(Knight)
            .with(Transform::from_xyz(0.0, 0.0, 0.0))
            .child(
                Object::new("Model")
                    .with(ModelDef {
                        model: "meshes/chr/MD_CHR_TPS.glb",
                        scene: 0,
                    })
                    .with(Transform::from_xyz(0.0, 0.0, 0.0)),
            )
            .child(
                Object::new("Shield Ring")
                    .with(ShieldRing)
                    .with(MeshDef::Plane { size: 2.2 })
                    .with(MaterialDef {
                        color: SHIELD,
                        texture: None,
                    })
                    .with(Transform::from_xyz(0.0, 0.02, 0.0))
                    .with(Visibility::Hidden),
            ),
    }
}
