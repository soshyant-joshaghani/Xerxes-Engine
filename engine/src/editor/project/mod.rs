//! The open project, as the editor sees it: where projects live (`store`: the disk natively,
//! the backend on the web through `client`), the `.rs` asset codec (`codec`), and the naming
//! rules the editor checks as you type (`names`).

pub mod browse;
#[cfg(target_arch = "wasm32")]
pub mod client;
pub mod codec;
pub mod jobs;
pub mod names;
pub mod settings_text;
pub mod store;
pub mod table;
