//! The main menu, as a scene: the game mode taxonomy as a grid (sides x win condition, Solo x
//! Role structurally empty) and a row for the kinds outside the grid. Every tile is a button
//! (`LevelButton`) that starts that game's level; a game not built yet shows *Coming soon* and
//! opens its empty scene. GDDs: `__docs__/gdd/README.md`.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const BACKDROP: Color = Color::srgb(0.067, 0.086, 0.114);
const PANEL_SOON: Color = Color::srgb(0.075, 0.098, 0.125);
const LINE: Color = Color::srgb(0.169, 0.208, 0.259);
const TEXT: Color = Color::srgb(0.914, 0.902, 0.863);
const DIM: Color = Color::srgb(0.596, 0.631, 0.675);
const FAINT: Color = Color::srgb(0.373, 0.416, 0.463);
const AMBER: Color = Color::srgb(0.878, 0.659, 0.243);
const AMBER_DIM: Color = Color::srgb(0.541, 0.416, 0.180);

pub fn scene() -> SceneAsset {
    SceneAsset {
        dimension: Dimension::D2,
        objects: vec![
            Object::new("Camera").with(GameCamera2D),
            Object::new("Menu")
                .with(Canvas::overlay())
                .with(Node {
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: px(10),
                    padding: UiRect::all(px(16)),
                    ..default()
                })
                .with(BackgroundColor(BACKDROP))
                .child(
                    Object::new("Title")
                        .with(Text::new("Darius"))
                        .with(TextFont::from_font_size(34.0))
                        .with(TextColor(TEXT)),
                )
                .child(
                    Object::new("Tagline")
                        .with(Text::new("Every game mode, in 2D"))
                        .with(TextFont::from_font_size(13.0))
                        .with(TextColor(DIM)),
                )
                .child(
                    Object::new("Rooms")
                        .with(crate::rooms::RoomListText)
                        .with(Text::new("Rooms: checking..."))
                        .with(TextFont::from_font_size(12.0))
                        .with(TextColor(DIM)),
                )
                .child(
                    Object::new("Grid")
                        .with(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        })
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_strip_prefab::prefab)
                                .named("Grid Head")
                                .child(
                                    Object::prefab(crate::assets::prefabs::menu_row_prefab::prefab)
                                        .named("Corner")
                                        .child(
                                            Object::new("Corner Text")
                                                .with(Text::new("SIDES / WIN"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_head_prefab::prefab,
                                    )
                                    .named("Head Score")
                                    .child(
                                        Object::new("Head Score Text")
                                            .with(Text::new("SCORE"))
                                            .with(TextFont::from_font_size(12.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_head_prefab::prefab,
                                    )
                                    .named("Head Objective")
                                    .child(
                                        Object::new("Head Objective Text")
                                            .with(Text::new("OBJECTIVE"))
                                            .with(TextFont::from_font_size(12.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_head_prefab::prefab,
                                    )
                                    .named("Head Role")
                                    .child(
                                        Object::new("Head Role Text")
                                            .with(Text::new("ROLE"))
                                            .with(TextFont::from_font_size(12.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_head_prefab::prefab,
                                    )
                                    .named("Head Survival")
                                    .child(
                                        Object::new("Head Survival Text")
                                            .with(Text::new("SURVIVAL"))
                                            .with(TextFont::from_font_size(12.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_strip_prefab::prefab)
                                .named("Row Solo Tiles")
                                .child(
                                    Object::prefab(crate::assets::prefabs::menu_row_prefab::prefab)
                                        .named("Row Solo")
                                        .child(
                                            Object::new("Row Solo Text")
                                                .with(Text::new("SOLO"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(AMBER)),
                                        )
                                        .child(
                                            Object::new("Row Solo Sub")
                                                .with(Text::new("one player"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Chapar Tile")
                                    .with(LevelButton(2))
                                    .child(
                                        Object::new("Chapar Name")
                                            .with(Text::new("Chapar"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(TEXT)),
                                    )
                                    .child(
                                        Object::new("Chapar Mode")
                                            .with(Text::new("Solo / Score"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Chapar Status")
                                            .with(Text::new("PLAY"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Qanat Tile")
                                    .with(LevelButton(1))
                                    .child(
                                        Object::new("Qanat Name")
                                            .with(Text::new("Qanat"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(TEXT)),
                                    )
                                    .child(
                                        Object::new("Qanat Mode")
                                            .with(Text::new("Solo / Objective"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Qanat Status")
                                            .with(Text::new("PLAY"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_empty_prefab::prefab,
                                    )
                                    .named("Solo Role Empty")
                                    .child(
                                        Object::new("Solo Role Empty Label")
                                            .with(Text::new("Structurally empty"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Ahriman's Horde Tile")
                                    .with(LevelButton(0))
                                    .child(
                                        Object::new("Ahriman's Horde Name")
                                            .with(Text::new("Ahriman's Horde"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(TEXT)),
                                    )
                                    .child(
                                        Object::new("Ahriman's Horde Mode")
                                            .with(Text::new("Solo / Survival"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Ahriman's Horde Status")
                                            .with(Text::new("PLAY"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER)),
                                    ),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_strip_prefab::prefab)
                                .named("Row Coop Tiles")
                                .child(
                                    Object::prefab(crate::assets::prefabs::menu_row_prefab::prefab)
                                        .named("Row Coop")
                                        .child(
                                            Object::new("Row Coop Text")
                                                .with(Text::new("COOP"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(AMBER)),
                                        )
                                        .child(
                                            Object::new("Row Coop Sub")
                                                .with(Text::new("one side, shared fate"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Tombak Duo Tile")
                                    .with(LevelButton(3))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Tombak Duo Name")
                                            .with(Text::new("Tombak Duo"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Tombak Duo Mode")
                                            .with(Text::new("Coop / Score"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Tombak Duo Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Bound Bros Tile")
                                    .with(LevelButton(4))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Bound Bros Name")
                                            .with(Text::new("Bound Bros"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Bound Bros Mode")
                                            .with(Text::new("Coop / Objective"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Bound Bros Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Behistun Tile")
                                    .with(LevelButton(5))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Behistun Name")
                                            .with(Text::new("Behistun"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Behistun Mode")
                                            .with(Text::new("Coop / Role"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Behistun Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Naft Tile")
                                    .with(LevelButton(6))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Naft Name")
                                            .with(Text::new("Naft"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Naft Mode")
                                            .with(Text::new("Coop / Survival"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Naft Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_strip_prefab::prefab)
                                .named("Row Duel Tiles")
                                .child(
                                    Object::prefab(crate::assets::prefabs::menu_row_prefab::prefab)
                                        .named("Row Duel")
                                        .child(
                                            Object::new("Row Duel Text")
                                                .with(Text::new("DUEL"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(AMBER)),
                                        )
                                        .child(
                                            Object::new("Row Duel Sub")
                                                .with(Text::new("two sides"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Sofreh Tile")
                                    .with(LevelButton(7))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Sofreh Name")
                                            .with(Text::new("Sofreh"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Sofreh Mode")
                                            .with(Text::new("Duel / Score"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Sofreh Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Siege of Sardis Tile")
                                    .with(LevelButton(8))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Siege of Sardis Name")
                                            .with(Text::new("Siege of Sardis"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Siege of Sardis Mode")
                                            .with(Text::new("Duel / Objective"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Siege of Sardis Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Karvan Tile")
                                    .with(LevelButton(9))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Karvan Name")
                                            .with(Text::new("Karvan"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Karvan Mode")
                                            .with(Text::new("Duel / Role"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Karvan Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Mobarezeh Tile")
                                    .with(LevelButton(10))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Mobarezeh Name")
                                            .with(Text::new("Mobarezeh"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Mobarezeh Mode")
                                            .with(Text::new("Duel / Survival"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Mobarezeh Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_strip_prefab::prefab)
                                .named("Row N-gon Tiles")
                                .child(
                                    Object::prefab(crate::assets::prefabs::menu_row_prefab::prefab)
                                        .named("Row N-gon")
                                        .child(
                                            Object::new("Row N-gon Text")
                                                .with(Text::new("N-GON"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(AMBER)),
                                        )
                                        .child(
                                            Object::new("Row N-gon Sub")
                                                .with(Text::new("three or more sides"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Zahhak Tile")
                                    .with(LevelButton(11))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Zahhak Name")
                                            .with(Text::new("Zahhak"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Zahhak Mode")
                                            .with(Text::new("N-gon / Score"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Zahhak Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Kashi Tile")
                                    .with(LevelButton(12))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Kashi Name")
                                            .with(Text::new("Kashi"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Kashi Mode")
                                            .with(Text::new("N-gon / Objective"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Kashi Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("The False Smerdis Tile")
                                    .with(LevelButton(13))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("The False Smerdis Name")
                                            .with(Text::new("The False Smerdis"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("The False Smerdis Mode")
                                            .with(Text::new("N-gon / Role"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("The False Smerdis Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                )
                                .child(
                                    Object::prefab(
                                        crate::assets::prefabs::menu_tile_prefab::prefab,
                                    )
                                    .named("Palang Tile")
                                    .with(LevelButton(14))
                                    .with(BackgroundColor(PANEL_SOON))
                                    .with(BorderColor::all(LINE))
                                    .child(
                                        Object::new("Palang Name")
                                            .with(Text::new("Palang"))
                                            .with(TextFont::from_font_size(14.0))
                                            .with(TextColor(DIM)),
                                    )
                                    .child(
                                        Object::new("Palang Mode")
                                            .with(Text::new("N-gon / Survival"))
                                            .with(TextFont::from_font_size(10.0))
                                            .with(TextColor(FAINT)),
                                    )
                                    .child(
                                        Object::new("Palang Status")
                                            .with(Text::new("COMING SOON"))
                                            .with(TextFont::from_font_size(9.0))
                                            .with(TextColor(AMBER_DIM)),
                                    ),
                                ),
                        ),
                )
                .child(
                    Object::new("Outside Label")
                        .with(Text::new("OUTSIDE THE GRID"))
                        .with(TextFont::from_font_size(10.0))
                        .with(TextColor(FAINT)),
                )
                .child(
                    Object::prefab(crate::assets::prefabs::menu_strip_prefab::prefab)
                        .named("Outside Tiles")
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_tile_narrow_prefab::prefab)
                                .named("Jamshid's Workshop Tile")
                                .with(LevelButton(15))
                                .child(
                                    Object::new("Jamshid's Workshop Name")
                                        .with(Text::new("Jamshid's Workshop"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Jamshid's Workshop Mode")
                                        .with(Text::new("Sandbox"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Jamshid's Workshop Status")
                                        .with(Text::new("PLAY"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER)),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_tile_narrow_prefab::prefab)
                                .named("Caravanserai Tile")
                                .with(LevelButton(16))
                                .with(BackgroundColor(PANEL_SOON))
                                .with(BorderColor::all(LINE))
                                .child(
                                    Object::new("Caravanserai Name")
                                        .with(Text::new("Caravanserai"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Caravanserai Mode")
                                        .with(Text::new("Progression"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Caravanserai Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_tile_narrow_prefab::prefab)
                                .named("Naqqal's Teahouse Tile")
                                .with(LevelButton(17))
                                .with(BackgroundColor(PANEL_SOON))
                                .with(BorderColor::all(LINE))
                                .child(
                                    Object::new("Naqqal's Teahouse Name")
                                        .with(Text::new("Naqqal's Teahouse"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Naqqal's Teahouse Mode")
                                        .with(Text::new("Narrative"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Naqqal's Teahouse Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_tile_narrow_prefab::prefab)
                                .named("Persepolis Tile")
                                .with(LevelButton(18))
                                .with(BackgroundColor(PANEL_SOON))
                                .with(BorderColor::all(LINE))
                                .child(
                                    Object::new("Persepolis Name")
                                        .with(Text::new("Persepolis"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Persepolis Mode")
                                        .with(Text::new("Composite"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Persepolis Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_tile_narrow_prefab::prefab)
                                .named("Zurkhaneh Cup Tile")
                                .with(LevelButton(19))
                                .with(BackgroundColor(PANEL_SOON))
                                .with(BorderColor::all(LINE))
                                .child(
                                    Object::new("Zurkhaneh Cup Name")
                                        .with(Text::new("Zurkhaneh Cup"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Zurkhaneh Cup Mode")
                                        .with(Text::new("Tournament"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Zurkhaneh Cup Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::prefab(crate::assets::prefabs::menu_tile_narrow_prefab::prefab)
                                .named("Maydan Tile")
                                .with(LevelButton(20))
                                .with(BackgroundColor(PANEL_SOON))
                                .with(BorderColor::all(LINE))
                                .child(
                                    Object::new("Maydan Name")
                                        .with(Text::new("Maydan"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Maydan Mode")
                                        .with(Text::new("Live session"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Maydan Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        ),
                ),
        ],
    }
}
