//! Qanat (Solo · Objective): a hillside cut open, a spring at the top left, a village at the
//! bottom right. Lay and turn channel tiles in the soil so the water runs down to the village;
//! fewer tiles earn a better tier. GDD: `__docs__/gdd/darius/solo-objective-qanat.md`.
//!
//! The scene has three objects at its root, like Ahriman's Horde: the `Camera`, the `Game` canvas (a
//! world-space layer: the hill with its grass edge, the cursor and the room's 9 x 7 cells, each a
//! `QanatCell` saying what it is made of) and the `HUD` canvas. The room *is* these cells: the
//! game's code (`src/qanat`) builds its rules from them and draws the tiles over them.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

use crate::layout::{Band, BandContent, BandToggle, BandToggleLabel, HeaderBand};
use crate::qanat::{
    BoardCamera, CellCursor, Ground, QanatAction, QanatButton, QanatCell, ResultBanner, ResultText,
    StatusText,
};

const GRASS: Color = Color::srgb(0.42, 0.56, 0.27);
const HILL: Color = Color::srgb(0.20, 0.16, 0.13);
const ROCK: Color = Color::srgb(0.33, 0.27, 0.22);
const SOIL: Color = Color::srgb(0.56, 0.43, 0.31);
const SPRING: Color = Color::srgb(0.20, 0.47, 0.72);
const VILLAGE: Color = Color::srgb(0.87, 0.76, 0.52);
const CURSOR: Color = Color::srgb(0.98, 0.82, 0.30);
// Opaque: the header bar hides the world behind it.
const PANEL: Color = Color::srgb(0.047, 0.055, 0.071);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const DIM: Color = Color::srgb(0.62, 0.65, 0.70);
const STATUS: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera")
                .with(GameCamera2D)
                .with(BoardCamera)
                // The room is built around (170, -50), not the origin, so point the camera there to
                // centre it in the middle band.
                .with(Transform::from_xyz(170.0, -50.0, 0.0))
                .with(Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: 620.0,
                        min_height: 520.0,
                    },
                    ..OrthographicProjection::default_2d()
                })),
            Object::new("Game")
                .with(Canvas::world(800, 500, 1.0))
                .child(
                    Object::new("Hill")
                        .with(SpriteDef {
                            image: None,
                            color: HILL,
                            size: Some(Vec2::new(516.0, 404.0)),
                        })
                        .with(Transform::from_xyz(170.0, -50.0, 0.0)),
                )
                .child(
                    Object::new("Grass")
                        .with(SpriteDef {
                            image: None,
                            color: GRASS,
                            size: Some(Vec2::new(516.0, 8.0)),
                        })
                        .with(Transform::from_xyz(170.0, 156.0, 0.0)),
                )
                .child(
                    // A frame of four bars drawn over the selected cell (a square behind it would
                    // show through the seam of the cell's sprite).
                    Object::new("Cursor")
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
                )
                .child(
                    Object::new("Cell 0,0")
                        .with(QanatCell {
                            col: 0,
                            row: 0,
                            ground: Ground::Spring,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SPRING,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,0")
                        .with(QanatCell {
                            col: 1,
                            row: 0,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,0")
                        .with(QanatCell {
                            col: 2,
                            row: 0,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,0")
                        .with(QanatCell {
                            col: 3,
                            row: 0,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,0")
                        .with(QanatCell {
                            col: 4,
                            row: 0,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,0")
                        .with(QanatCell {
                            col: 5,
                            row: 0,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,0")
                        .with(QanatCell {
                            col: 6,
                            row: 0,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,0")
                        .with(QanatCell {
                            col: 7,
                            row: 0,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,0")
                        .with(QanatCell {
                            col: 8,
                            row: 0,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, 118.0, 0.1)),
                )
                .child(
                    Object::new("Cell 0,1")
                        .with(QanatCell {
                            col: 0,
                            row: 1,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,1")
                        .with(QanatCell {
                            col: 1,
                            row: 1,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,1")
                        .with(QanatCell {
                            col: 2,
                            row: 1,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,1")
                        .with(QanatCell {
                            col: 3,
                            row: 1,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,1")
                        .with(QanatCell {
                            col: 4,
                            row: 1,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,1")
                        .with(QanatCell {
                            col: 5,
                            row: 1,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,1")
                        .with(QanatCell {
                            col: 6,
                            row: 1,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,1")
                        .with(QanatCell {
                            col: 7,
                            row: 1,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,1")
                        .with(QanatCell {
                            col: 8,
                            row: 1,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, 62.0, 0.1)),
                )
                .child(
                    Object::new("Cell 0,2")
                        .with(QanatCell {
                            col: 0,
                            row: 2,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,2")
                        .with(QanatCell {
                            col: 1,
                            row: 2,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,2")
                        .with(QanatCell {
                            col: 2,
                            row: 2,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,2")
                        .with(QanatCell {
                            col: 3,
                            row: 2,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,2")
                        .with(QanatCell {
                            col: 4,
                            row: 2,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,2")
                        .with(QanatCell {
                            col: 5,
                            row: 2,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,2")
                        .with(QanatCell {
                            col: 6,
                            row: 2,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,2")
                        .with(QanatCell {
                            col: 7,
                            row: 2,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,2")
                        .with(QanatCell {
                            col: 8,
                            row: 2,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, 6.0, 0.1)),
                )
                .child(
                    Object::new("Cell 0,3")
                        .with(QanatCell {
                            col: 0,
                            row: 3,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,3")
                        .with(QanatCell {
                            col: 1,
                            row: 3,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,3")
                        .with(QanatCell {
                            col: 2,
                            row: 3,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,3")
                        .with(QanatCell {
                            col: 3,
                            row: 3,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,3")
                        .with(QanatCell {
                            col: 4,
                            row: 3,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,3")
                        .with(QanatCell {
                            col: 5,
                            row: 3,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,3")
                        .with(QanatCell {
                            col: 6,
                            row: 3,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,3")
                        .with(QanatCell {
                            col: 7,
                            row: 3,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,3")
                        .with(QanatCell {
                            col: 8,
                            row: 3,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, -50.0, 0.1)),
                )
                .child(
                    Object::new("Cell 0,4")
                        .with(QanatCell {
                            col: 0,
                            row: 4,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,4")
                        .with(QanatCell {
                            col: 1,
                            row: 4,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,4")
                        .with(QanatCell {
                            col: 2,
                            row: 4,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,4")
                        .with(QanatCell {
                            col: 3,
                            row: 4,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,4")
                        .with(QanatCell {
                            col: 4,
                            row: 4,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,4")
                        .with(QanatCell {
                            col: 5,
                            row: 4,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,4")
                        .with(QanatCell {
                            col: 6,
                            row: 4,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,4")
                        .with(QanatCell {
                            col: 7,
                            row: 4,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,4")
                        .with(QanatCell {
                            col: 8,
                            row: 4,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, -106.0, 0.1)),
                )
                .child(
                    Object::new("Cell 0,5")
                        .with(QanatCell {
                            col: 0,
                            row: 5,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,5")
                        .with(QanatCell {
                            col: 1,
                            row: 5,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,5")
                        .with(QanatCell {
                            col: 2,
                            row: 5,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,5")
                        .with(QanatCell {
                            col: 3,
                            row: 5,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,5")
                        .with(QanatCell {
                            col: 4,
                            row: 5,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,5")
                        .with(QanatCell {
                            col: 5,
                            row: 5,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,5")
                        .with(QanatCell {
                            col: 6,
                            row: 5,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,5")
                        .with(QanatCell {
                            col: 7,
                            row: 5,
                            ground: Ground::Soil,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: SOIL,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,5")
                        .with(QanatCell {
                            col: 8,
                            row: 5,
                            ground: Ground::Village,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: VILLAGE,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, -162.0, 0.1)),
                )
                .child(
                    Object::new("Cell 0,6")
                        .with(QanatCell {
                            col: 0,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(-54.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 1,6")
                        .with(QanatCell {
                            col: 1,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(2.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 2,6")
                        .with(QanatCell {
                            col: 2,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(58.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 3,6")
                        .with(QanatCell {
                            col: 3,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(114.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 4,6")
                        .with(QanatCell {
                            col: 4,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(170.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 5,6")
                        .with(QanatCell {
                            col: 5,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(226.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 6,6")
                        .with(QanatCell {
                            col: 6,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(282.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 7,6")
                        .with(QanatCell {
                            col: 7,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(338.0, -218.0, 0.1)),
                )
                .child(
                    Object::new("Cell 8,6")
                        .with(QanatCell {
                            col: 8,
                            row: 6,
                            ground: Ground::Rock,
                        })
                        .with(SpriteDef {
                            image: None,
                            color: ROCK,
                            size: Some(Vec2::new(52.0, 52.0)),
                        })
                        .with(Transform::from_xyz(394.0, -218.0, 0.1)),
                ),
            // Qanat is played with the pointer, so it has no controls footer: one header bar at the
            // top (text on the left, the action buttons on the right, the collapse toggle in the
            // corner) and the room filling everything below it. The header's height and the game's
            // viewport are driven by `crate::layout`; every child is placed with an explicit anchor.
            Object::new("HUD")
                .with(Canvas::overlay())
                .child(
                    Object::new("Header")
                        .with(HeaderBand)
                        .with(Node::default())
                        // Above the banner, so the bar stays solid and clickable.
                        .with(ZIndex(1))
                        .with(RectTransform {
                            anchor_min: Vec2::new(0.0, 0.8),
                            anchor_max: Vec2::new(1.0, 1.0),
                            pivot: Vec2::new(0.5, 0.5),
                            position: Vec2::new(0.0, 0.0),
                            size: Vec2::new(0.0, 0.0),
                        })
                        .with(BackgroundColor(PANEL))
                        .child(
                            Object::new("Info")
                                .with(BandContent(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::MiddleLeft,
                                    Vec2::new(16.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    ..default()
                                })
                                .child(
                                    Object::new("Title")
                                        .with(Text::new("Qanat"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Hint")
                                        .with(Text::new("Bring the blue spring's water down to the village"))
                                        .with(TextFont::from_font_size(13.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Keys")
                                        .with(Text::new("Click a cell: lay or turn a tile  |  Space turn  |  Z undo  |  Enter open  |  R restart"))
                                        .with(TextFont::from_font_size(11.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Status")
                                        .with(StatusText)
                                        .with(Text::new("Tiles 0  |  par 12"))
                                        .with(TextFont::from_font_size(18.0))
                                        .with(TextColor(STATUS)),
                                ),
                        )
                        .child(
                            Object::new("Buttons")
                                .with(BandContent(Band::Header))
                                .with(RectTransform::anchored(
                                    Anchor::MiddleRight,
                                    Vec2::new(-52.0, 0.0),
                                    Vec2::new(0.0, 0.0),
                                ))
                                .with(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(8),
                                    ..default()
                                })
                                .child(
                                    Object::new("Open Button")
                                        .with(Button)
                                        .with(QanatButton(QanatAction::Open))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PRIMARY))
                                        .child(
                                            Object::new("Open Label")
                                                .with(Text::new("Open Sluice"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(PRIMARY_TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Undo Button")
                                        .with(Button)
                                        .with(QanatButton(QanatAction::Undo))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Undo Label")
                                                .with(Text::new("Undo"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Clear Button")
                                        .with(Button)
                                        .with(QanatButton(QanatAction::Clear))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Clear Label")
                                                .with(Text::new("Clear"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Restart Button")
                                        .with(Button)
                                        .with(QanatButton(QanatAction::Restart))
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Restart Label")
                                                .with(Text::new("Restart"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                )
                                .child(
                                    Object::new("Back Button")
                                        .with(Button)
                                        .with(ExitToMenuButton)
                                        .with(Node {
                                            padding: UiRect::axes(px(10), px(6)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(GHOST))
                                        .child(
                                            Object::new("Back Label")
                                                .with(Text::new("Back to Menu"))
                                                .with(TextFont::from_font_size(13.0))
                                                .with(TextColor(TEXT)),
                                        ),
                                ),
                        )
                        .child(
                            Object::new("Header Toggle")
                                .with(Button)
                                .with(BandToggle(Band::Header))
                                // Top-right corner, 10px in and 8px down. The engine folds the
                                // pivot into the layout box, so the hit-box matches where it draws.
                                .with(RectTransform::anchored(
                                    Anchor::TopRight,
                                    Vec2::new(-10.0, -8.0),
                                    Vec2::new(36.0, 28.0),
                                ))
                                .with(Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
                                .with(BackgroundColor(GHOST))
                                .child(
                                    Object::new("Header Toggle Label")
                                        .with(BandToggleLabel(Band::Header))
                                        .with(Text::new("-"))
                                        .with(TextFont::from_font_size(16.0))
                                        .with(TextColor(TEXT)),
                                ),
                        ),
                )
                // The banner covers the whole screen when the room is cleared.
                .child(
                    Object::new("Banner")
                        .with(ResultBanner)
                        .with(RectTransform::stretch(0.0, 0.0, 0.0, 0.0))
                        .with(Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)))
                        .with(Visibility::Hidden)
                        .child(
                            Object::new("Banner Text")
                                .with(ResultText)
                                .with(Text::new("THE VILLAGE HAS WATER"))
                                .with(TextFont::from_font_size(28.0))
                                .with(TextColor(Color::WHITE)),
                        ),
                ),
        ],
    }
}
