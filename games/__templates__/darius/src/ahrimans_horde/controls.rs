//! Ahriman's Horde's movement controls: a floating virtual joystick or a four-way D-pad. A button in
//! the header switches between them; both controls live in the footer band of the scene
//! (`ahrimans_horde.scene.rs`) and feed the `move` action (stick `source("move")`, D-pad
//! `TouchButton("dpad_*")`). This module only shows one of them at a time.

use bevy::prelude::*;
use xerxes_engine::modules::flow::Flow;
use xerxes_engine::modules::ui::{Bands, VirtualJoystick};

/// Which control the footer shows. Default: the joystick.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MoveMode {
    pub dpad: bool,
}

/// The D-pad cluster (shown in D-pad mode).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Dpad;

/// The button that switches between the joystick and the D-pad.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ControlSwitch;

/// The switch button's label: this module writes the control it switches *to*.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ControlSwitchLabel;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveMode>().add_systems(
            Update,
            (switch_control, show_control)
                .chain()
                .run_if(in_state(Flow::Level)),
        );
    }
}

/// A press on the switch button flips the control.
fn switch_control(
    pressed: Query<&Interaction, (Changed<Interaction>, With<ControlSwitch>)>,
    mut mode: ResMut<MoveMode>,
) {
    for interaction in &pressed {
        if *interaction == Interaction::Pressed {
            mode.dpad = !mode.dpad;
        }
    }
}

/// Shows the selected control (and neither when the footer is collapsed), and labels the switch.
#[allow(clippy::type_complexity)]
fn show_control(
    mode: Res<MoveMode>,
    bands: Res<Bands>,
    mut dpad: Query<&mut Node, (With<Dpad>, Without<VirtualJoystick>)>,
    mut stick: Query<&mut Node, (With<VirtualJoystick>, Without<Dpad>)>,
    mut label: Query<&mut Text, With<ControlSwitchLabel>>,
) {
    let open = !bands.footer_collapsed;
    let show = |want: bool, node: &mut Node| {
        let display = if want { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    };
    for mut node in &mut dpad {
        show(open && mode.dpad, &mut node);
    }
    for mut node in &mut stick {
        show(open && !mode.dpad, &mut node);
    }
    for mut text in &mut label {
        let wanted = if mode.dpad { "Stick" } else { "D-Pad" };
        if text.0 != wanted {
            text.0 = wanted.into();
        }
    }
}
