//! Qanat's **cell cursor**: a frame of four bars drawn over the selected cell (a square behind it
//! would show through the seam of the cell's sprite). `src/qanat` moves it from cell to cell.
//!
//! Prefabs are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::qanat::CellCursor;

const CURSOR: Color = Color::srgb(0.98, 0.82, 0.30);

pub fn prefab() -> PrefabAsset {
    PrefabAsset {
        root: Object::new("Cursor")
            .with(CellCursor)
            .with(SpriteDef {
                image: None,
                color: Color::NONE,
                size: Some(Vec2::new(1.0, 1.0)),
            })
            .with(Transform::from_xyz(-54.0, 62.0, 0.3))
            .child(
                Object::new("Cursor Top")
                    .with(SpriteDef {
                        image: None,
                        color: CURSOR,
                        size: Some(Vec2::new(60.0, 4.0)),
                    })
                    .with(Transform::from_xyz(0.0, 28.0, 0.0)),
            )
            .child(
                Object::new("Cursor Bottom")
                    .with(SpriteDef {
                        image: None,
                        color: CURSOR,
                        size: Some(Vec2::new(60.0, 4.0)),
                    })
                    .with(Transform::from_xyz(0.0, -28.0, 0.0)),
            )
            .child(
                Object::new("Cursor Left")
                    .with(SpriteDef {
                        image: None,
                        color: CURSOR,
                        size: Some(Vec2::new(4.0, 60.0)),
                    })
                    .with(Transform::from_xyz(-28.0, 0.0, 0.0)),
            )
            .child(
                Object::new("Cursor Right")
                    .with(SpriteDef {
                        image: None,
                        color: CURSOR,
                        size: Some(Vec2::new(4.0, 60.0)),
                    })
                    .with(Transform::from_xyz(28.0, 0.0, 0.0)),
            ),
    }
}
