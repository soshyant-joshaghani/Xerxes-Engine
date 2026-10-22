//! The Xerxes engine app on Windows, macOS, Linux and the web: the editor where games will
//! be created. Android starts from `platform/android/main.rs`.

fn main() {
    xerxes_engine::editor::launch();
}
