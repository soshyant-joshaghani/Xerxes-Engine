//! The online bridge a game calls (M0, see `__docs__/backend-multiplayer.md`): join a room, send
//! messages, receive what the room sends. The room runtime is in the backend (`/multiplayer/ws`).
//!
//! [`protocol`] is the wire format, the same as the backend's; [`client`] is the socket (native: plain
//! `ws://` with `tungstenite`; web: not built yet).

pub mod client;
pub mod protocol;
