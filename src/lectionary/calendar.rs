//! Date math shared by RCL and the Catholic lectionary — both follow the
//! same Western liturgical calendar (Easter, Advent, Ash Wednesday, Trinity,
//! Christ the King all land on the same dates in both traditions; only the
//! reading content and Ordinary Time week-numbering scheme differ). The
//! Narrative Lectionary's calendar is structurally different and does not
//! use this module — see `narrative.rs`.

use chrono::{Datelike, NaiveDate};

/// Anonymous Gregorian algorithm.
pub fn easter(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .expect("Anonymous Gregorian algorithm always yields a valid date")
}

/// First Sunday of Advent for a given civil year.
pub fn advent_sunday(year: i32) -> NaiveDate {
    let xmas = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
    let days_since_sunday = (python_weekday(xmas) + 1) % 7;
    let last_sunday_before_xmas = xmas - chrono::Duration::days(days_since_sunday);
    last_sunday_before_xmas - chrono::Duration::weeks(3)
}

/// Return 'A', 'B', or 'C' for the lectionary year containing date `d`.
/// Shared by RCL and the Catholic lectionary — both use the same 3-year
/// cycle keyed to the same Advent Sunday.
pub fn lectionary_year(d: NaiveDate) -> &'static str {
    let adv = advent_sunday(d.year());
    let base = if d >= adv { d.year() + 1 } else { d.year() };
    const YEARS: [&str; 3] = ["A", "B", "C"];
    YEARS[(base - 2023).rem_euclid(3) as usize]
}

/// Python's `date.weekday()`: Monday = 0 .. Sunday = 6. Chrono's
/// `num_days_from_monday()` agrees, so this is just a naming/type bridge to
/// keep the calendar math a direct line-by-line mirror of Rubric's source.
pub fn python_weekday(d: NaiveDate) -> i64 {
    d.weekday().num_days_from_monday() as i64
}

pub fn is_sunday(d: NaiveDate) -> bool {
    (python_weekday(d) + 1) % 7 == 0
}

/// Return the Sunday nearest to `d` (or `d` itself if Sunday).
pub fn nearest_sunday(d: NaiveDate) -> NaiveDate {
    let wd = (python_weekday(d) + 1) % 7; // 0 = Sunday
    if wd <= 3 {
        d - chrono::Duration::days(wd)
    } else {
        d + chrono::Duration::days(7 - wd)
    }
}

pub fn sunday_on_or_before(d: NaiveDate) -> NaiveDate {
    let wd = (python_weekday(d) + 1) % 7;
    d - chrono::Duration::days(wd)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn easter_2024() {
        assert_eq!(easter(2024), d(2024, 3, 31));
    }

    #[test]
    fn advent_2024() {
        assert_eq!(advent_sunday(2024), d(2024, 12, 1));
    }

    #[test]
    fn advent_is_sunday() {
        for year in 2020..2030 {
            let day = advent_sunday(year);
            assert_eq!(day.weekday(), chrono::Weekday::Sun);
        }
    }

    #[test]
    fn three_year_cycle() {
        let years: std::collections::HashSet<_> =
            (0..6).map(|i| lectionary_year(d(2020 + i, 6, 1))).collect();
        assert_eq!(years.len(), 3);
    }
}
