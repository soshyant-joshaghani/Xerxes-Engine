//! The Xerxes simulation core: game rules and physics with no Bevy in them.
//!
//! The Bevy game, the multiplayer server and the replay viewer all run the same code from this
//! crate, stepped at a fixed rate from the same inputs and seed, so they agree. Nothing here
//! draws, reads a device or touches the clock.
//!
//! - [`mode`] the game mode taxonomy (sides x win condition, modifiers) as data.
//! - [`matches`] the match runtime: phases, sides, scoring, win conditions, rounds, modifiers.
//! - [`physics`] a 2D physics world (Rapier, deterministic mode): bodies, colliders, layers,
//!   sensors, contact events and a character mover.

pub mod matches;
pub mod mode;
pub mod physics;

pub use physics::{BodyDef, BodyId, Contact, Kind, Layers, Moved, Physics2d, Shape, V2};
