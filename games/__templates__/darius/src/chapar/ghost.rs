//! Chapar's ghost: the best run of the day, ridden again beside you.
//!
//! A run is fully decided by its seed and the [`Input`] of every tick, so a ghost is nothing but
//! those inputs: a second [`Run`] on the same road, fed the recorded inputs in step with the
//! player's own run. Nothing about its position is stored, so it can never drift from what the
//! rules would do, and a recording is a few kilobytes of text at most.
//!
//! A recording is one line, `chapar1 <day> <metres> <hex>`: the day it was ridden on (its road),
//! how far it got, and the inputs as (flags, repeat) byte pairs in hex. Only one recording is
//! kept: the best of the latest day played.

use super::rules::{Input, Run};

/// The format tag at the start of a saved recording.
const TAG: &str = "chapar1";

/// One tick's input as flag bits.
pub fn pack(input: Input) -> u8 {
    u8::from(input.jump_pressed) | u8::from(input.jump_held) << 1 | u8::from(input.duck_held) << 2
}

/// The input a flag byte stands for.
pub fn unpack(flags: u8) -> Input {
    Input {
        jump_pressed: flags & 1 != 0,
        jump_held: flags & 2 != 0,
        duck_held: flags & 4 != 0,
    }
}

/// A recorded run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    /// The day whose road it was ridden on (days since 1970-01-01 UTC).
    pub day: u64,
    /// How far it got, in metres.
    pub metres: u32,
    /// One packed [`Input`] per fixed tick.
    pub inputs: Vec<u8>,
}

impl Recording {
    /// The recording as one line of text.
    pub fn encode(&self) -> String {
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
        format!("{TAG} {} {} {hex}", self.day, self.metres)
    }

    /// A recording from its text, or `None` if it is not one (an older format, a damaged file).
    pub fn decode(text: &str) -> Option<Self> {
        let mut parts = text.split_whitespace();
        if parts.next()? != TAG {
            return None;
        }
        let day = parts.next()?.parse().ok()?;
        let metres = parts.next()?.parse().ok()?;
        let hex = parts.next().unwrap_or("");
        if hex.len() % 4 != 0 || parts.next().is_some() {
            return None;
        }
        let mut inputs = Vec::new();
        for pair in hex.as_bytes().chunks(4) {
            let flags = u8::from_str_radix(std::str::from_utf8(&pair[..2]).ok()?, 16).ok()?;
            let count = u8::from_str_radix(std::str::from_utf8(&pair[2..]).ok()?, 16).ok()?;
            if count == 0 || flags > 7 {
                return None;
            }
            inputs.extend(std::iter::repeat_n(flags, count as usize));
        }
        Some(Self {
            day,
            metres,
            inputs,
        })
    }
}

/// A recording being ridden: a run on the same road fed the recorded inputs.
#[derive(Debug, Clone)]
pub struct Ghost {
    run: Run,
    inputs: Vec<u8>,
    metres: u32,
}

impl Ghost {
    /// The ghost of `recording` on the road of `seed`.
    pub fn new(seed: u64, recording: &Recording) -> Self {
        Self {
            run: Run::new(seed),
            inputs: recording.inputs.clone(),
            metres: recording.metres,
        }
    }

    /// Advances one fixed tick, in step with the player's own run. Once the recording is used
    /// up the ghost stands where its rider ended.
    pub fn step(&mut self) {
        let tick = self.run.ticks() as usize;
        if let Some(&flags) = self.inputs.get(tick) {
            self.run.step(unpack(flags));
        }
    }

    /// The ghost's run, for drawing (its position, height, ducking).
    pub fn run(&self) -> &Run {
        &self.run
    }

    /// How far the recorded run got in the end.
    pub fn final_metres(&self) -> u32 {
        self.metres
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapar::rules::TICKS_PER_SECOND;

    /// Plays a seed with a fixed pattern of presses (not smart: it ends at some obstacle) and
    /// returns the run and its recorded inputs.
    fn ride(seed: u64) -> (Run, Vec<u8>) {
        let mut run = Run::new(seed);
        let mut inputs = Vec::new();
        for t in 0..20 * TICKS_PER_SECOND {
            if run.is_over() {
                break;
            }
            let input = Input {
                jump_pressed: t % 67 == 0,
                jump_held: t % 67 < 22,
                duck_held: t % 131 > 100,
            };
            inputs.push(pack(input));
            run.step(input);
        }
        (run, inputs)
    }

    #[test]
    fn inputs_pack_into_flags_and_back() {
        for flags in 0..8u8 {
            assert_eq!(pack(unpack(flags)), flags);
        }
    }

    #[test]
    fn a_recording_survives_its_text() {
        let (run, inputs) = ride(5);
        let rec = Recording {
            day: 20_000,
            metres: run.metres(),
            inputs,
        };
        let text = rec.encode();
        assert!(text.starts_with("chapar1 20000 "));
        assert!(!text.contains('\n'));
        assert_eq!(Recording::decode(&text), Some(rec));
    }

    #[test]
    fn long_holds_are_stored_as_repeats() {
        let rec = Recording {
            day: 1,
            metres: 0,
            inputs: vec![2; 1000],
        };
        // 1000 ticks of one input: four byte pairs (255 + 255 + 255 + 235).
        assert_eq!(rec.encode(), "chapar1 1 0 02ff02ff02ff02eb");
        assert_eq!(Recording::decode(&rec.encode()), Some(rec));
    }

    #[test]
    fn damaged_text_is_not_a_recording() {
        for text in [
            "",
            "chapar2 1 2 0001",
            "chapar1 x 2 0001",
            "chapar1 1 2 000",
            "chapar1 1 2 zz01",
            "chapar1 1 2 0900",
            "chapar1 1 2 0001 extra",
        ] {
            assert_eq!(Recording::decode(text), None, "{text:?}");
        }
    }

    #[test]
    fn the_ghost_rides_exactly_the_recorded_run() {
        let (run, inputs) = ride(9);
        let rec = Recording {
            day: 1,
            metres: run.metres(),
            inputs,
        };
        let mut ghost = Ghost::new(9, &rec);
        for _ in 0..rec.inputs.len() + 50 {
            ghost.step();
        }
        // The same place, the same end, and it stays there however long it is stepped.
        assert_eq!(ghost.run().x(), run.x());
        assert_eq!(ghost.run().phase(), run.phase());
        assert_eq!(ghost.final_metres(), run.metres());
    }
}
