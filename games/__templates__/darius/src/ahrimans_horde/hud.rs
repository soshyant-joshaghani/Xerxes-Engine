//! HUD: lives / loadout / depth, pause, banner. Pad labels are updated in `combat`.

use bevy::prelude::*;
use xerxes_engine::modules::flow::Flow;
use xerxes_engine::modules::matches::{ActiveMatch, MatchPause, Phase};
use xerxes_engine::modules::ui::TouchSeen;

use super::combat::{Run, loadout_line, run_status};
use super::world::{Action, DARIC_COUNT};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (press_buttons, refresh_hud, touch_hint).run_if(in_state(Flow::Level)),
        );
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ScoreText;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PauseLabel;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Banner;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BannerText;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HintText;

#[derive(Component, Debug, Clone, Copy)]
pub struct HudButton(pub Action);

fn touch_hint(seen: Res<TouchSeen>, mut hint: Single<&mut Text, With<HintText>>) {
    if seen.is_changed() && seen.0 {
        hint.0 = "Stick move  ·  Fire / Switch / Reload-Dash  ·  D-Pad toggle in header".into();
    }
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
    run: Option<Res<Run>>,
    mut score_text: Single<&mut Text, (With<ScoreText>, Without<PauseLabel>, Without<BannerText>)>,
    mut pause_label: Single<&mut Text, (With<PauseLabel>, Without<ScoreText>, Without<BannerText>)>,
    mut banner_text: Single<&mut Text, (With<BannerText>, Without<ScoreText>, Without<PauseLabel>)>,
    mut banner: Single<&mut Visibility, With<Banner>>,
) {
    let Some(active) = active else {
        return;
    };
    let m = &active.0;
    let (lives, depth, _mag, cleared) = run
        .as_ref()
        .map(|r| run_status(r))
        .unwrap_or((3, 0, 6, false));
    let guns = run
        .as_ref()
        .map(|r| loadout_line(r))
        .unwrap_or_else(|| ">Pistol 6/6∞".into());
    let darics = m.score(0).min(DARIC_COUNT as i64);
    score_text.0 = format!("Lives {lives}  ·  Depth {depth}  ·  Darics {darics}\n{guns}");
    pause_label.0 = if pause.0 { "Resume" } else { "Pause" }.into();

    let message = match m.phase() {
        Phase::Countdown(left) => format!("GET READY  {}", left.div_ceil(60)),
        Phase::Results if cleared => "COURTYARD CLEARED  ·  WARPED".into(),
        Phase::Results if lives == 0 => "RUN OVER".into(),
        Phase::Results => format!("{darics} DARICS"),
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
