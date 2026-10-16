//! The day's road: today's date (UTC) is the seed, so everyone who rides today rides the same
//! road, and tomorrow's is another. The clock and the calendar are the engine's
//! (`xerxes_engine::modules::clock`); this only gives Chapar's roads their own salt.

use xerxes_engine::modules::clock::{self, Date};

/// A salt, so Chapar's roads are not another game's "day N" road.
const SALT: u64 = 0x43_48_41_50_41_52; // "CHAPAR"

/// Today's day number (days since 1970-01-01 UTC).
pub fn today() -> u64 {
    clock::today()
}

/// The seed of a day's road.
pub fn seed_of_day(day: u64) -> u64 {
    clock::daily_seed(day, SALT)
}

/// The day as `YYYY-MM-DD`.
pub fn date_label(day: u64) -> String {
    Date::from_day(day).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_road_of_a_day_is_its_own_and_stays_the_same_all_day() {
        assert_eq!(seed_of_day(20_000), seed_of_day(20_000));
        assert_ne!(seed_of_day(20_000), seed_of_day(20_001));
        assert_eq!(date_label(19_782), "2024-02-29");
    }
}
