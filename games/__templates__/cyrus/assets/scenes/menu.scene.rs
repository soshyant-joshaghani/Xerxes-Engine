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
                        .with(Text::new("Cyrus"))
                        .with(TextFont::from_font_size(34.0))
                        .with(TextColor(TEXT)),
                )
                .child(
                    Object::new("Tagline")
                        .with(Text::new("Every game mode, in 3D"))
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
                                    Object::new("Rakhsh Tile")
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
                                        .with(BackgroundColor(PANEL_SOON))
                                        .with(BorderColor::all(LINE))
                                        .child(
                                            Object::new("Rakhsh Name")
                                                .with(Text::new("Rakhsh"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Rakhsh Mode")
                                                .with(Text::new("Solo / Score"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Rakhsh Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Arash's Arrow Tile")
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
                                            Object::new("Arash's Arrow Name")
                                                .with(Text::new("Arash's Arrow"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(TEXT)),
                                        )
                                        .child(
                                            Object::new("Arash's Arrow Mode")
                                                .with(Text::new("Solo / Objective"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Arash's Arrow Status")
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
                                    Object::new("Atash Tile")
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
                                        .with(BackgroundColor(PANEL_SOON))
                                        .with(BorderColor::all(LINE))
                                        .child(
                                            Object::new("Atash Name")
                                                .with(Text::new("Atash"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Atash Mode")
                                                .with(Text::new("Solo / Survival"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Atash Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
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
                                    Object::new("Nowruz Feast Tile")
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
                                            Object::new("Nowruz Feast Name")
                                                .with(Text::new("Nowruz Feast"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Nowruz Feast Mode")
                                                .with(Text::new("Coop / Score"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Nowruz Feast Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Croesus' Gold Tile")
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
                                            Object::new("Croesus' Gold Name")
                                                .with(Text::new("Croesus' Gold"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Croesus' Gold Mode")
                                                .with(Text::new("Coop / Objective"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Croesus' Gold Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Pasargadae Depths Tile")
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
                                            Object::new("Pasargadae Depths Name")
                                                .with(Text::new("Pasargadae Depths"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Pasargadae Depths Mode")
                                                .with(Text::new("Coop / Role"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Pasargadae Depths Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("The Immortals Tile")
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
                                            Object::new("The Immortals Name")
                                                .with(Text::new("The Immortals"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("The Immortals Mode")
                                                .with(Text::new("Coop / Survival"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("The Immortals Status")
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
                                    Object::new("Farr Tile")
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
                                            Object::new("Farr Name")
                                                .with(Text::new("Farr"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Farr Mode")
                                                .with(Text::new("Duel / Score"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Farr Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Derafsh Tile")
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
                                            Object::new("Derafsh Name")
                                                .with(Text::new("Derafsh"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Derafsh Mode")
                                                .with(Text::new("Duel / Objective"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Derafsh Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Atashkadeh Tile")
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
                                            Object::new("Atashkadeh Name")
                                                .with(Text::new("Atashkadeh"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Atashkadeh Mode")
                                                .with(Text::new("Duel / Role"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Atashkadeh Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Ramz Tile")
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
                                            Object::new("Ramz Name")
                                                .with(Text::new("Ramz"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Ramz Mode")
                                                .with(Text::new("Duel / Survival"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Ramz Status")
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
                                    Object::new("Kuchulu Cup Tile")
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
                                            Object::new("Kuchulu Cup Name")
                                                .with(Text::new("Kuchulu Cup"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Kuchulu Cup Mode")
                                                .with(Text::new("N-gon / Score"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Kuchulu Cup Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Arzhang Tile")
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
                                            Object::new("Arzhang Name")
                                                .with(Text::new("Arzhang"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Arzhang Mode")
                                                .with(Text::new("N-gon / Objective"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Arzhang Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Writing on the Wall Tile")
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
                                            Object::new("Writing on the Wall Name")
                                                .with(Text::new("Writing on the Wall"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Writing on the Wall Mode")
                                                .with(Text::new("N-gon / Role"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Writing on the Wall Status")
                                                .with(Text::new("COMING SOON"))
                                                .with(TextFont::from_font_size(9.0))
                                                .with(TextColor(AMBER_DIM)),
                                        ),
                                )
                                .child(
                                    Object::new("Sandstorm Tile")
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
                                            Object::new("Sandstorm Name")
                                                .with(Text::new("Sandstorm"))
                                                .with(TextFont::from_font_size(14.0))
                                                .with(TextColor(DIM)),
                                        )
                                        .child(
                                            Object::new("Sandstorm Mode")
                                                .with(Text::new("N-gon / Survival"))
                                                .with(TextFont::from_font_size(10.0))
                                                .with(TextColor(FAINT)),
                                        )
                                        .child(
                                            Object::new("Sandstorm Status")
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
                            Object::new("Pairidaeza Tile")
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
                                    Object::new("Pairidaeza Name")
                                        .with(Text::new("Pairidaeza"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(TEXT)),
                                )
                                .child(
                                    Object::new("Pairidaeza Mode")
                                        .with(Text::new("Sandbox"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Pairidaeza Status")
                                        .with(Text::new("PLAY"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER)),
                                ),
                        )
                        .child(
                            Object::new("Haft Khan Tile")
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
                                    Object::new("Haft Khan Name")
                                        .with(Text::new("Haft Khan"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Haft Khan Mode")
                                        .with(Text::new("Progression"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Haft Khan Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::new("The Dream of Astyages Tile")
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
                                    Object::new("The Dream of Astyages Name")
                                        .with(Text::new("The Dream of Astyages"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("The Dream of Astyages Mode")
                                        .with(Text::new("Narrative"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("The Dream of Astyages Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::new("Ecbatana Tile")
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
                                    Object::new("Ecbatana Name")
                                        .with(Text::new("Ecbatana"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Ecbatana Mode")
                                        .with(Text::new("Composite"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Ecbatana Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::new("Farr League Tile")
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
                                    Object::new("Farr League Name")
                                        .with(Text::new("Farr League"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Farr League Mode")
                                        .with(Text::new("Tournament"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Farr League Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        )
                        .child(
                            Object::new("Tirgan Tile")
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
                                    Object::new("Tirgan Name")
                                        .with(Text::new("Tirgan"))
                                        .with(TextFont::from_font_size(14.0))
                                        .with(TextColor(DIM)),
                                )
                                .child(
                                    Object::new("Tirgan Mode")
                                        .with(Text::new("Live session"))
                                        .with(TextFont::from_font_size(10.0))
                                        .with(TextColor(FAINT)),
                                )
                                .child(
                                    Object::new("Tirgan Status")
                                        .with(Text::new("COMING SOON"))
                                        .with(TextFont::from_font_size(9.0))
                                        .with(TextColor(AMBER_DIM)),
                                ),
                        ),
                ),
        ],
    }
}
