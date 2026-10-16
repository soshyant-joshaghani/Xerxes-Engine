//! Per-viewport gizmo config groups so each Scene view sizes handles from its own zoom.
//!
//! Layer `GIZMO_LAYER_BASE + slot` is private to that viewport camera; layer 0 stays the scene.
//! `depth_bias: -1` keeps handles in front of meshes.

use bevy::camera::visibility::RenderLayers;
use bevy::ecs::system::SystemParam;
use bevy::gizmos::config::{DefaultGizmoConfigGroup, GizmoConfig, GizmoConfigGroup};
use bevy::prelude::*;
use bevy::reflect::Reflect;

use super::camera::GIZMO_LAYER_BASE;
use crate::editor::layout::MAX_VIEWPORTS;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ViewportGizmos0;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ViewportGizmos1;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ViewportGizmos2;
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct ViewportGizmos3;

/// The four per-viewport gizmo writers as one system param (Bevy's 16-param limit).
#[derive(SystemParam)]
pub struct ViewportGizmoWriters<'w, 's> {
    pub g0: Gizmos<'w, 's, ViewportGizmos0>,
    pub g1: Gizmos<'w, 's, ViewportGizmos1>,
    pub g2: Gizmos<'w, 's, ViewportGizmos2>,
    pub g3: Gizmos<'w, 's, ViewportGizmos3>,
}

fn viewport_config(slot: usize) -> GizmoConfig {
    GizmoConfig {
        depth_bias: -1.0,
        render_layers: RenderLayers::layer(GIZMO_LAYER_BASE + slot),
        ..default()
    }
}

/// Registers the four viewport groups and biases the default group (grid, selection, frames).
pub fn configure(app: &mut App) {
    debug_assert_eq!(MAX_VIEWPORTS, 4);
    app.insert_gizmo_config(
        DefaultGizmoConfigGroup,
        GizmoConfig {
            depth_bias: -1.0,
            ..default()
        },
    )
    .insert_gizmo_config(ViewportGizmos0, viewport_config(0))
    .insert_gizmo_config(ViewportGizmos1, viewport_config(1))
    .insert_gizmo_config(ViewportGizmos2, viewport_config(2))
    .insert_gizmo_config(ViewportGizmos3, viewport_config(3));
}
