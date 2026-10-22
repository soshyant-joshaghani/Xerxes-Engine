//! The wire format of the room runtime (M0, see `__docs__/backend-multiplayer.md`).
//!
//! A frame is one binary WebSocket message: a name, then a binary payload.
//!
//! | byte | meaning |
//! |---|---|
//! | 0 | the length of the name, in bytes (at most [`MAX_NAME`]) |
//! | 1 .. | the name, UTF-8 |
//! | after the name | the payload, any bytes |
//!
//! Every message name is listed here, in one place, and the client bridge uses the same list.

use std::fmt;

/// The longest name a frame can carry.
pub const MAX_NAME: usize = 64;

/// A client asks to join a room. Payload: the room id (UTF-8).
pub const JOIN: &str = "join";
/// A client asks which rooms exist. Empty payload; the server answers with `rooms` and closes.
pub const LIST: &str = "list";
/// The answer to `list`. Payload: the room ids, one per line (UTF-8).
pub const ROOMS: &str = "rooms";
/// The server confirms a join. Payload: the client's session id (8 bytes, big-endian).
pub const JOINED: &str = "joined";
/// The server refuses a join or a message. Payload: a reason (UTF-8).
pub const ERROR: &str = "error";
/// A client's message. The room relays it to the other clients unchanged.
pub const MESSAGE: &str = "message";
/// Another client joined the room. Payload: their session id (8 bytes, big-endian).
pub const PEER_JOINED: &str = "peer_joined";
/// Another client left the room. Payload: their session id (8 bytes, big-endian).
pub const PEER_LEFT: &str = "peer_left";

/// One message: a name and its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub name: String,
    pub payload: Vec<u8>,
}

/// Why a frame could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    Empty,
    Truncated,
    NameTooLong(usize),
    NotUtf8,
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => write!(f, "an empty frame"),
            FrameError::Truncated => write!(f, "a frame shorter than its name"),
            FrameError::NameTooLong(n) => write!(f, "a name of {n} bytes (at most {MAX_NAME})"),
            FrameError::NotUtf8 => write!(f, "a name that is not UTF-8"),
        }
    }
}

impl Frame {
    pub fn new(name: &str, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.to_string(),
            payload: payload.into(),
        }
    }

    /// The frame as the bytes of one binary message.
    pub fn encode(&self) -> Vec<u8> {
        let name = self.name.as_bytes();
        let mut out = Vec::with_capacity(1 + name.len() + self.payload.len());
        out.push(name.len().min(MAX_NAME) as u8);
        out.extend_from_slice(&name[..name.len().min(MAX_NAME)]);
        out.extend_from_slice(&self.payload);
        out
    }

    /// Reads one binary message into a frame.
    pub fn decode(bytes: &[u8]) -> Result<Self, FrameError> {
        let (&len, rest) = bytes.split_first().ok_or(FrameError::Empty)?;
        let len = len as usize;
        if len > MAX_NAME {
            return Err(FrameError::NameTooLong(len));
        }
        if rest.len() < len {
            return Err(FrameError::Truncated);
        }
        let name = std::str::from_utf8(&rest[..len]).map_err(|_| FrameError::NotUtf8)?;
        Ok(Self {
            name: name.to_string(),
            payload: rest[len..].to_vec(),
        })
    }
}

/// A session id as the payload of a frame.
pub fn id_payload(id: u64) -> Vec<u8> {
    id.to_be_bytes().to_vec()
}

/// The session id a payload carries (see [`id_payload`]).
pub fn id_from_payload(payload: &[u8]) -> Option<u64> {
    let bytes: [u8; 8] = payload.try_into().ok()?;
    Some(u64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_round_trips_with_its_binary_payload() {
        let frame = Frame::new(MESSAGE, vec![0, 1, 2, 255]);
        let bytes = frame.encode();
        assert_eq!(bytes[0] as usize, MESSAGE.len());
        assert_eq!(Frame::decode(&bytes), Ok(frame));
    }

    #[test]
    fn a_bad_frame_is_refused_not_guessed() {
        assert_eq!(Frame::decode(&[]), Err(FrameError::Empty));
        assert_eq!(Frame::decode(&[5, b'a']), Err(FrameError::Truncated));
        assert_eq!(Frame::decode(&[200]), Err(FrameError::NameTooLong(200)));
        assert_eq!(Frame::decode(&[2, 0xff, 0xfe]), Err(FrameError::NotUtf8));
    }

    /// The same vector as the engine's `protocol.rs`: a join for room "lobby". If either side's
    /// layout changes, this and the engine's test fail together.
    #[test]
    fn the_join_frame_has_the_agreed_bytes() {
        let frame = Frame::new(JOIN, b"lobby".to_vec());
        assert_eq!(frame.encode(), vec![4, b'j', b'o', b'i', b'n', b'l', b'o', b'b', b'b', b'y']);
    }

    #[test]
    fn a_session_id_is_eight_bytes_and_comes_back() {
        assert_eq!(id_from_payload(&id_payload(42)), Some(42));
        assert_eq!(id_from_payload(&[1, 2, 3]), None);
    }
}
