//! Strict RFC 3339 UTC timestamps with second precision.

use core::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::UpdaterError;

/// Clock-skew allowance in seconds.
pub const CLOCK_SKEW_SECONDS: i64 = 300;

/// Maximum manifest validity window in seconds (366 days).
pub const MAX_VALIDITY_SECONDS: i64 = 366 * 24 * 60 * 60;

/// Maximum trust-key validity window in seconds (10 years of 366 days).
pub const MAX_KEY_VALIDITY_SECONDS: i64 = 10 * 366 * 24 * 60 * 60;

const SECONDS_PER_DAY: i64 = 86_400;
const MIN_YEAR: i32 = 1970;
const MAX_YEAR: i32 = 9999;

/// A validated UTC timestamp with second precision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp {
    unix_seconds: i64,
}

impl Timestamp {
    /// Parses exactly `YYYY-MM-DDTHH:MM:SSZ`.
    ///
    /// Offsets, fractional seconds, leap seconds, and non-UTC forms are
    /// rejected rather than normalized.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidTimestamp`] for any malformed or
    /// out-of-range value.
    pub fn parse(value: &str) -> Result<Self, UpdaterError> {
        let invalid = || UpdaterError::InvalidTimestamp {
            value: UpdaterError::bounded_detail(value),
        };

        if value.len() != 20 {
            return Err(invalid());
        }
        let bytes = value.as_bytes();
        if bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes[10] != b'T'
            || bytes[13] != b':'
            || bytes[16] != b':'
            || bytes[19] != b'Z'
        {
            return Err(invalid());
        }

        let year = parse_digits(bytes, 0, 4).ok_or_else(invalid)?;
        let month = parse_digits(bytes, 5, 2).ok_or_else(invalid)?;
        let day = parse_digits(bytes, 8, 2).ok_or_else(invalid)?;
        let hour = parse_digits(bytes, 11, 2).ok_or_else(invalid)?;
        let minute = parse_digits(bytes, 14, 2).ok_or_else(invalid)?;
        let second = parse_digits(bytes, 17, 2).ok_or_else(invalid)?;

        let year = i32::try_from(year).map_err(|_| invalid())?;
        if !(MIN_YEAR..=MAX_YEAR).contains(&year)
            || !(1..=12).contains(&month)
            || day == 0
            || day > days_in_month(year, month)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return Err(invalid());
        }

        let days = days_from_civil(year, month, day);
        let seconds = days * SECONDS_PER_DAY
            + i64::from(hour) * 3_600
            + i64::from(minute) * 60
            + i64::from(second);
        Ok(Self {
            unix_seconds: seconds,
        })
    }

    /// Creates a timestamp from Unix seconds.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::InvalidTimestamp`] when the value falls outside
    /// years 1970 through 9999.
    pub fn from_unix_seconds(seconds: i64) -> Result<Self, UpdaterError> {
        let days = seconds.div_euclid(SECONDS_PER_DAY);
        let (year, _, _) = civil_from_days(days);
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(UpdaterError::InvalidTimestamp {
                value: format!("unix seconds {seconds} is outside the supported range"),
            });
        }
        Ok(Self {
            unix_seconds: seconds,
        })
    }

    /// Returns the current UTC time.
    ///
    /// # Errors
    ///
    /// Returns [`UpdaterError::Clock`] when the system clock is before the
    /// Unix epoch or outside the supported year range.
    pub fn now_utc() -> Result<Self, UpdaterError> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| UpdaterError::Clock {
                detail: UpdaterError::bounded_detail(error.to_string()),
            })?;
        let seconds = i64::try_from(duration.as_secs()).map_err(|_| UpdaterError::Clock {
            detail: "system clock is outside the supported range".to_owned(),
        })?;
        Self::from_unix_seconds(seconds).map_err(|_| UpdaterError::Clock {
            detail: "system clock is outside the supported range".to_owned(),
        })
    }

    /// Returns Unix seconds.
    #[must_use]
    pub const fn unix_seconds(self) -> i64 {
        self.unix_seconds
    }

    /// Adds seconds, returning `None` when the result is outside the supported
    /// range.
    #[must_use]
    pub fn checked_add_seconds(self, seconds: i64) -> Option<Self> {
        let value = self.unix_seconds.checked_add(seconds)?;
        Self::from_unix_seconds(value).ok()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days = self.unix_seconds.div_euclid(SECONDS_PER_DAY);
        let seconds_of_day = self.unix_seconds.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        let hour = seconds_of_day / 3_600;
        let minute = (seconds_of_day % 3_600) / 60;
        let second = seconds_of_day % 60;
        write!(
            formatter,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
        )
    }
}

fn parse_digits(bytes: &[u8], start: usize, length: usize) -> Option<u32> {
    let mut value = 0u32;
    for byte in bytes.get(start..start + length)? {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value * 10 + u32::from(byte - b'0');
    }
    Some(value)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Days since 1970-01-01, from Howard Hinnant's civil calendar algorithm.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = i64::from((month + 9) % 12);
    let day_of_year = (153 * month_prime + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Civil date from days since 1970-01-01, from Howard Hinnant's algorithm.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (
        i32::try_from(year).unwrap_or(0),
        u32::try_from(month).unwrap_or(0),
        u32::try_from(day).unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_epoch() {
        let timestamp = Timestamp::parse("1970-01-01T00:00:00Z").unwrap();
        assert_eq!(timestamp.unix_seconds(), 0);
        assert_eq!(timestamp.to_string(), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn round_trips_known_values() {
        for value in [
            "1970-01-01T00:00:00Z",
            "2000-02-29T23:59:59Z",
            "2024-02-29T12:34:56Z",
            "2026-11-07T00:00:00Z",
            "9999-12-31T23:59:59Z",
        ] {
            let timestamp = Timestamp::parse(value).unwrap();
            assert_eq!(timestamp.to_string(), value);
            assert_eq!(
                Timestamp::from_unix_seconds(timestamp.unix_seconds())
                    .unwrap()
                    .to_string(),
                value
            );
        }
    }

    #[test]
    fn rejects_noncanonical_forms() {
        for value in [
            "2026-11-07T00:00:00+00:00",
            "2026-11-07T00:00:00.000Z",
            "2026-11-07T00:00:60Z",
            "2026-11-07 00:00:00Z",
            "2026-02-30T00:00:00Z",
            "2023-02-29T00:00:00Z",
            "1969-12-31T23:59:59Z",
            "10000-01-01T00:00:00Z",
            "",
            "not a timestamp",
        ] {
            assert!(
                Timestamp::parse(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn accepts_leap_day() {
        assert!(Timestamp::parse("2024-02-29T00:00:00Z").is_ok());
        assert!(Timestamp::parse("2023-02-29T00:00:00Z").is_err());
    }

    #[test]
    fn orders_timestamps() {
        let earlier = Timestamp::parse("2026-01-01T00:00:00Z").unwrap();
        let later = Timestamp::parse("2026-01-02T00:00:00Z").unwrap();
        assert!(earlier < later);
        assert_eq!(earlier.checked_add_seconds(86_400), Some(later));
    }

    #[test]
    fn rejects_out_of_range_unix_seconds() {
        assert!(Timestamp::from_unix_seconds(-1).is_err());
        assert!(Timestamp::from_unix_seconds(i64::MAX).is_err());
    }
}
