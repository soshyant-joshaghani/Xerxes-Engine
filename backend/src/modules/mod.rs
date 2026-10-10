//! The backend is two series of modules in one binary today, built to be cut apart later:
//!
//! - [`api`]: the HTTP/JSON API (auth, users, system and engine services, product modules).
//! - [`multiplayer`]: rooms and realtime (WebSocket). Long-lived connections load a server very
//!   differently from request/response traffic, so this series is meant to become its own
//!   container when the load calls for it.
//!
//! The rules that keep the cut cheap (a test enforces the first one):
//!
//! 1. `multiplayer` never imports `api`, and `api` never imports `multiplayer`. What both need
//!    (config, database, cache, jobs, security, errors, state) lives in `crate::core`.
//! 2. Each series exposes one `router(..)` and mounts under its own prefix (`/api/v1/...`,
//!    `/multiplayer/...`), so a reverse proxy can send each prefix to its own container.
//! 3. `SERVICES=all|api|multiplayer` picks what a process serves (default `all`). Splitting is
//!    then two containers of the same image with `SERVICES=api` and `SERVICES=multiplayer`;
//!    a dedicated `multiplayer` bin and image come after, when it needs its own dependencies.

pub mod api;
pub mod multiplayer;
