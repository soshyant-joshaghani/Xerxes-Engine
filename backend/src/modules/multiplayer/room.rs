//! A room: one task that owns its clients, relays their messages and tells them who joined or left.
//! M0 keeps the room's rules to membership: a room has a maximum number of clients, and a message
//! goes to every other client unchanged. The game's rules come in M1 (see `backend-multiplayer.md`).

use std::collections::HashMap;
use std::sync::Mutex;

use tokio::sync::mpsc;

use super::protocol::{self, Frame};

/// The most clients a room takes when it is made without a size.
pub const DEFAULT_MAX_CLIENTS: usize = 8;
/// How many frames may wait for a client before the room stops sending to it.
const OUTBOX: usize = 32;

/// A connected player, as the room sees it: an id and where to send its frames.
#[derive(Debug, Clone)]
pub struct Client {
    pub id: u64,
    pub outbox: mpsc::Sender<Frame>,
}

/// What a room is asked to do.
#[derive(Debug)]
enum Command {
    Join(Client),
    Leave(u64),
    Message { from: u64, frame: Frame },
}

/// The way to talk to a room. Cheap to clone; every clone reaches the same room task.
#[derive(Debug, Clone)]
pub struct RoomHandle {
    pub id: String,
    commands: mpsc::Sender<Command>,
}

impl RoomHandle {
    /// Asks to join. The room answers on the client's outbox: `joined`, or `error` when it is full.
    pub async fn join(&self, client: Client) {
        let _ = self.commands.send(Command::Join(client)).await;
    }

    /// Leaves the room; the others are told.
    pub async fn leave(&self, client: u64) {
        let _ = self.commands.send(Command::Leave(client)).await;
    }

    /// Sends a message from a client to the room.
    pub async fn message(&self, from: u64, frame: Frame) {
        let _ = self.commands.send(Command::Message { from, frame }).await;
    }
}

/// Starts the task of a room with `max_clients` seats and returns the handle to it.
pub fn spawn(id: &str, max_clients: usize) -> RoomHandle {
    let (commands, mut inbox) = mpsc::channel::<Command>(64);
    tokio::spawn(async move {
        let mut clients: HashMap<u64, mpsc::Sender<Frame>> = HashMap::new();
        while let Some(command) = inbox.recv().await {
            match command {
                Command::Join(client) => {
                    if clients.len() >= max_clients {
                        let _ = client
                            .outbox
                            .send(Frame::new(protocol::ERROR, "the room is full"))
                            .await;
                        continue;
                    }
                    let _ = client
                        .outbox
                        .send(Frame::new(protocol::JOINED, protocol::id_payload(client.id)))
                        .await;
                    let joined = Frame::new(protocol::PEER_JOINED, protocol::id_payload(client.id));
                    for other in clients.values() {
                        let _ = other.send(joined.clone()).await;
                    }
                    clients.insert(client.id, client.outbox);
                }
                Command::Leave(id) => {
                    if clients.remove(&id).is_some() {
                        let left = Frame::new(protocol::PEER_LEFT, protocol::id_payload(id));
                        for other in clients.values() {
                            let _ = other.send(left.clone()).await;
                        }
                    }
                }
                Command::Message { from, frame } => {
                    for (id, outbox) in &clients {
                        if *id != from {
                            let _ = outbox.send(frame.clone()).await;
                        }
                    }
                }
            }
        }
    });
    RoomHandle {
        id: id.to_string(),
        commands,
    }
}

/// The rooms of this process, by id. M0 keeps them in memory, one process (see the deployment
/// notes in `backend-multiplayer.md`).
#[derive(Debug, Default)]
pub struct Rooms {
    rooms: Mutex<HashMap<String, RoomHandle>>,
}

impl Rooms {
    /// The room with this id, made with `max_clients` if it did not exist yet.
    pub fn get_or_create(&self, id: &str, max_clients: usize) -> RoomHandle {
        let mut rooms = self.rooms.lock().expect("the room list lock");
        rooms
            .entry(id.to_string())
            .or_insert_with(|| spawn(id, max_clients))
            .clone()
    }

    /// The ids of the rooms that exist.
    pub fn ids(&self) -> Vec<String> {
        let rooms = self.rooms.lock().expect("the room list lock");
        let mut ids: Vec<String> = rooms.keys().cloned().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(id: u64) -> (Client, mpsc::Receiver<Frame>) {
        let (outbox, inbox) = mpsc::channel(OUTBOX);
        (Client { id, outbox }, inbox)
    }

    #[tokio::test]
    async fn a_joiner_is_confirmed_and_the_others_are_told() {
        let room = spawn("lobby", DEFAULT_MAX_CLIENTS);
        let (a, mut a_in) = client(1);
        let (b, mut b_in) = client(2);
        room.join(a).await;
        room.join(b).await;
        assert_eq!(a_in.recv().await.unwrap().name, protocol::JOINED);
        assert_eq!(b_in.recv().await.unwrap().name, protocol::JOINED);
        let told = a_in.recv().await.unwrap();
        assert_eq!(told.name, protocol::PEER_JOINED);
        assert_eq!(protocol::id_from_payload(&told.payload), Some(2));
    }

    #[tokio::test]
    async fn a_message_goes_to_the_others_and_not_back_to_the_sender() {
        let room = spawn("chat", DEFAULT_MAX_CLIENTS);
        let (a, mut a_in) = client(1);
        let (b, mut b_in) = client(2);
        room.join(a).await;
        room.join(b).await;
        // Drain the join traffic: a's confirmation and its notice of b.
        a_in.recv().await;
        a_in.recv().await;
        b_in.recv().await;
        room.message(1, Frame::new(protocol::MESSAGE, b"hi".to_vec())).await;
        let got = b_in.recv().await.unwrap();
        assert_eq!(got, Frame::new(protocol::MESSAGE, b"hi".to_vec()));
        room.leave(2).await;
        // a hears nothing of its own message: the next thing it gets is b leaving.
        let next = a_in.recv().await.unwrap();
        assert_eq!(next.name, protocol::PEER_LEFT);
    }

    #[tokio::test]
    async fn a_full_room_refuses_the_next_client() {
        let room = spawn("small", 1);
        let (a, mut a_in) = client(1);
        let (b, mut b_in) = client(2);
        room.join(a).await;
        room.join(b).await;
        assert_eq!(a_in.recv().await.unwrap().name, protocol::JOINED);
        assert_eq!(b_in.recv().await.unwrap().name, protocol::ERROR);
    }

    #[tokio::test]
    async fn rooms_by_id_are_the_same_room() {
        let rooms = Rooms::default();
        let one = rooms.get_or_create("a", 2);
        let again = rooms.get_or_create("a", 99);
        assert_eq!(one.id, again.id);
        rooms.get_or_create("b", 2);
        assert_eq!(rooms.ids(), vec!["a".to_string(), "b".to_string()]);
    }
}
