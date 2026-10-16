//! The main menu, as a scene: the game mode taxonomy as a grid (sides x win condition, Solo x
//! Role structurally empty) and a row for the kinds outside the grid. Every tile is a button
//! (`LevelButton`) that starts that game's level; a game not built yet shows *Coming soon* and
//! opens its empty scene. GDDs: `__docs__/gdd/README.md`.
//!
//! Scenes are plain data (no helper functions) so the editor can read, show and edit them.

use xerxes_engine::prelude::*;

const BACKDROP: Color = Color::srgb(0.067, 0.086, 0.114);
const PANEL: Color = Color::srgb(0.090, 0.122, 0.157);
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
                    Object::new("Grid")
                        .with(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        })
                        .child(
                            Object::new("Grid Head")
                                .with(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(6),
                                    ..default()
                                })
                                .child(
                                    Object::new("Corner")
                                        .with(Node {
                                            width: px(96),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Corner Text")
                                                .with(Text::new("SIDES / WIN"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::new("Head Score")
                                        .with(Node {
                                            width: px(196),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Head Score Text")
                                                .with(Text::new("SCORE"))
                                                .with(TextFont::from_font_size(12.0))
                                                .with(TextColor(AMBER)),
                                        ),
                                )
                                .child(
                                    Object::new("Head Objective")
                                        .with(Node {
                                            width: px(196),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Head Objective Text")
                                                .with(Text::new("OBJECTIVE"))
                                                .with(TextFont::from_font_size(12.0))
                                                .with(TextColor(AMBER)),
                                        ),
                                )
                                .child(
                                    Object::new("Head Role")
                                        .with(Node {
                                            width: px(196),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Head Role Text")
                                                .with(Text::new("ROLE"))
                                                .with(TextFont::from_font_size(12.0))
                                                .with(TextColor(AMBER)),
                                        ),
                                )
                                .child(
                                    Object::new("Head Survival")
                                        .with(Node {
                                            width: px(196),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
                                        .child(
                                            Object::new("Head Survival Text")
                                                .with(Text::new("SURVIVAL"))
                                                .with(TextFont::from_font_size(12.0))
                                                .with(TextColor(AMBER)),
                                        ),
                                ),
                        )
                        .child(
                            Object::new("Row Solo Tiles")
                                .with(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(6),
                                    ..default()
                                })
                                .child(
                                    Object::new("Row Solo")
                                        .with(Node {
                                            width: px(96),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
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
                                    Object::new("Chapar Tile")
                                        .with(Button)
                                        .with(LevelButton(2))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PANEL))
                                        .with(BorderColor::all(AMBER))
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
                                    Object::new("Qanat Tile")
                                        .with(Button)
                                        .with(LevelButton(1))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PANEL))
                                        .with(BorderColor::all(AMBER))
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
                                    Object::new("Solo Role Empty")
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BorderColor::all(LINE))
                                        .child(
                                            Object::new("Solo Role Empty Label")
                                                .with(Text::new("Structurally empty"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        ),
                                )
                                .child(
                                    Object::new("Ahriman's Horde Tile")
                                        .with(Button)
                                        .with(LevelButton(0))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
                                        .with(BackgroundColor(PANEL))
                                        .with(BorderColor::all(AMBER))
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
                            Object::new("Row Coop Tiles")
                                .with(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(6),
                                    ..default()
                                })
                                .child(
                                    Object::new("Row Coop")
                                        .with(Node {
                                            width: px(96),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
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
                                    Object::new("Tombak Duo Tile")
                                        .with(Button)
                                        .with(LevelButton(3))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Bound Bros Tile")
                                        .with(Button)
                                        .with(LevelButton(4))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Behistun Tile")
                                        .with(Button)
                                        .with(LevelButton(5))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Naft Tile")
                                        .with(Button)
                                        .with(LevelButton(6))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                            Object::new("Row Duel Tiles")
                                .with(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(6),
                                    ..default()
                                })
                                .child(
                                    Object::new("Row Duel")
                                        .with(Node {
                                            width: px(96),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
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
                                    Object::new("Sofreh Tile")
                                        .with(Button)
                                        .with(LevelButton(7))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Siege of Sardis Tile")
                                        .with(Button)
                                        .with(LevelButton(8))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Karvan Tile")
                                        .with(Button)
                                        .with(LevelButton(9))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Mobarezeh Tile")
                                        .with(Button)
                                        .with(LevelButton(10))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                            Object::new("Row N-gon Tiles")
                                .with(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(6),
                                    ..default()
                                })
                                .child(
                                    Object::new("Row N-gon")
                                        .with(Node {
                                            width: px(96),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        })
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
                                    Object::new("Zahhak Tile")
                                        .with(Button)
                                        .with(LevelButton(11))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Kashi Tile")
                                        .with(Button)
                                        .with(LevelButton(12))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("The False Smerdis Tile")
                                        .with(Button)
                                        .with(LevelButton(13))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                                    Object::new("Palang Tile")
                                        .with(Button)
                                        .with(LevelButton(14))
                                        .with(Node {
                                            width: px(196),
                                            height: px(58),
                                            flex_direction: FlexDirection::Column,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::axes(px(10), px(6)),
                                            row_gap: px(2),
                                            border: UiRect::all(px(1)),
                                            border_radius: BorderRadius::all(px(6)),
                                            ..default()
                                        })
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
                    Object::new("Outside Tiles")
                        .with(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .child(
                            Object::new("Jamshid's Workshop Tile")
                                .with(Button)
                                .with(LevelButton(15))
                                .with(Node {
                                    width: px(146),
                                    height: px(58),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::axes(px(10), px(6)),
                                    row_gap: px(2),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
                                .with(BackgroundColor(PANEL))
                                .with(BorderColor::all(AMBER))
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
                            Object::new("Caravanserai Tile")
                                .with(Button)
                                .with(LevelButton(16))
                                .with(Node {
                                    width: px(146),
                                    height: px(58),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::axes(px(10), px(6)),
                                    row_gap: px(2),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
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
                            Object::new("Naqqal's Teahouse Tile")
                                .with(Button)
                                .with(LevelButton(17))
                                .with(Node {
                                    width: px(146),
                                    height: px(58),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::axes(px(10), px(6)),
                                    row_gap: px(2),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
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
                            Object::new("Persepolis Tile")
                                .with(Button)
                                .with(LevelButton(18))
                                .with(Node {
                                    width: px(146),
                                    height: px(58),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::axes(px(10), px(6)),
                                    row_gap: px(2),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
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
                            Object::new("Zurkhaneh Cup Tile")
                                .with(Button)
                                .with(LevelButton(19))
                                .with(Node {
                                    width: px(146),
                                    height: px(58),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::axes(px(10), px(6)),
                                    row_gap: px(2),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
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
                            Object::new("Maydan Tile")
                                .with(Button)
                                .with(LevelButton(20))
                                .with(Node {
                                    width: px(146),
                                    height: px(58),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::axes(px(10), px(6)),
                                    row_gap: px(2),
                                    border: UiRect::all(px(1)),
                                    border_radius: BorderRadius::all(px(6)),
                                    ..default()
                                })
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
