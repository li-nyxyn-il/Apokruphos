//! The calendar: eight months and five weekdays.
//!
//! ASSUMPTION: the notes give the month and weekday names but not the length
//! of a month. Forty-five days (nine five-day weeks) gives a 360-day year.
//! Change `DAYS_PER_MONTH` when the real figure is decided.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const MONTHS: [&str; 8] = [
    "Jhuze", "Zeiphe", "Pyndas", "Hevellu", "Aetyr", "Swon", "Measli", "Imaehir",
];

pub const WEEKDAYS: [&str; 5] = ["Arden", "Bhurden", "Fiurden", "Dereden", "Seldyn"];

pub const DAYS_PER_MONTH: u32 = 50;
pub const DAYS_PER_YEAR: u32 = DAYS_PER_MONTH * MONTHS.len() as u32;

/// A day in the zdy reckoning.
///
/// The field order matters: deriving `Ord` compares `year` first and then
/// `day_of_year`, so dates sort chronologically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Date {
    pub year: u32,
    /// Counted from 0, so the first day of the year is 0.
    pub day_of_year: u32,
}

impl Date {
    /// Builds a date from a year, a month (1 to 8) and a day of the month
    /// (1 to `DAYS_PER_MONTH`). Returns `None` if either is out of range.
    pub fn new(year: u32, month: u32, day: u32) -> Option<Date> {
        let month_ok = (1..=MONTHS.len() as u32).contains(&month);
        let day_ok = (1..=DAYS_PER_MONTH).contains(&day);
        if month_ok && day_ok {
            Some(Date {
                year,
                day_of_year: (month - 1) * DAYS_PER_MONTH + (day - 1),
            })
        } else {
            None
        }
    }

    pub fn month_name(self) -> &'static str {
        MONTHS[(self.day_of_year / DAYS_PER_MONTH) as usize]
    }

    /// The day of the month, counted from 1.
    pub fn day_of_month(self) -> u32 {
        self.day_of_year % DAYS_PER_MONTH + 1
    }

    pub fn weekday_name(self) -> &'static str {
        WEEKDAYS[(self.day_of_year % WEEKDAYS.len() as u32) as usize]
    }

    pub fn next_day(self) -> Date {
        self.plus_days(1)
    }

    pub fn plus_days(self, days: u32) -> Date {
        let total = self.day_of_year + days;
        Date {
            year: self.year + total / DAYS_PER_YEAR,
            day_of_year: total % DAYS_PER_YEAR,
        }
    }

    /// Whole days from `earlier` to `self`. Zero if `earlier` is not earlier.
    pub fn days_since(self, earlier: Date) -> u32 {
        let a = self.year * DAYS_PER_YEAR + self.day_of_year;
        let b = earlier.year * DAYS_PER_YEAR + earlier.day_of_year;
        a.saturating_sub(b)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {} {} {} zdy",
            self.weekday_name(),
            self.day_of_month(),
            self.month_name(),
            self.year
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_day_of_the_year() {
        let d = Date::new(192, 1, 1).unwrap();
        assert_eq!(d.to_string(), "Arden, 1 Jhuze 192 zdy");
    }

    #[test]
    fn the_year_rolls_over() {
        let last = Date::new(192, 8, DAYS_PER_MONTH).unwrap();
        assert_eq!(last.month_name(), "Imaehir");
        assert_eq!(last.next_day(), Date::new(193, 1, 1).unwrap());
    }

    #[test]
    fn out_of_range_dates_are_refused() {
        assert!(Date::new(192, 0, 1).is_none());
        assert!(Date::new(192, 9, 1).is_none());
        assert!(Date::new(192, 1, DAYS_PER_MONTH + 1).is_none());
    }
}
