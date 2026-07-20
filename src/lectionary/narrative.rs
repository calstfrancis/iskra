//! Narrative Lectionary (Luther Seminary / Working Preacher, 4-year cycle).
//! See `narrative_data` for the full sourcing notes and the real structure
//! this engine is built against — it is NOT a variant of RCL/Catholic's
//! calendar, so it only reuses `super::calendar` for the stretch (Advent
//! through Pentecost) where NL's season labels line up with the same fixed
//! Western liturgical dates RCL/Catholic use. The Fall stretch (Sunday
//! after Labor Day through the Sunday before Advent 1) has no calendar
//! anchor of its own beyond Labor Day, so it's indexed sequentially instead.
//!
//! The 4-year rotation is anchored to the specific program years
//! `narrative_data` was fetched for (2023→Year 2, 2024→Year 3, 2025→Year 4,
//! 2026→Year 1), repeating every 4 years. Summer (Pentecost through the
//! Sunday before the next Labor Day) falls outside the cycle entirely and
//! only has data for the specific summer fetched (`SUMMER_2026`) — other
//! years' summers resolve to "not found" rather than guessing.

use chrono::{Datelike, NaiveDate};

use super::calendar::{advent_sunday, easter, is_sunday, nearest_sunday};
use super::narrative_data::{SummerWeek, Week, SUMMER_2026, YEAR1_MATTHEW, YEAR2_MARK, YEAR3_LUKE, YEAR4_JOHN};
use super::LiturgicalInfo;

const ORDINALS: &[&str] =
    &["First", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh", "Eighth", "Ninth"];

fn ordinal(n: i64) -> &'static str {
    ORDINALS.get((n - 1) as usize).copied().unwrap_or("Nth")
}

/// First Monday of September (US Labor Day) — NL's Fall anchor.
fn labor_day(year: i32) -> NaiveDate {
    let sep1 = NaiveDate::from_ymd_opt(year, 9, 1).unwrap();
    let offset = (7 - sep1.weekday().num_days_from_monday() as i64) % 7;
    sep1 + chrono::Duration::days(offset)
}

/// The NL "program year" a date belongs to: the cycle runs from the Sunday
/// after Labor Day through the following Pentecost, so a January date
/// belongs to the program year that started the PREVIOUS September.
fn program_year(d: NaiveDate) -> i32 {
    if d >= labor_day(d.year()) {
        d.year()
    } else {
        d.year() - 1
    }
}

fn weeks_for_program_year(program_year: i32) -> (&'static str, &'static [Week]) {
    match (program_year - 2023).rem_euclid(4) {
        0 => ("2 (Mark)", YEAR2_MARK),
        1 => ("3 (Luke)", YEAR3_LUKE),
        2 => ("4 (John)", YEAR4_JOHN),
        _ => ("1 (Matthew)", YEAR1_MATTHEW),
    }
}

/// Index of the first entry whose season is part of Advent — i.e. the
/// length of the Fall (Labor Day–anchored, sequential-Sunday) segment.
fn fall_len(weeks: &[Week]) -> usize {
    weeks.iter().position(|w| w.season.contains("Sunday of Advent")).unwrap_or(weeks.len())
}

fn find_exact<'a>(weeks: &'a [Week], season: &str) -> Option<&'a Week> {
    weeks.iter().find(|w| w.season == season)
}

fn find_contains<'a>(weeks: &'a [Week], needle: &str) -> Option<&'a Week> {
    weeks.iter().find(|w| w.season.contains(needle))
}

fn readings_vec(week: &Week) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, p) in week.primary.iter().enumerate() {
        let label = if week.primary.len() > 1 {
            format!("Reading {}", i + 1)
        } else {
            "Reading".to_string()
        };
        out.push((label, p.to_string()));
    }
    if let Some(alt) = week.alt_primary {
        out.push(("Alternate".to_string(), alt.to_string()));
    }
    if let Some(acc) = week.accompanying {
        out.push(("Accompanying".to_string(), acc.to_string()));
    }
    out
}

/// Reuses RCL's fixed liturgical-colour-per-season palette, mapped from
/// NL's season-label text rather than a shared season key.
fn colour_for(season: &str) -> (&'static str, &'static str) {
    let key = if season.contains("Advent") {
        "Advent"
    } else if season.contains("Christmas") {
        "Christmas"
    } else if season.contains("Epiphany") || season.contains("Baptism") {
        "Epiphany"
    } else if season.contains("Transfiguration") {
        "Transfiguration"
    } else if season.contains("Lent") || season.contains("Ash Wednesday") {
        "Lent"
    } else if season.contains("Palm") {
        "Palm Sunday"
    } else if season.contains("Maundy") {
        "Holy Thursday"
    } else if season.contains("Good Friday") {
        "Good Friday"
    } else if season.contains("Easter") {
        "Easter"
    } else if season.contains("Pentecost") {
        "Pentecost"
    } else if season.contains("Christ the King") || season.contains("Reign of Christ") {
        "Christ the King"
    } else {
        "Ordinary"
    };
    super::rcl::COLOURS
        .iter()
        .find(|&&(k, _)| k == key)
        .map(|&(_, c)| c)
        .unwrap_or(("Green", "#15803D"))
}

fn result_for(year_label: &str, week: &Week) -> LiturgicalInfo {
    let (colour_name, colour_hex) = colour_for(week.season);
    LiturgicalInfo {
        season: week.season.to_string(),
        week: format!("{} · Year {}", week.title, year_label),
        year: year_label.to_string(),
        colour: colour_name.to_string(),
        colour_hex: colour_hex.to_string(),
        readings: readings_vec(week),
        found: true,
    }
}

fn not_found() -> LiturgicalInfo {
    LiturgicalInfo {
        season: "Narrative".to_string(),
        week: "No Narrative Lectionary reading for this date".to_string(),
        year: String::new(),
        colour: "Green".to_string(),
        colour_hex: "#15803D".to_string(),
        readings: Vec::new(),
        found: false,
    }
}

pub fn get_liturgical_info(d: NaiveDate) -> LiturgicalInfo {
    let py = program_year(d);
    let ld_sunday = labor_day(py); // Labor Day itself is always a Monday.
    let (year_label, weeks) = weeks_for_program_year(py);
    let fall_len = fall_len(weeks);

    // ── Fall: sequential Sundays from Labor Day, no calendar anchor of its
    // own beyond that (see module doc) ──────────────────────────────────
    if d >= ld_sunday && is_sunday(d) {
        let idx = ((d - ld_sunday).num_days() / 7) as usize;
        if idx < fall_len {
            return result_for(year_label, &weeks[idx]);
        }
    }

    // ── Advent through Pentecost: same fixed Western liturgical dates as
    // RCL/Catholic, matched to this program year's entries by season label
    // (which may legitimately be absent some years — see module doc on
    // gaps — in which case this falls through to `not_found`) ───────────
    let year = d.year();
    let e = easter(year);
    let adv = advent_sunday(year);

    for week_num in 1..=4i64 {
        if d == adv + chrono::Duration::weeks(week_num - 1) {
            if let Some(week) = find_exact(weeks, &format!("{} Sunday of Advent", ordinal(week_num))) {
                return result_for(year_label, week);
            }
        }
    }

    let xmas = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
    if d == xmas - chrono::Duration::days(1) {
        if let Some(week) = find_exact(weeks, "Christmas Eve") {
            return result_for(year_label, week);
        }
    }
    if d == xmas {
        if let Some(week) = find_exact(weeks, "Christmas Day") {
            return result_for(year_label, week);
        }
    }
    if xmas < d && d <= NaiveDate::from_ymd_opt(year, 12, 31).unwrap() && is_sunday(d) {
        if let Some(week) = find_exact(weeks, "First Sunday of Christmas") {
            return result_for(year_label, week);
        }
    }
    let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    let jan5 = NaiveDate::from_ymd_opt(year, 1, 5).unwrap();
    if jan1 <= d && d <= jan5 && is_sunday(d) {
        if let Some(week) = find_exact(weeks, "Second Sunday of Christmas") {
            return result_for(year_label, week);
        }
    }

    let epiphany = NaiveDate::from_ymd_opt(year, 1, 6).unwrap();
    let baptism = nearest_sunday(epiphany);
    if d == baptism {
        if let Some(week) = find_exact(weeks, "Baptism of Our Lord") {
            return result_for(year_label, week);
        }
    }
    let ash_wed = e - chrono::Duration::days(46);
    if baptism < d && d < ash_wed && is_sunday(d) {
        let weeks_after = (d - baptism).num_days() / 7;
        let n = weeks_after + 1;
        if let Some(week) = find_exact(weeks, &format!("{} Sunday after Epiphany", ordinal(n))) {
            return result_for(year_label, week);
        }
    }

    let transfig = super::calendar::sunday_on_or_before(ash_wed - chrono::Duration::days(1));
    if d == transfig {
        if let Some(week) = find_exact(weeks, "Transfiguration") {
            return result_for(year_label, week);
        }
    }
    if d == ash_wed {
        if let Some(week) = find_exact(weeks, "Ash Wednesday") {
            return result_for(year_label, week);
        }
    }

    let palm_sun = e - chrono::Duration::days(7);
    for lent_week in 1..=5i64 {
        if d == e - chrono::Duration::weeks(7 - lent_week) {
            if let Some(week) = find_exact(weeks, &format!("{} Sunday in Lent", ordinal(lent_week))) {
                return result_for(year_label, week);
            }
        }
    }
    if d == palm_sun {
        if let Some(week) = find_contains(weeks, "Palm") {
            return result_for(year_label, week);
        }
    }
    if d == e - chrono::Duration::days(3) {
        if let Some(week) = find_exact(weeks, "Maundy Thursday") {
            return result_for(year_label, week);
        }
    }
    if d == e - chrono::Duration::days(2) {
        if let Some(week) = find_exact(weeks, "Good Friday") {
            return result_for(year_label, week);
        }
    }
    if d == e {
        if let Some(week) = find_exact(weeks, "Easter") {
            return result_for(year_label, week);
        }
    }
    for easter_week in 2..=7i64 {
        if d == e + chrono::Duration::weeks(easter_week - 1) {
            if let Some(week) = find_exact(weeks, &format!("{} Sunday of Easter", ordinal(easter_week))) {
                return result_for(year_label, week);
            }
        }
    }
    let pentecost = e + chrono::Duration::days(49);
    if d == pentecost {
        if let Some(week) = find_exact(weeks, "Pentecost") {
            return result_for(year_label, week);
        }
    }

    // ── Summer: outside the 4-year cycle entirely — only the specific
    // summer that was actually fetched has data ──────────────────────────
    if let Some(week) = summer_week_for(d) {
        let (colour_name, colour_hex) = colour_for("Ordinary");
        return LiturgicalInfo {
            season: week.series.to_string(),
            week: week.title.to_string(),
            year: String::new(),
            colour: colour_name.to_string(),
            colour_hex: colour_hex.to_string(),
            readings: vec![
                ("Primary".to_string(), week.primary.to_string()),
                ("Accompanying".to_string(), week.accompanying.to_string()),
            ],
            found: true,
        };
    }

    not_found()
}

fn summer_week_for(d: NaiveDate) -> Option<&'static SummerWeek> {
    SUMMER_2026.iter().find(|w| parse_us_date(w.date) == Some(d))
}

fn parse_us_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<i32> = s.split('/').filter_map(|p| p.parse().ok()).collect();
    match parts.as_slice() {
        [m, day, y] => NaiveDate::from_ymd_opt(*y, *m as u32, *day as u32),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn labor_day_2026_is_first_monday_of_september() {
        let ld = labor_day(2026);
        assert_eq!(ld.weekday(), chrono::Weekday::Mon);
        assert_eq!(ld.month(), 9);
        assert!(ld.day() <= 7);
    }

    #[test]
    fn program_year_before_labor_day_is_previous_year() {
        assert_eq!(program_year(d(2027, 3, 1)), 2026);
    }

    #[test]
    fn program_year_after_labor_day_is_same_year() {
        assert_eq!(program_year(d(2026, 9, 20)), 2026);
    }

    #[test]
    fn year1_matthew_maps_to_2026_program_year() {
        let (label, weeks) = weeks_for_program_year(2026);
        assert_eq!(label, "1 (Matthew)");
        assert!(std::ptr::eq(weeks, YEAR1_MATTHEW));
    }

    #[test]
    fn fall_sunday_after_labor_day_2026_resolves() {
        // Labor Day 2026 = Sept 7 (Monday); first Fall Sunday = Sept 13,
        // matching YEAR1_MATTHEW's NL101 source_date "September 13".
        let info = get_liturgical_info(d(2026, 9, 13));
        assert!(info.found);
        assert_eq!(info.season, "Sixteenth Sunday after Pentecost");
    }

    #[test]
    fn christmas_eve_2026_resolves() {
        let info = get_liturgical_info(d(2026, 12, 24));
        assert!(info.found);
        assert_eq!(info.season, "Christmas Eve");
    }

    #[test]
    fn easter_sunday_resolves_to_exact_season_not_a_later_sunday() {
        // 2027: Easter = March 28 (matches YEAR1_MATTHEW's NL139 source_date).
        let info = get_liturgical_info(d(2027, 3, 28));
        assert!(info.found);
        assert_eq!(info.season, "Easter");
    }

    #[test]
    fn pentecost_2027_resolves() {
        let info = get_liturgical_info(d(2027, 5, 16));
        assert!(info.found);
        assert_eq!(info.season, "Pentecost");
    }

    #[test]
    fn summer_2026_week_resolves() {
        let info = get_liturgical_info(d(2026, 5, 31));
        assert!(info.found);
        assert_eq!(info.week, "Loss and Loyalty");
    }

    #[test]
    fn every_sunday_of_2026_resolves_without_panic() {
        let mut day = d(2026, 1, 1);
        let end = d(2027, 1, 1);
        while day < end {
            let _ = get_liturgical_info(day);
            day += chrono::Duration::days(1);
        }
    }
}
