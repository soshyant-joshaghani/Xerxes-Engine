//! The WebSocket of the room runtime at `/multiplayer/ws`. A client's first frame decides what the
//! socket is for:
//!
//! - `join` with the room id as its payload: the client joins that room. Its `message` frames go to
//!   the room, and the room's frames come back over the socket. Leaving (or dropping the socket)
//!   tells the room.
//! - `list`: the server answers with one `rooms` frame (the room ids, one per line) and closes.
//!
//! Anything else as the first frame closes the socket.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::mpsc;

use super::protocol::{self, Frame};
use super::room::{self, Client, Rooms, DEFAULT_MAX_CLIENTS};

/// The rooms of this process.
pub static ROOMS: std::sync::LazyLock<Rooms> = std::sync::LazyLock::new(Rooms::default);

/// Session ids for the clients of this process.
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

/// What a socket's first frame asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum First {
    Join(String),
    List,
}

/// Reads what a socket's first frame asks for; `None` for anything else.
pub fn first_request(bytes: &[u8]) -> Option<First> {
    let frame = Frame::decode(bytes).ok()?;
    match frame.name.as_str() {
        protocol::LIST => Some(First::List),
        protocol::JOIN => {
            let id = String::from_utf8(frame.payload).ok()?;
            (!id.is_empty()).then_some(First::Join(id))
        }
        _ => None,
    }
}

/// The rooms of a `rooms` frame's payload: one id per line.
pub fn rooms_payload(ids: &[String]) -> Vec<u8> {
    ids.join("\n").into_bytes()
}

/// Upgrades the request to a WebSocket and serves the client until it leaves.
pub async fn upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(serve)
}

async fn serve(mut socket: WebSocket) {
    let request = match socket.recv().await {
        Some(Ok(Message::Binary(bytes))) => first_request(&bytes),
        _ => None,
    };
    match request {
        Some(First::List) => {
            let frame = Frame::new(protocol::ROOMS, rooms_payload(&ROOMS.ids()));
            let _ = socket.send(Message::Binary(frame.encode().into())).await;
            let _ = socket.send(Message::Close(None)).await;
        }
        Some(First::Join(room_id)) => join_and_relay(socket, &room_id).await,
        None => {}
    }
}

/// Joins a room and relays until the client leaves.
async fn join_and_relay(mut socket: WebSocket, room_id: &str) {
    let room = ROOMS.get_or_create(room_id, DEFAULT_MAX_CLIENTS);
    let id = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
    let (outbox, mut frames) = mpsc::channel::<Frame>(32);
    room.join(Client { id, outbox }).await;

    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Binary(bytes))) => {
                    if let Ok(frame) = Frame::decode(&bytes) {
                        if frame.name == protocol::MESSAGE {
                            room.message(id, frame).await;
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            outgoing = frames.recv() => match outgoing {
                Some(frame) => {
                    if socket.send(Message::Binary(frame.encode().into())).await.is_err() {
                        break;
                    }
                }
                None => break,
            },
        }
    }
    room.leave(id).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_frame_asks_for_a_join_or_a_list() {
        let join = Frame::new(protocol::JOIN, b"lobby".to_vec()).encode();
        assert_eq!(first_request(&join), Some(First::Join("lobby".to_string())));
        let list = Frame::new(protocol::LIST, Vec::new()).encode();
        assert_eq!(first_request(&list), Some(First::List));
    }

    #[test]
    fn anything_else_first_is_refused() {
        let message = Frame::new(protocol::MESSAGE, b"hi".to_vec()).encode();
        assert_eq!(first_request(&message), None);
        let empty_room = Frame::new(protocol::JOIN, Vec::new()).encode();
        assert_eq!(first_request(&empty_room), None);
        assert_eq!(first_request(&[]), None);
    }

    #[test]
    fn the_room_list_is_one_id_per_line() {
        let ids = vec!["lobby".to_string(), "arena".to_string()];
        assert_eq!(rooms_payload(&ids), b"lobby\narena".to_vec());
        assert_eq!(rooms_payload(&[]), Vec::<u8>::new());
    }
}
