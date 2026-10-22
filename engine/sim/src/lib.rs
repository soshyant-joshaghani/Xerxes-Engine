//! The Xerxes simulation core: game rules and physics with no Bevy in them.
//!
//! The Bevy game, the multiplayer server and the replay viewer all run the same code from this
//! crate, stepped at a fixed rate from the same inputs and seed, so they agree. Nothing here
//! draws, reads a device or touches the clock.
//!
//! - [`mode`] the game mode taxonomy (sides x win condition, modifiers) as data.
//! - [`matches`] the match runtime: phases, sides, scoring, win conditions, rounds, modifiers.
//! - [`participants`] who plays each seat (local, bot, remote, spectator) and how their reports reach the match.
//! - [`qanat`] step rules for channel / sluice puzzles (Qanat; Arash's Arrow's diorama in its first slice).
//! - [`physics`] a 2D physics world (Rapier, deterministic mode): bodies, colliders, layers,
//!   sensors, contact events and a character mover.
//! - [`physics3d`] the same for 3D (Rapier3d): bodies, colliders, layers, contacts, rays.

pub mod ghost;
pub mod matches;
pub mod mode;
pub mod participants;
pub mod physics;
pub mod physics3d;
pub mod qanat;

pub use physics::{BodyDef, BodyId, Contact, Kind, Layers, Moved, Physics2d, Shape, V2};
pub use physics3d::{BodyDef3d, BodyId3d, Contact3d, Physics3d, Shape3d, V3};
