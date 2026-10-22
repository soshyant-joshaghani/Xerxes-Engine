//! The client socket of the room runtime (M0, see `__docs__/backend-multiplayer.md`).
//!
//! Native: a background thread owns a `tungstenite` socket and passes frames over channels, so the
//! game never waits on the network. Only `ws://` is supported (`wss://` needs TLS, not built yet).
//!
//! Web: not built yet. [`connect`] reports that, until the socket is checked in a browser.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use super::protocol::{self, Frame};

/// A joined room, from the game's side: send frames, and poll for what the room sent back.
pub struct Connection {
    outgoing: Sender<Frame>,
    incoming: Receiver<Frame>,
}

impl Connection {
    /// Sends a frame to the room. `false` when the connection is gone.
    pub fn send(&self, frame: Frame) -> bool {
        self.outgoing.send(frame).is_ok()
    }

    /// The frames the room sent since the last call, oldest first.
    pub fn poll(&self) -> Vec<Frame> {
        let mut frames = Vec::new();
        loop {
            match self.incoming.try_recv() {
                Ok(frame) => frames.push(frame),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        frames
    }
}

/// Opens a socket to `url` (`ws://host:port/multiplayer/ws`) and joins `room`.
#[cfg(not(target_arch = "wasm32"))]
pub fn connect(url: &str, room: &str) -> Result<Connection, String> {
    use std::time::Duration;

    use tungstenite::Message;
    use tungstenite::stream::MaybeTlsStream;

    if !url.starts_with("ws://") {
        return Err("only ws:// is supported for now (wss:// needs TLS)".into());
    }
    let (mut socket, _) = tungstenite::connect(url).map_err(|e| e.to_string())?;
    // A short read timeout, so the thread can also send while it waits for the room.
    match socket.get_mut() {
        MaybeTlsStream::Plain(tcp) => tcp
            .set_read_timeout(Some(Duration::from_millis(10)))
            .map_err(|e| e.to_string())?,
        _ => return Err("only a plain ws:// socket can be polled".into()),
    }
    let join = Frame::new(protocol::JOIN, room.as_bytes().to_vec());
    socket
        .send(Message::Binary(join.encode().into()))
        .map_err(|e| e.to_string())?;

    let (outgoing, from_game) = mpsc::channel::<Frame>();
    let (to_game, incoming) = mpsc::channel::<Frame>();
    std::thread::spawn(move || pump(socket, from_game, to_game));
    Ok(Connection { outgoing, incoming })
}

/// Moves frames both ways until either side is gone.
#[cfg(not(target_arch = "wasm32"))]
fn pump(
    mut socket: tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    from_game: Receiver<Frame>,
    to_game: Sender<Frame>,
) {
    use tungstenite::Message;

    loop {
        loop {
            match from_game.try_recv() {
                Ok(frame) => {
                    if socket.send(Message::Binary(frame.encode().into())).is_err() {
                        return;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = socket.close(None);
                    return;
                }
            }
        }
        match socket.read() {
            Ok(Message::Binary(bytes)) => {
                if let Ok(frame) = Frame::decode(&bytes) {
                    if to_game.send(frame).is_err() {
                        return;
                    }
                }
            }
            Ok(Message::Close(_)) => return,
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return,
        }
    }
}

/// Asks a server which rooms exist: one `list` request, one `rooms` answer, then the socket closes.
/// Waits at most two seconds for the answer.
#[cfg(not(target_arch = "wasm32"))]
pub fn list_rooms(url: &str) -> Result<Vec<String>, String> {
    use std::time::Duration;

    use tungstenite::Message;
    use tungstenite::stream::MaybeTlsStream;

    if !url.starts_with("ws://") {
        return Err("only ws:// is supported for now (wss:// needs TLS)".into());
    }
    let (mut socket, _) = tungstenite::connect(url).map_err(|e| e.to_string())?;
    if let MaybeTlsStream::Plain(tcp) = socket.get_mut() {
        tcp.set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
    }
    let list = Frame::new(protocol::LIST, Vec::new());
    socket
        .send(Message::Binary(list.encode().into()))
        .map_err(|e| e.to_string())?;
    loop {
        match socket.read().map_err(|e| e.to_string())? {
            Message::Binary(bytes) => {
                let frame = Frame::decode(&bytes).map_err(|e| e.to_string())?;
                if frame.name == protocol::ROOMS {
                    let text = String::from_utf8(frame.payload).map_err(|e| e.to_string())?;
                    return Ok(text
                        .split('\n')
                        .filter(|id| !id.is_empty())
                        .map(str::to_string)
                        .collect());
                }
            }
            Message::Close(_) => return Err("the server closed before it listed the rooms".into()),
            _ => {}
        }
    }
}

/// Web: not built yet, as for [`connect`].
#[cfg(target_arch = "wasm32")]
pub fn list_rooms(_url: &str) -> Result<Vec<String>, String> {
    Err("the web socket is not built yet".into())
}

/// Web: not built yet. A browser socket needs checking in a browser before it ships.
#[cfg(target_arch = "wasm32")]
pub fn connect(_url: &str, _room: &str) -> Result<Connection, String> {
    Err("the web socket is not built yet".into())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use tungstenite::Message;

    use super::*;

    /// A one-room server: it reads the join, confirms it, and echoes the first message back.
    fn serve_once() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
        let addr = listener.local_addr().expect("the address");
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("a client");
            let mut ws = tungstenite::accept(stream).expect("a handshake");
            let join = ws.read().expect("the join");
            let frame = Frame::decode(&join.into_data()).expect("a frame");
            assert_eq!(frame.name, protocol::JOIN);
            assert_eq!(frame.payload, b"lobby".to_vec());
            let joined = Frame::new(protocol::JOINED, protocol::id_payload(7));
            ws.send(Message::Binary(joined.encode().into())).expect("the confirmation");
            loop {
                match ws.read() {
                    Ok(Message::Binary(bytes)) => {
                        let echo = Frame::decode(&bytes).expect("a frame");
                        ws.send(Message::Binary(echo.encode().into())).expect("the echo");
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });
        format!("ws://{addr}")
    }

    /// Polls until a frame arrives, or gives up after five seconds.
    fn next(conn: &Connection) -> Option<Frame> {
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if let Some(frame) = conn.poll().into_iter().next() {
                return Some(frame);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }

    #[test]
    fn a_client_joins_a_room_and_its_messages_come_back() {
        let conn = connect(&serve_once(), "lobby").expect("a connection");
        let joined = next(&conn).expect("the confirmation");
        assert_eq!(joined.name, protocol::JOINED);
        assert_eq!(protocol::id_from_payload(&joined.payload), Some(7));

        assert!(conn.send(Frame::new(protocol::MESSAGE, b"hi".to_vec())));
        let echo = next(&conn).expect("the echo");
        assert_eq!(echo, Frame::new(protocol::MESSAGE, b"hi".to_vec()));
    }

    /// A server that answers a `list` with two room ids and closes.
    fn serve_list() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
        let addr = listener.local_addr().expect("the address");
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("a client");
            let mut ws = tungstenite::accept(stream).expect("a handshake");
            let list = ws.read().expect("the list request");
            assert_eq!(Frame::decode(&list.into_data()).expect("a frame").name, protocol::LIST);
            let rooms = Frame::new(protocol::ROOMS, b"lobby\narena".to_vec());
            ws.send(Message::Binary(rooms.encode().into())).expect("the answer");
            let _ = ws.close(None);
            let _ = ws.flush();
        });
        format!("ws://{addr}")
    }

    #[test]
    fn list_rooms_reads_the_ids_the_server_has() {
        let rooms = list_rooms(&serve_list()).expect("the rooms");
        assert_eq!(rooms, vec!["lobby".to_string(), "arena".to_string()]);
    }

    #[test]
    fn wss_is_refused_for_now() {
        assert!(connect("wss://example.invalid/multiplayer/ws", "lobby").is_err());
    }
}
