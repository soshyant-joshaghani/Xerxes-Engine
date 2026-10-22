//! The game's logic plugins. Each one plugs a game's code (`src/<game>/`) into the project, and the
//! project file lists them in its `LOGIC`. Logic is project-wide, so each plugin's systems only act
//! on a level whose scene has their markers.

use bevy::prelude::*;

/// Arash's Arrow: 3D channel diorama (slice 1). Acts only on a level with GateCells.
pub fn arashs_arrow(app: &mut App) {
    crate::arashs_arrow::plugin(app);
}

/// Siavash: the knight's run on a three-lane road. Acts only on a level with a Knight.
pub fn siavash(app: &mut App) {
    crate::siavash::plugin(app);
}

/// Pairidaeza: 3D physics lab. Acts only on a level with a Plot.
pub fn pairidaeza(app: &mut App) {
    crate::pairidaeza::plugin(app);
}
