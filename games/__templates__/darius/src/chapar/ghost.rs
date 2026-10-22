//! Chapar's ghost: the best run of the day, ridden again beside you.
//!
//! The recording and the replay are the shared ones in `xerxes_sim::ghost` (step 6 of the plan:
//! Chapar was the first game to use them). What is Chapar's own is its input (a jump and a duck,
//! packed into a byte here) and the road: a recording's key is the day, and its road is the day's seed.
//!
//! A recording is one line, `chapar1 <day> <metres> <hex>`, as the shared format writes it under
//! [`TAG`]. Only one recording is kept: the best of the latest day played.

use xerxes_sim::ghost::Ghost as SharedGhost;
pub use xerxes_sim::ghost::{Recording, Sim};

use super::rules::{Input, Run};

/// The format tag at the start of a saved recording.
pub const TAG: &str = "chapar1";

/// A ghost of Chapar's run.
pub type Ghost = SharedGhost<Run>;

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

/// The ghost of `recording` on the road of `seed` (the day's seed).
pub fn ghost_of(seed: u64, recording: &Recording) -> Ghost {
    Ghost::new(Run::new(seed), recording, unpack)
}

impl Sim for Run {
    type Input = Input;

    fn step(&mut self, input: Input) {
        Run::step(self, input);
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
            key: 20_000,
            score: run.metres(),
            inputs,
        };
        let text = rec.encode(TAG);
        assert!(text.starts_with("chapar1 20000 "));
        assert!(!text.contains('\n'));
        assert_eq!(Recording::decode(TAG, &text), Some(rec));
    }

    #[test]
    fn long_holds_are_stored_as_repeats() {
        let rec = Recording {
            key: 1,
            score: 0,
            inputs: vec![2; 1000],
        };
        // 1000 ticks of one input: four byte pairs (255 + 255 + 255 + 235).
        assert_eq!(rec.encode(TAG), "chapar1 1 0 02ff02ff02ff02eb");
        assert_eq!(Recording::decode(TAG, &rec.encode(TAG)), Some(rec));
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
            assert_eq!(Recording::decode(TAG, text), None, "{text:?}");
        }
    }

    #[test]
    fn the_ghost_rides_exactly_the_recorded_run() {
        let (run, inputs) = ride(9);
        let rec = Recording {
            key: 1,
            score: run.metres(),
            inputs,
        };
        let mut ghost = ghost_of(9, &rec);
        for _ in 0..rec.inputs.len() + 50 {
            ghost.step();
        }
        // The same place, the same end, and it stays there however long it is stepped.
        assert_eq!(ghost.sim().x(), run.x());
        assert_eq!(ghost.sim().phase(), run.phase());
        assert_eq!(ghost.sim().metres(), run.metres());
    }
}
