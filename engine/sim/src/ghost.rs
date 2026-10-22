//! Ghosts: the best run, ridden again beside the player (step 6 of the plan).
//!
//! A run is decided by its key (the road: a seed, or a day) and one input per tick, so a ghost is a
//! second run on the same road fed the recorded inputs. Nothing about its position is stored, so it
//! cannot drift from what the rules would do. Chapar's ghost was the first; Siavash's is the second,
//! so the code lives here.
//!
//! A recording is one line of text: `<tag> <key> <score> <hex>`. The tag names the game's format,
//! the key is the road, the score is how far the run got, and the inputs are (flags, repeat) byte
//! pairs in hex, one flag byte per tick. Each game packs its own input into those flags.

/// A run that takes one input per tick.
pub trait Sim {
    type Input: Copy;

    /// Advances the run one fixed tick with this input.
    fn step(&mut self, input: Self::Input);
}

/// A recorded run: its road, how far it got, and one packed input per tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    pub key: u64,
    pub score: u32,
    pub inputs: Vec<u8>,
}

impl Recording {
    /// The recording as one line of text, under `tag`.
    pub fn encode(&self, tag: &str) -> String {
        let mut hex = String::new();
        let mut i = 0;
        while i < self.inputs.len() {
            let flags = self.inputs[i];
            let mut count = 1;
            while count < 255 && i + count < self.inputs.len() && self.inputs[i + count] == flags {
                count += 1;
            }
            hex.push_str(&format!("{flags:02x}{count:02x}"));
            i += count;
        }
        format!("{tag} {} {} {hex}", self.key, self.score)
    }

    /// A recording from its text, or `None` if it is not one under `tag` (another game's format, a
    /// damaged file).
    pub fn decode(tag: &str, text: &str) -> Option<Self> {
        let mut parts = text.split_whitespace();
        if parts.next()? != tag {
            return None;
        }
        let key = parts.next()?.parse().ok()?;
        let score = parts.next()?.parse().ok()?;
        let hex = parts.next().unwrap_or("");
        if hex.len() % 4 != 0 || parts.next().is_some() {
            return None;
        }
        let mut inputs = Vec::new();
        for pair in hex.as_bytes().chunks(4) {
            let flags = u8::from_str_radix(std::str::from_utf8(&pair[..2]).ok()?, 16).ok()?;
            let count = u8::from_str_radix(std::str::from_utf8(&pair[2..]).ok()?, 16).ok()?;
            if count == 0 {
                return None;
            }
            inputs.extend(std::iter::repeat_n(flags, count as usize));
        }
        Some(Self { key, score, inputs })
    }
}

/// A recording being ridden: a run on the same road, fed the recorded inputs in step with the
/// player's own run. Once the recording is used up, the ghost stands where its rider ended.
#[derive(Debug, Clone)]
pub struct Ghost<S: Sim> {
    sim: S,
    inputs: Vec<u8>,
    unpack: fn(u8) -> S::Input,
    next: usize,
}

impl<S: Sim> Ghost<S> {
    /// The ghost of `recording`, on the run `sim` (the same road), with `unpack` reading the flags.
    pub fn new(sim: S, recording: &Recording, unpack: fn(u8) -> S::Input) -> Self {
        Self {
            sim,
            inputs: recording.inputs.clone(),
            unpack,
            next: 0,
        }
    }

    /// Advances one fixed tick, in step with the player's own run.
    pub fn step(&mut self) {
        if let Some(&flags) = self.inputs.get(self.next) {
            self.sim.step((self.unpack)(flags));
            self.next += 1;
        }
    }

    /// The ghost's run, for drawing.
    pub fn sim(&self) -> &S {
        &self.sim
    }

    /// Whether the recording has been used up.
    pub fn finished(&self) -> bool {
        self.next >= self.inputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Recording {
        Recording {
            key: 42,
            score: 17,
            inputs: vec![0, 0, 0, 5, 5, 0, 7, 1],
        }
    }

    #[test]
    fn a_recording_round_trips_under_its_tag() {
        let text = sample().encode("cyrus1");
        assert_eq!(text, "cyrus1 42 17 00030502000107010101");
        assert_eq!(Recording::decode("cyrus1", &text), Some(sample()));
    }

    #[test]
    fn another_games_recording_is_refused() {
        let text = sample().encode("chapar1");
        assert_eq!(Recording::decode("cyrus1", &text), None);
        assert_eq!(Recording::decode("cyrus1", "cyrus1 1 2 zz"), None);
        assert_eq!(Recording::decode("cyrus1", "cyrus1 1 2 0000"), None, "a zero repeat");
    }

    /// A run that counts what it was fed.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Counter(Vec<u8>);

    impl Sim for Counter {
        type Input = u8;
        fn step(&mut self, input: u8) {
            self.0.push(input);
        }
    }

    #[test]
    fn a_ghost_is_fed_its_recording_in_step_and_then_stands() {
        let mut ghost = Ghost::new(Counter(Vec::new()), &sample(), |flags| flags);
        assert!(!ghost.finished());
        for _ in 0..20 {
            ghost.step();
        }
        assert!(ghost.finished());
        assert_eq!(ghost.sim().0, sample().inputs, "fed exactly the recorded inputs, then nothing");
    }
}
