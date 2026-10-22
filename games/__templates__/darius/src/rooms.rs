//! The room list on the main menu (M0 of the online track). Each time the menu opens, the game asks
//! the backend which rooms are live (`list` over the room socket, see `backend-multiplayer.md`) on a
//! background thread, and writes the answer into the menu's Rooms line. Nothing waits on the network.
//!
//! The web has no socket yet, so there the line says so. A build with no backend says that instead.

use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use bevy::prelude::*;
use xerxes_engine::modules::flow::Flow;
use xerxes_engine::modules::multiplayer::client::list_rooms;
use xerxes_engine::modules::project::GameBackend;

/// The menu's Rooms line.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RoomListText;

/// The answer on its way from the background thread.
#[derive(Resource)]
struct Pending(Mutex<Receiver<Result<Vec<String>, String>>>);

/// The room socket's address for a backend's HTTP address: `http://h:p` becomes `ws://h:p/multiplayer/ws`.
/// `None` for anything that is not an HTTP address.
pub fn ws_url(http: &str) -> Option<String> {
    let base = if let Some(rest) = http.strip_prefix("https://") {
        format!("wss://{rest}")
    } else {
        format!("ws://{}", http.strip_prefix("http://")?)
    };
    Some(format!("{}/multiplayer/ws", base.trim_end_matches('/')))
}

/// Adds the room list (called by `src/logic.rs`). It acts only where the menu has its Rooms line.
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Flow::MainMenu), request)
        .add_systems(Update, show.run_if(resource_exists::<Pending>));
}

/// Starts the question, or says why it cannot be asked.
fn request(
    mut commands: Commands,
    backend: Option<Res<GameBackend>>,
    mut lines: Query<&mut Text, With<RoomListText>>,
) {
    let Some(url) = backend.and_then(|b| b.0).and_then(ws_url) else {
        set_line(&mut lines, "Rooms: no backend in this build".into());
        return;
    };
    let (answer, waiting) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = answer.send(list_rooms(&url));
    });
    commands.insert_resource(Pending(Mutex::new(waiting)));
}

/// Writes the answer once it has come back, and forgets the question.
fn show(
    mut commands: Commands,
    pending: Res<Pending>,
    mut lines: Query<&mut Text, With<RoomListText>>,
) {
    let answer = {
        let waiting = pending.0.lock().expect("the room list lock");
        match waiting.try_recv() {
            Ok(answer) => answer,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("the question was lost".to_string()),
        }
    };
    commands.remove_resource::<Pending>();
    let line = match answer {
        Ok(ids) if ids.is_empty() => "Rooms: none live yet".to_string(),
        Ok(ids) => format!("Rooms: {}", ids.join(", ")),
        Err(reason) => format!("Rooms: {reason}"),
    };
    set_line(&mut lines, line);
}

fn set_line(lines: &mut Query<&mut Text, With<RoomListText>>, line: String) {
    for mut text in lines.iter_mut() {
        **text = line.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_room_socket_is_the_backend_address_with_the_socket_path() {
        assert_eq!(
            ws_url("http://127.0.0.1:8000").as_deref(),
            Some("ws://127.0.0.1:8000/multiplayer/ws")
        );
        assert_eq!(
            ws_url("https://api.example.com/").as_deref(),
            Some("wss://api.example.com/multiplayer/ws")
        );
        assert_eq!(ws_url("127.0.0.1:8000"), None);
    }

    #[test]
    fn the_menu_has_a_rooms_line() {
        let mut world = World::new();
        xerxes_engine::modules::assets::spawn_scene(
            &mut world,
            &crate::assets::scenes::menu_scene::scene(),
        );
        assert_eq!(world.query::<&RoomListText>().iter(&world).count(), 1);
    }
}
