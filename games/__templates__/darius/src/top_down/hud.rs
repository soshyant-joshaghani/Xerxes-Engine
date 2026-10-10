//! The HUD in bevy_ui: title, controls, the coin count, Pause/Resume and Restart buttons, and
//! a banner when paused or won. It reads the world's state and asks for changes with [`Action`].

use bevy::prelude::*;
use xerxes_engine::modules::flow::{Flow, LevelEntity};
use xerxes_engine::modules::matches::{ActiveMatch, MatchPause, Outcome, Phase};

use super::world::{Action, COIN_COUNT};

const PANEL: Color = Color::srgba(0.047, 0.055, 0.071, 0.72);
const TEXT: Color = Color::srgb(0.91, 0.918, 0.929);
const SCORE: Color = Color::srgb(0.984, 0.827, 0.302);
const PRIMARY: Color = Color::srgb(0.22, 0.741, 0.973);
const PRIMARY_TEXT: Color = Color::srgb(0.043, 0.071, 0.125);
const GHOST: Color = Color::srgb(0.165, 0.184, 0.216);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Flow::Level), spawn_hud)
            .add_systems(
                Update,
                (press_buttons, refresh_hud).run_if(in_state(Flow::Level)),
            );
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct PauseLabel;

#[derive(Component)]
struct Banner;

#[derive(Component)]
struct BannerText;

/// What a HUD button asks the world to do.
#[derive(Component, Clone, Copy)]
struct HudButton(Action);

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

fn button(action: Action, background: Color) -> impl Bundle {
    (
        Button,
        HudButton(action),
        Node {
            padding: UiRect::axes(px(10), px(6)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(background),
    )
}

fn spawn_hud(mut commands: Commands) {
    // The banner first, so the panel (and its buttons) draws and picks above it.
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
        children![(BannerText, text("", 28.0, Color::WHITE))],
    ));

    commands.spawn((
        LevelEntity,
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::axes(px(14), px(12)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(PANEL),
        children![
            text("Darius", 14.0, TEXT),
            text("WASD / arrows to move · Esc to pause", 13.0, TEXT),
            (ScoreText, text("", 18.0, SCORE)),
            (
                Node {
                    column_gap: px(8),
                    ..default()
                },
                children![
                    (
                        button(Action::TogglePause, PRIMARY),
                        children![(PauseLabel, text("Pause", 13.0, PRIMARY_TEXT))]
                    ),
                    (
                        button(Action::Restart, GHOST),
                        children![text("Restart", 13.0, TEXT)]
                    ),
                ],
            ),
        ],
    ));
}

fn press_buttons(
    buttons: Query<(&Interaction, &HudButton), Changed<Interaction>>,
    mut actions: MessageWriter<Action>,
) {
    for (interaction, HudButton(action)) in &buttons {
        if *interaction == Interaction::Pressed {
            actions.write(*action);
        }
    }
}

fn refresh_hud(
    pause: Res<MatchPause>,
    active: Option<Res<ActiveMatch>>,
    mut score_text: Single<&mut Text, (With<ScoreText>, Without<PauseLabel>, Without<BannerText>)>,
    mut pause_label: Single<&mut Text, (With<PauseLabel>, Without<ScoreText>, Without<BannerText>)>,
    mut banner_text: Single<&mut Text, (With<BannerText>, Without<ScoreText>, Without<PauseLabel>)>,
    mut banner: Single<&mut Visibility, With<Banner>>,
) {
    let Some(active) = active else {
        return;
    };
    let m = &active.0;
    let seconds = m.time_left().map_or(0, |t| t.div_ceil(60));
    score_text.0 = format!(
        "Coins {} / {COIN_COUNT}   {seconds}s",
        m.progress(0).min(COIN_COUNT as u32)
    );
    pause_label.0 = if pause.0 { "Resume" } else { "Pause" }.into();

    let message = match m.phase() {
        Phase::Countdown(left) => format!("GET READY  {}", left.div_ceil(60)),
        Phase::Results => match m.result().map(|r| r.standings[0].outcome) {
            Some(Outcome::Won) => "ALL COINS COLLECTED".into(),
            _ => "TIME IS UP".into(),
        },
        _ if pause.0 => "PAUSED".into(),
        _ => String::new(),
    };
    **banner = if message.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    banner_text.0 = message;
}
