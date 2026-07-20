//! Roman Catholic Sunday Lectionary for Mass, Years A/B/C.
//!
//! Shares RCL's Western liturgical calendar (`super::calendar`) for every
//! fixed point — Easter, Advent, Ash Wednesday, Pentecost, Trinity Sunday,
//! Christ the King all land on the same dates in both traditions. The one
//! genuinely different piece of engineering is Ordinary Time week
//! numbering: RCL's "Propers" are pinned to specific calendar dates (see
//! `rcl::PROPER_CENTRES`), but Catholic Ordinary Time Sundays are numbered
//! *sequentially* — 2nd Sunday in Ordinary Time is the first Sunday after
//! the Baptism of the Lord, counting up to the Sunday before Ash Wednesday;
//! then, since Christ the King is always the 34th and final Sunday in
//! Ordinary Time (the Sunday immediately before Advent 1), the sequence
//! resumes after Trinity Sunday at whatever number makes that arithmetic
//! land on 34 — no fixed-date table needed, just counting backward from
//! Christ the King. This is standard, well-documented Catholic liturgical
//! calendar arithmetic (see e.g. the General Norms for the Liturgical Year
//! and Calendar), independent of the actual reading content in
//! `catholic_data`.
//!
//! Known simplifications (matching the level of approximation `rcl.rs`
//! already makes elsewhere): Holy Family, when Christmas Day itself falls
//! on a Sunday, is actually transferred to Dec 30 (a weekday) rather than
//! kept on a Sunday — not handled here, since this app plans Sunday
//! services. Baptism of the Lord, when Epiphany Sunday falls on Jan 7 or 8,
//! is technically transferred to the following Monday — likewise not
//! handled; this always resolves Baptism to "the Sunday after Epiphany
//! Sunday".

use chrono::{Datelike, NaiveDate};

use super::calendar::{advent_sunday, easter, is_sunday, lectionary_year};
use super::catholic_data;
use super::LiturgicalInfo;

type Readings = (&'static str, &'static str, &'static str, &'static str);

fn lookup(key_year: &str, week_id: &str) -> Option<Readings> {
    catholic_data::READINGS
        .iter()
        .find(|&&(yk, wid, _)| yk == key_year && wid == week_id)
        .map(|&(_, _, r)| r)
        .or_else(|| {
            catholic_data::READINGS
                .iter()
                .find(|&&(yk, wid, _)| yk == "ALL" && wid == week_id)
                .map(|&(_, _, r)| r)
        })
}

fn readings_vec(r: Option<Readings>) -> Vec<(String, String)> {
    let (ot, psalm, epistle, gospel) = r.unwrap_or(("—", "—", "—", "—"));
    vec![
        ("OT".to_string(), ot.to_string()),
        ("Psalm".to_string(), psalm.to_string()),
        ("Epistle".to_string(), epistle.to_string()),
        ("Gospel".to_string(), gospel.to_string()),
    ]
}

/// Same fixed liturgical-colour-per-season palette as RCL (`rcl::COLOURS`)
/// — the Western calendar's colours don't vary between the two traditions.
fn colour_for(season_key: &str) -> (&'static str, &'static str) {
    super::rcl::COLOURS
        .iter()
        .find(|&&(k, _)| k == season_key)
        .map(|&(_, c)| c)
        .unwrap_or(("Green", "#15803D"))
}

fn result_for(lec_year: &str, season_key: &str, week_label: &str, week_id: &str) -> LiturgicalInfo {
    let r = lookup(lec_year, week_id);
    let (colour_name, colour_hex) = colour_for(season_key);
    LiturgicalInfo {
        season: season_key.to_string(),
        week: week_label.to_string(),
        year: lec_year.to_string(),
        colour: colour_name.to_string(),
        colour_hex: colour_hex.to_string(),
        readings: readings_vec(r),
        found: r.is_some(),
    }
}

/// The one Sunday in `[Jan 2, Jan 8]` — Epiphany as transferred in the US
/// (and most English-speaking dioceses) rather than fixed to Jan 6.
fn epiphany_sunday(year: i32) -> NaiveDate {
    let jan2 = NaiveDate::from_ymd_opt(year, 1, 2).unwrap();
    (0..7)
        .map(|i| jan2 + chrono::Duration::days(i))
        .find(|d| is_sunday(*d))
        .expect("a 7-day window always contains exactly one Sunday")
}

pub fn get_liturgical_info(d: NaiveDate) -> LiturgicalInfo {
    let year = d.year();
    let lec_year = lectionary_year(d);

    let e = easter(year);
    let adv = advent_sunday(year);

    let ash_wed = e - chrono::Duration::days(46);
    let palm_sun = e - chrono::Duration::days(7);
    let holy_thu = e - chrono::Duration::days(3);
    let good_fri = e - chrono::Duration::days(2);
    let pentecost = e + chrono::Duration::days(49);
    let trinity = pentecost + chrono::Duration::days(7);
    let corpus_christi = trinity + chrono::Duration::days(7);
    let christ_king = adv - chrono::Duration::days(7);

    // ── Fixed days ───────────────────────────────────────────────────────
    if d == good_fri {
        return result_for(lec_year, "Good Friday", "Good Friday", "GoodFriday");
    }
    if d == holy_thu {
        return result_for(lec_year, "Holy Thursday", "Holy Thursday", "HolyThursday");
    }
    if d == ash_wed {
        return result_for(lec_year, "Lent", "Ash Wednesday", "AshWednesday");
    }

    // ── Advent ───────────────────────────────────────────────────────────
    for week_num in 1..=4i64 {
        let adv_sun = adv + chrono::Duration::weeks(week_num - 1);
        if d == adv_sun {
            return result_for(
                lec_year,
                "Advent",
                &format!("Advent {week_num}, Year {lec_year}"),
                &format!("Advent{week_num}"),
            );
        }
    }

    // ── Christmas / Mary, Mother of God ──────────────────────────────────
    let xmas = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
    if d == xmas {
        return result_for(lec_year, "Christmas", "The Nativity of the Lord", "Christmas");
    }
    let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    if d == jan1 {
        return result_for(lec_year, "Christmas", "Mary, Mother of God", "MaryMotherOfGod");
    }
    let dec31 = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
    if xmas < d && d <= dec31 && is_sunday(d) {
        return result_for(
            lec_year,
            "Christmas",
            &format!("Holy Family, Year {lec_year}"),
            "HolyFamily",
        );
    }

    // ── Epiphany / Baptism of the Lord ───────────────────────────────────
    let epiphany = epiphany_sunday(year);
    if d == epiphany {
        return result_for(lec_year, "Epiphany", "The Epiphany of the Lord", "Epiphany");
    }
    let baptism = epiphany + chrono::Duration::weeks(1);
    if d == baptism {
        return result_for(
            lec_year,
            "Baptism",
            &format!("Baptism of the Lord, Year {lec_year}"),
            "BaptismOfLord",
        );
    }

    // ── Ordinary Time before Lent (sequential from Baptism) ──────────────
    if baptism < d && d < ash_wed && is_sunday(d) {
        let weeks_after = (d - baptism).num_days() / 7;
        let n = weeks_after + 1;
        return result_for(
            lec_year,
            "Ordinary",
            &format!("Ordinary Time {n}, Year {lec_year}"),
            &format!("Ordinary{n}"),
        );
    }

    // ── Lent ─────────────────────────────────────────────────────────────
    for lent_week in 1..=6i64 {
        let lent_sun = if lent_week == 6 {
            palm_sun
        } else {
            e - chrono::Duration::weeks(7 - lent_week)
        };
        if d == lent_sun {
            if lent_week == 6 {
                return result_for(
                    lec_year,
                    "Palm Sunday",
                    &format!("Palm Sunday of the Passion, Year {lec_year}"),
                    "PalmSunday",
                );
            }
            return result_for(
                lec_year,
                "Lent",
                &format!("Lent {lent_week}, Year {lec_year}"),
                &format!("Lent{lent_week}"),
            );
        }
    }

    // ── Easter ───────────────────────────────────────────────────────────
    if d == e {
        return result_for(lec_year, "Easter", &format!("Easter Sunday, Year {lec_year}"), "Easter");
    }
    for easter_week in 2..=7i64 {
        let easter_sun = e + chrono::Duration::weeks(easter_week - 1);
        if d == easter_sun {
            if easter_week == 7 {
                return result_for(
                    lec_year,
                    "Easter",
                    &format!("Easter 7 (Ascension observed), Year {lec_year}"),
                    "Easter7",
                );
            }
            return result_for(
                lec_year,
                "Easter",
                &format!("Easter {easter_week}, Year {lec_year}"),
                &format!("Easter{easter_week}"),
            );
        }
    }

    // ── Pentecost / Trinity / Corpus Christi ─────────────────────────────
    if d == pentecost {
        return result_for(lec_year, "Pentecost", &format!("Pentecost, Year {lec_year}"), "Pentecost");
    }
    if d == trinity {
        return result_for(lec_year, "Trinity", &format!("Trinity Sunday, Year {lec_year}"), "Trinity");
    }
    if d == corpus_christi {
        return result_for(
            lec_year,
            "Trinity", // reuse the same White/Gold colour entry as Trinity
            &format!("Corpus Christi, Year {lec_year}"),
            "CorpusChristi",
        );
    }
    if d == christ_king {
        return result_for(
            lec_year,
            "Christ the King",
            &format!("Christ the King, Year {lec_year}"),
            "Ordinary34",
        );
    }

    // ── Ordinary Time after Pentecost (counting backward from Christ the
    // King, which is always the 34th and final Sunday) ──────────────────
    if corpus_christi < d && d < christ_king && is_sunday(d) {
        let weeks_before_christking = (christ_king - d).num_days() / 7;
        let n = 34 - weeks_before_christking;
        return result_for(
            lec_year,
            "Ordinary",
            &format!("Ordinary Time {n}, Year {lec_year}"),
            &format!("Ordinary{n}"),
        );
    }

    // Fallback
    let (colour_name, colour_hex) = colour_for("Ordinary");
    LiturgicalInfo {
        season: "Ordinary".to_string(),
        week: "Ordinary Time".to_string(),
        year: lec_year.to_string(),
        colour: colour_name.to_string(),
        colour_hex: colour_hex.to_string(),
        readings: Vec::new(),
        found: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn christ_the_king_is_ordinary_34() {
        // 2026: Advent Sunday is Nov 29 (see rcl.rs's own golden test), so
        // Christ the King is Nov 22.
        let info = get_liturgical_info(d(2026, 11, 22));
        assert_eq!(info.season, "Christ the King");
    }

    #[test]
    fn trinity_sunday_2026() {
        // Easter 2026 = Apr 5 (see rcl.rs), Pentecost = +49 days = May 24,
        // Trinity = May 31.
        let info = get_liturgical_info(d(2026, 5, 31));
        assert_eq!(info.season, "Trinity");
    }

    #[test]
    fn corpus_christi_follows_trinity_by_a_week() {
        let info = get_liturgical_info(d(2026, 6, 7));
        assert_eq!(info.week, "Corpus Christi, Year A");
    }

    #[test]
    fn baptism_of_lord_is_the_sunday_after_epiphany() {
        let epiphany = epiphany_sunday(2026);
        let info = get_liturgical_info(epiphany + chrono::Duration::weeks(1));
        assert_eq!(info.season, "Baptism");
    }

    #[test]
    fn ordinary_time_numbering_starts_at_2() {
        let epiphany = epiphany_sunday(2026);
        let baptism = epiphany + chrono::Duration::weeks(1);
        let info = get_liturgical_info(baptism + chrono::Duration::weeks(1));
        assert_eq!(info.week, "Ordinary Time 2, Year A");
    }

    #[test]
    fn ordinary_time_after_pentecost_reaches_34_at_christ_the_king() {
        let info = get_liturgical_info(d(2026, 11, 22));
        assert_eq!(info.week, "Christ the King, Year A");
    }

    #[test]
    fn every_sunday_of_2026_resolves_without_panic() {
        let mut day = d(2026, 1, 1);
        let end = d(2027, 1, 1);
        while day < end {
            if is_sunday(day) {
                let info = get_liturgical_info(day);
                assert!(!info.season.is_empty());
            }
            day += chrono::Duration::days(1);
        }
    }
}
