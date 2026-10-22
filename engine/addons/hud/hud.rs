//! Template: HUD (bevy_ui). Attach `Hud` to an object and a panel appears with the title,
//! a hint line, Pause/Resume and Menu buttons, plus a PAUSED banner while paused. Its UI
//! leaves with the level (`LevelEntity`).
//! needs: pause

use xerxes_engine::prelude::*;

use super::pause::Paused;

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Hud {
    pub title: &'static str,
    pub hint: &'static str,
}

const PANEL: Color = Color::srgba(0.047, 0.055, 0.071, 0.72);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const PRIMARY: Color = Color::srgb(0.949, 0.455, 0.122);
const PRIMARY_TEXT: Color = Color::srgb(0.067, 0.067, 0.067);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_panel, press_buttons, show_paused));
}

#[derive(Component)]
struct PauseButton;

#[derive(Component)]
struct MenuButton;

#[derive(Component)]
struct PauseLabel;

#[derive(Component)]
struct Banner;

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

/// UI is its own root (not a child of the object), so it lays out against the window.
fn spawn_panel(mut commands: Commands, huds: Query<&Hud, Added<Hud>>) {
    for hud in &huds {
        // The banner first, so the panel (and its button) draws and picks above it.
        commands.spawn((
            LevelEntity,
            Banner,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            Visibility::Hidden,
            children![text("PAUSED", 28.0, Color::WHITE)],
        ));
        commands.spawn((
            LevelEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: px(16),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(8),
                padding: UiRect::axes(px(14), px(12)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(PANEL),
            children![
                text(hud.title, 14.0, TEXT),
                text(hud.hint, 13.0, TEXT),
                (
                    Node {
                        column_gap: px(8),
                        ..default()
                    },
                    children![
                        (
                            Button,
                            PauseButton,
                            Node {
                                padding: UiRect::axes(px(10), px(6)),
                                border_radius: BorderRadius::all(px(6)),
                                ..default()
                            },
                            BackgroundColor(PRIMARY),
                            children![(PauseLabel, text("Pause", 13.0, PRIMARY_TEXT))],
                        ),
                        (
                            Button,
                            MenuButton,
                            Node {
                                padding: UiRect::axes(px(10), px(6)),
                                border_radius: BorderRadius::all(px(6)),
                                ..default()
                            },
                            BackgroundColor(GHOST),
                            children![text("Menu", 13.0, TEXT)],
                        ),
                    ],
                ),
            ],
        ));
    }
}

fn press_buttons(
    pause: Query<&Interaction, (With<PauseButton>, Changed<Interaction>)>,
    menu: Query<&Interaction, (With<MenuButton>, Changed<Interaction>)>,
    mut paused: ResMut<Paused>,
    mut exit: MessageWriter<ExitToMenu>,
) {
    if pause.iter().any(|i| *i == Interaction::Pressed) {
        paused.0 = !paused.0;
    }
    if menu.iter().any(|i| *i == Interaction::Pressed) {
        exit.write(ExitToMenu);
    }
}

fn show_paused(
    paused: Res<Paused>,
    mut label: Query<&mut Text, With<PauseLabel>>,
    mut banner: Query<&mut Visibility, With<Banner>>,
    added: Query<(), Added<PauseLabel>>,
) {
    if !paused.is_changed() && added.is_empty() {
        return;
    }
    for mut label in &mut label {
        label.0 = if paused.0 { "Resume" } else { "Pause" }.into();
    }
    for mut visibility in &mut banner {
        *visibility = if paused.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
