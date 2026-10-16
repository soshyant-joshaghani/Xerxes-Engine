//! The wall clock and the calendar day, the same on every platform.
//!
//! Games need the date for things the game clock cannot give: a daily seed (Chapar's road of the
//! day), a daily reward, a "played today" flag. `std::time::SystemTime` panics on the web, so a
//! game must not call it directly; this module reads the device's clock (`Date.now()` in the
//! browser, `SystemTime` elsewhere) and turns it into days and dates, in UTC so that everyone on
//! the same day gets the same value wherever they are.
//!
//! Nothing here is a Bevy resource or system: they are plain functions, so Bevy-free code (the
//! `xerxes_sim` rules, a server) can take the same day number as an argument.

/// Seconds since 1970-01-01 00:00:00 UTC from the device's clock (0 if it cannot be read).
pub fn unix_seconds() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }
}

/// The day a time is on: whole days since 1970-01-01 UTC.
pub fn day_number(unix_seconds: u64) -> u64 {
    unix_seconds / 86_400
}

/// Today's day number (UTC), from the device's clock.
pub fn today() -> u64 {
    day_number(unix_seconds())
}

/// A seed for a day, mixed with a `salt` the game chooses so that two games (or two modes of one
/// game) do not get the same "day N" number. Neighbouring days give unrelated seeds, and the same
/// day and salt give the same seed on every device.
pub fn daily_seed(day: u64, salt: u64) -> u64 {
    // SplitMix64's finaliser over the salted day.
    let mut z = day.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A calendar date (proleptic Gregorian, UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    /// 1 to 12.
    pub month: u8,
    /// 1 to 31.
    pub day: u8,
}

impl Date {
    /// The date of a day number (days since 1970-01-01).
    pub fn from_day(day: u64) -> Self {
        let z = day as i64 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        Self {
            year: y as i32,
            month: m as u8,
            day: d as u8,
        }
    }
}

impl std::fmt::Display for Date {
    /// `YYYY-MM-DD`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_are_dates() {
        assert_eq!(Date::from_day(0).to_string(), "1970-01-01");
        assert_eq!(Date::from_day(19_723).to_string(), "2024-01-01");
        // A leap day, and the day after it.
        assert_eq!(Date::from_day(19_782).to_string(), "2024-02-29");
        assert_eq!(Date::from_day(19_783).to_string(), "2024-03-01");
        // The century year that is not a leap year: no 29 February in 2100.
        assert_eq!(Date::from_day(47_540).to_string(), "2100-02-28");
        assert_eq!(Date::from_day(47_541).to_string(), "2100-03-01");
        assert_eq!(
            Date::from_day(day_number(1_791_800_000)).to_string(),
            "2026-10-12"
        );
    }

    #[test]
    fn a_day_is_86400_seconds() {
        assert_eq!(day_number(86_399), 0);
        assert_eq!(day_number(86_400), 1);
    }

    #[test]
    fn dates_order_like_days() {
        assert!(Date::from_day(19_782) < Date::from_day(19_783));
    }

    #[test]
    fn every_day_has_its_own_seed_for_a_salt_and_it_is_the_same_all_day() {
        assert_eq!(daily_seed(20_000, 7), daily_seed(20_000, 7));
        assert_ne!(daily_seed(20_000, 7), daily_seed(20_000, 8));
        let mut seeds: Vec<u64> = (20_000..20_100).map(|d| daily_seed(d, 7)).collect();
        seeds.sort_unstable();
        seeds.dedup();
        assert_eq!(seeds.len(), 100);
    }

    #[test]
    fn the_clock_reads_a_plausible_time() {
        // After this was written (2026-10), and a date that formats.
        assert!(unix_seconds() > 1_790_000_000);
        assert_eq!(today(), day_number(unix_seconds()));
    }
}
