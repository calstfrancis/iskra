//! Roman Catholic Sunday Lectionary for Mass, Years A/B/C.
//!
//! Source(s):
//!   - Felix Just, S.J., "Roman Catholic Lectionary for Mass" reference
//!     tables (1998/2002 USA edition), catholic-resources.org/Lectionary/,
//!     specifically the Advent, Christmas, Lent, Easter, Ordinary Time
//!     (Cycle A/B/C), and Solemnities (Trinity/Corpus Christi) pages.
//!   - Cross-checked against USCCB's own daily-readings pages
//!     (bible.usccb.org/bible/readings/*.cfm), which is the primary US
//!     authority — used directly to verify Ash Wednesday and spot-check
//!     several entries above.
//!   Fetched 2026-07-19.
//!
//! Citation style matches `rcl.rs`: en dash "–" for verse ranges, standard
//! abbreviations (Isa, Ps, Rom, Matt, 1 Cor, Jas, etc.). Two extensions
//! beyond what `rcl.rs` needed, both authentic to how the Lectionary itself
//! and every reputable secondary source cite it:
//!   - "+" joins non-contiguous verses within a single citation (e.g.
//!     "Ps 80:9+12, 13–14" = verses 9 and 12–14, skipping 10–11). This is
//!     the Lectionary's own notation, not an invention.
//!   - " or " separates an alternate/shorter reading option that the
//!     Missal itself offers (e.g. a long-form Gospel and a short-form
//!     excerpt). Where a reading is genuinely optional in the Missal
//!     (common alternates offered for Year B/C on days where Year A has a
//!     fixed set), that's noted with " or " between the alternatives, or
//!     with a `// ` comment above the entry when the two options draw on
//!     different biblical books entirely.
//!
//! ── Ordinary Time numbering (structural note) ───────────────────────────
//! Unlike RCL's "Propers" (pinned to specific calendar dates), Catholic
//! Ordinary Time Sundays are numbered *sequentially* from 2 (the Sunday
//! after the Baptism of the Lord) through 34 (Christ the King, the Sunday
//! immediately before Advent 1) — see `catholic.rs` for the date-counting
//! logic. Because Easter's date moves, not every number 2–34 is actually
//! used as a Sunday in every calendar year (e.g. numbers in the low-to-mid
//! 20s are sometimes skipped when Lent starts early, or a numbered Sunday
//! is displaced by a higher-ranking feast like Trinity, Corpus Christi, or
//! a solemnity that happens to fall on a Sunday) — this table nonetheless
//! includes the full 2–34 reference set for A/B/C, since the calendar
//! engine in `catholic.rs` is responsible for picking the right number for
//! a given year, not this data file.
//!
//! Trinity Sunday and Corpus Christi are true overrides of whatever
//! Ordinary Sunday number would otherwise fall on those dates — the reader
//! should look them up by their own week ids ("Trinity", "CorpusChristi"),
//! not by number. Corpus Christi is a holy day of obligation on the
//! Thursday after Trinity Sunday everywhere in the universal Church, but
//! the US (like several other conferences) transfers the public
//! celebration to the following Sunday — hence its inclusion here as a
//! Sunday entry. Christ the King is always Ordinary Sunday 34 exactly (no
//! separate week id needed — `catholic.rs` maps "Christ the King" directly
//! to the "Ordinary34" key).
//!
//! Ascension: many US dioceses (by regional Episcopal Conference decree)
//! transfer Ascension Thursday to the following Sunday (the 7th Sunday of
//! Easter), while others keep it on Thursday and observe Easter 7 with its
//! own proper readings. Both "Ascension" and "Easter7" week ids are
//! included below as distinct entries — this file doesn't pick one
//! observance over the other; that's a calendar/config decision for
//! `catholic.rs` or app settings, not for this data table.
//!
//! Lent 1–5: standard (non-Scrutiny) A/B/C readings are given as the
//! primary data per the task brief. Note for RCIA parishes: Year A's Lent
//! 3/4/5 Gospels (the Samaritan woman at the well, the man born blind, the
//! raising of Lazarus — all from John) are also permitted as substitutes
//! in Years B and C in parishes with elect preparing for baptism at the
//! Easter Vigil (the Scrutiny Gospels). Those alternates are not tabulated
//! separately here.
//!
//! Easter Vigil: the Missal offers a genuine menu of Old Testament
//! readings (commonly 7, minimum 3 — the Exodus/Red Sea reading is never
//! omitted), each paired with its own responsorial psalm/canticle, before
//! the fixed epistle (Rom 6:3–11) and year-specific Gospel. There is no
//! single canonical "the" OT+Psalm reading. Rather than invent one, the
//! entries below give the Exodus reading (Exod 14:15–15:1, "never
//! omitted") with its canticle as a representative choice, plus the fixed
//! epistle, plus the Gospel proper to each year — and this comment
//! documents the other widely-used options so nothing is silently implied
//! to be the only correct choice:
//!   1. Gen 1:1–2:2 (or 1:1, 26–31a) — Ps 104 or Ps 33
//!   2. Gen 22:1–18 (or 22:1–2, 9a, 10–13, 15–18) — Ps 16
//!   3. Exod 14:15–15:1 (never omitted) — Exod 15:1–6, 17–18 (canticle)
//!   4. Isa 54:5–14 — Ps 30
//!   5. Isa 55:1–11 — Isa 12:2–3, 4bcd, 5–6 (canticle)
//!   6. Bar 3:9–15, 32–4:4 — Ps 19
//!   7. Ezek 36:16–17a, 18–28 — Ps 42–43, or Isa 12, or Ps 51

pub static READINGS: &[(&str, &str, (&str, &str, &str, &str))] = &[
    // ── Advent ───────────────────────────────────────────────────────────
    ("A", "Advent1", ("Isa 2:1–5", "Ps 122:1–2, 3–4a, 4b–5, 6–7, 8–9", "Rom 13:11–14", "Matt 24:37–44")),
    ("A", "Advent2", ("Isa 11:1–10", "Ps 72:1–2, 7–8, 12–13, 17", "Rom 15:4–9", "Matt 3:1–12")),
    ("A", "Advent3", ("Isa 35:1–6a, 10", "Ps 146:6c–7, 8–9a, 9b–10", "Jas 5:7–10", "Matt 11:2–11")),
    ("A", "Advent4", ("Isa 7:10–14", "Ps 24:1–2, 3–4, 5–6", "Rom 1:1–7", "Matt 1:18–24")),
    ("B", "Advent1", ("Isa 63:16b–17, 19b; 64:2–7", "Ps 80:2–3, 15–16, 18–19", "1 Cor 1:3–9", "Mark 13:33–37")),
    ("B", "Advent2", ("Isa 40:1–5, 9–11", "Ps 85:9ab+10, 11–12, 13–14", "2 Pet 3:8–14", "Mark 1:1–8")),
    ("B", "Advent3", ("Isa 61:1–2a, 10–11", "Luke 1:46–48, 49–50, 53–54", "1 Thess 5:16–24", "John 1:6–8, 19–28")),
    ("B", "Advent4", ("2 Sam 7:1–5, 8b–12, 14a, 16", "Ps 89:2–3, 4–5, 27+29", "Rom 16:25–27", "Luke 1:26–38")),
    ("C", "Advent1", ("Jer 33:14–16", "Ps 25:4–5, 8–9, 10+14", "1 Thess 3:12–4:2", "Luke 21:25–28, 34–36")),
    ("C", "Advent2", ("Bar 5:1–9", "Ps 126:1–2a, 2b–3, 4–5, 6", "Phil 1:4–6, 8–11", "Luke 3:1–6")),
    ("C", "Advent3", ("Zeph 3:14–18a", "Isa 12:2–3, 4bcd, 5–6", "Phil 4:4–7", "Luke 3:10–18")),
    ("C", "Advent4", ("Mic 5:1–4a", "Ps 80:2–3, 15–16, 18–19", "Heb 10:5–10", "Luke 1:39–45")),

    // ── Christmas / Mary, Mother of God / Epiphany / Baptism ────────────
    ("ALL", "Christmas", ("Isa 52:7–10", "Ps 98:1, 2–3a, 3b–4, 5–6", "Heb 1:1–6", "John 1:1–18 or 1:1–5, 9–14")),
    ("A", "HolyFamily", ("Sir 3:3–7, 14–17a", "Ps 128:1–2, 3, 4–5", "Col 3:12–21", "Matt 2:13–15, 19–23")),
    // Year B/C Holy Family: OT/Psalm/Epistle each offer a year-specific
    // option (given here) or the common Sir 3 / Ps 128 / Col 3:12–17 set
    // used for Year A; the Gospel is fixed and distinct per year.
    ("B", "HolyFamily", ("Gen 15:1–6; 21:1–3 or Sir 3:2–6, 12–14", "Ps 105:1–2, 3–4, 5–6, 8–9 or Ps 128:1–2, 3, 4–5", "Heb 11:8, 11–12, 17–19 or Col 3:12–17", "Luke 2:22–40")),
    ("C", "HolyFamily", ("1 Sam 1:20–22, 24–28 or Sir 3:2–6, 12–14", "Ps 84:2–3, 5–6, 9–10 or Ps 128:1–2, 3, 4–5", "1 John 3:1–2, 21–24 or Col 3:12–17", "Luke 2:41–52")),
    ("ALL", "MaryMotherOfGod", ("Num 6:22–27", "Ps 67:2–3, 5, 6+8", "Gal 4:4–7", "Luke 2:16–21")),
    ("ALL", "Epiphany", ("Isa 60:1–6", "Ps 72:1–2, 7–8, 10–11, 12–13", "Eph 3:2–3a, 5–6", "Matt 2:1–12")),
    ("A", "BaptismOfLord", ("Isa 42:1–4, 6–7", "Ps 29:1–2, 3ac+4, 3b+9b–10", "Acts 10:34–38", "Matt 3:13–17")),
    // Year B/C Baptism of the Lord: the year-specific OT/Psalm/Epistle
    // option is given here; the common Isa 42:1–4, 6–7 / Ps 29 / Acts
    // 10:34–38 set (Year A's) is also permitted in B and C.
    ("B", "BaptismOfLord", ("Isa 55:1–11", "Isa 12:2–3, 4bcd, 5–6", "1 John 5:1–9", "Mark 1:7–11")),
    ("C", "BaptismOfLord", ("Isa 40:1–5, 9–11", "Ps 104:1b–2, 3–4, 24–25, 27–28, 29b–30", "Titus 2:11–14; 3:4–7", "Luke 3:15–16, 21–22")),

    // ── Ordinary Time, Year A (2–34) ─────────────────────────────────────
    ("A", "Ordinary2", ("Isa 49:3, 5–6", "Ps 40:2+4, 7–8a, 8b–9, 10", "1 Cor 1:1–3", "John 1:29–34")),
    ("A", "Ordinary3", ("Isa 8:23b–9:3", "Ps 27:1, 4, 13–14", "1 Cor 1:10–13, 17", "Matt 4:12–23")),
    ("A", "Ordinary4", ("Zeph 2:3; 3:12–13", "Ps 146:6c–7, 8–9a, 9b–10", "1 Cor 1:26–31", "Matt 5:1–12a")),
    ("A", "Ordinary5", ("Isa 58:7–10", "Ps 112:4–5, 6–7, 8–9", "1 Cor 2:1–5", "Matt 5:13–16")),
    ("A", "Ordinary6", ("Sir 15:16–21", "Ps 119:1–2, 4–5, 17–18, 33–34", "1 Cor 2:6–10", "Matt 5:17–37")),
    ("A", "Ordinary7", ("Lev 19:1–2, 17–18", "Ps 103:1–2, 3–4, 8+10, 12–13", "1 Cor 3:16–23", "Matt 5:38–48")),
    ("A", "Ordinary8", ("Isa 49:14–15", "Ps 62:2–3, 6–7, 8–9", "1 Cor 4:1–5", "Matt 6:24–34")),
    ("A", "Ordinary9", ("Deut 11:18, 26–28, 32", "Ps 31:2–3a, 3b–4, 17+25", "Rom 3:21–25, 28", "Matt 7:21–27")),
    ("A", "Ordinary10", ("Hos 6:3–6", "Ps 50:1+8, 12–13, 14–15", "Rom 4:18–25", "Matt 9:9–13")),
    ("A", "Ordinary11", ("Exod 19:2–6a", "Ps 100:1–2, 3, 5", "Rom 5:6–11", "Matt 9:36–10:8")),
    ("A", "Ordinary12", ("Jer 20:10–13", "Ps 69:8–10, 14+17, 33–35", "Rom 5:12–15", "Matt 10:26–33")),
    ("A", "Ordinary13", ("2 Kgs 4:8–11, 14–16a", "Ps 89:2–3, 16–17, 18–19", "Rom 6:3–4, 8–11", "Matt 10:37–42")),
    ("A", "Ordinary14", ("Zech 9:9–10", "Ps 145:1–2, 8–9, 10–11, 13–14", "Rom 8:9, 11–13", "Matt 11:25–30")),
    ("A", "Ordinary15", ("Isa 55:10–11", "Ps 65:10, 11, 12–13, 14", "Rom 8:18–23", "Matt 13:1–23 or 13:1–9")),
    ("A", "Ordinary16", ("Wis 12:13, 16–19", "Ps 86:5–6, 9–10, 15–16", "Rom 8:26–27", "Matt 13:24–43 or 13:24–30")),
    ("A", "Ordinary17", ("1 Kgs 3:5, 7–12", "Ps 119:57+72, 76–77, 127–128, 129–130", "Rom 8:28–30", "Matt 13:44–52 or 13:44–46")),
    ("A", "Ordinary18", ("Isa 55:1–3", "Ps 145:8–9, 15–16, 17–18", "Rom 8:35, 37–39", "Matt 14:13–21")),
    ("A", "Ordinary19", ("1 Kgs 19:9a, 11–13a", "Ps 85:9ab+10, 11–12, 13–14", "Rom 9:1–5", "Matt 14:22–33")),
    ("A", "Ordinary20", ("Isa 56:1, 6–7", "Ps 67:2–3, 5, 6+8", "Rom 11:13–15, 29–32", "Matt 15:21–28")),
    ("A", "Ordinary21", ("Isa 22:19–23", "Ps 138:1–2a, 2b–3, 6+8", "Rom 11:33–36", "Matt 16:13–20")),
    ("A", "Ordinary22", ("Jer 20:7–9", "Ps 63:2, 3–4, 5–6, 8–9", "Rom 12:1–2", "Matt 16:21–27")),
    ("A", "Ordinary23", ("Ezek 33:7–9", "Ps 95:1–2, 6–7b, 7c–9", "Rom 13:8–10", "Matt 18:15–20")),
    ("A", "Ordinary24", ("Sir 27:30–28:7", "Ps 103:1–2, 3–4, 9–10, 11–12", "Rom 14:7–9", "Matt 18:21–35")),
    ("A", "Ordinary25", ("Isa 55:6–9", "Ps 145:2–3, 8–9, 17–18", "Phil 1:20c–24, 27a", "Matt 20:1–16a")),
    ("A", "Ordinary26", ("Ezek 18:25–28", "Ps 25:4–5, 6–7, 8–9", "Phil 2:1–11 or 2:1–5", "Matt 21:28–32")),
    ("A", "Ordinary27", ("Isa 5:1–7", "Ps 80:9+12, 13–14, 15–16, 19–20", "Phil 4:6–9", "Matt 21:33–43")),
    ("A", "Ordinary28", ("Isa 25:6–10a", "Ps 23:1–3a, 3b–4, 5, 6", "Phil 4:12–14, 19–20", "Matt 22:1–14 or 22:1–10")),
    ("A", "Ordinary29", ("Isa 45:1, 4–6", "Ps 96:1+3, 4–5, 7–8, 9–10", "1 Thess 1:1–5b", "Matt 22:15–21")),
    ("A", "Ordinary30", ("Exod 22:20–26", "Ps 18:2–3a, 3b–4, 47+51", "1 Thess 1:5c–10", "Matt 22:34–40")),
    ("A", "Ordinary31", ("Mal 1:14b–2:2b, 8–10", "Ps 131:1, 2, 3", "1 Thess 2:7b–9, 13", "Matt 23:1–12")),
    ("A", "Ordinary32", ("Wis 6:12–16", "Ps 63:2, 3–4, 5–6, 7–8", "1 Thess 4:13–18 or 4:13–14", "Matt 25:1–13")),
    ("A", "Ordinary33", ("Prov 31:10–13, 19–20, 30–31", "Ps 128:1–2, 3, 4–5", "1 Thess 5:1–6", "Matt 25:14–30 or 25:14–15, 19–21")),
    ("A", "Ordinary34", ("Ezek 34:11–12, 15–17", "Ps 23:1–2a, 2b–3, 5, 6", "1 Cor 15:20–26, 28", "Matt 25:31–46")),

    // ── Ordinary Time, Year B (2–34) ─────────────────────────────────────
    ("B", "Ordinary2", ("1 Sam 3:3b–10, 19", "Ps 40:2+4, 7–8a, 8b–9, 10", "1 Cor 6:13c–15a, 17–20", "John 1:35–42")),
    ("B", "Ordinary3", ("Jon 3:1–5, 10", "Ps 25:4–5, 6–7, 8–9", "1 Cor 7:29–31", "Mark 1:14–20")),
    ("B", "Ordinary4", ("Deut 18:15–20", "Ps 95:1–2, 6–7b, 7c–9", "1 Cor 7:32–35", "Mark 1:21–28")),
    ("B", "Ordinary5", ("Job 7:1–4, 6–7", "Ps 147:1–2, 3–4, 5–6", "1 Cor 9:16–19, 22–23", "Mark 1:29–39")),
    ("B", "Ordinary6", ("Lev 13:1–2, 44–46", "Ps 32:1–2, 5, 11", "1 Cor 10:31–11:1", "Mark 1:40–45")),
    ("B", "Ordinary7", ("Isa 43:18–19, 21–22, 24b–25", "Ps 41:2–3, 4–5, 13–14", "2 Cor 1:18–22", "Mark 2:1–12")),
    ("B", "Ordinary8", ("Hos 2:16b, 17b, 21–22", "Ps 103:1–2, 3–4, 8+10, 12–13", "2 Cor 3:1b–6", "Mark 2:18–22")),
    ("B", "Ordinary9", ("Deut 5:12–15", "Ps 81:3–4, 5–6, 7–8, 10–11", "2 Cor 4:6–11", "Mark 2:23–3:6")),
    ("B", "Ordinary10", ("Gen 3:9–15", "Ps 130:1–2, 3–4, 5–6, 7–8", "2 Cor 4:13–5:1", "Mark 3:20–35")),
    ("B", "Ordinary11", ("Ezek 17:22–24", "Ps 92:2–3, 13–14, 15–16", "2 Cor 5:6–10", "Mark 4:26–34")),
    ("B", "Ordinary12", ("Job 38:1, 8–11", "Ps 107:23–24, 25–26, 28–29, 30–31", "2 Cor 5:14–17", "Mark 4:35–41")),
    ("B", "Ordinary13", ("Wis 1:13–15; 2:23–24", "Ps 30:2+4, 5–6, 11–12a+13b", "2 Cor 8:7, 9, 13–15", "Mark 5:21–43")),
    ("B", "Ordinary14", ("Ezek 2:2–5", "Ps 123:1–2a, 2bc, 3–4", "2 Cor 12:7–10", "Mark 6:1–6")),
    ("B", "Ordinary15", ("Amos 7:12–15", "Ps 85:9ab+10, 11–12, 13–14", "Eph 1:3–14", "Mark 6:7–13")),
    ("B", "Ordinary16", ("Jer 23:1–6", "Ps 23:1–3a, 3b–4, 5, 6", "Eph 2:13–18", "Mark 6:30–34")),
    ("B", "Ordinary17", ("2 Kgs 4:42–44", "Ps 145:10–11, 15–16, 17–18", "Eph 4:1–6", "John 6:1–15")),
    ("B", "Ordinary18", ("Exod 16:2–4, 12–15", "Ps 78:3–4, 23–24, 25+54", "Eph 4:17, 20–24", "John 6:24–35")),
    ("B", "Ordinary19", ("1 Kgs 19:4–8", "Ps 34:2–3, 4–5, 6–7, 8–9", "Eph 4:30–5:2", "John 6:41–51")),
    ("B", "Ordinary20", ("Prov 9:1–6", "Ps 34:2–3, 4–5, 6–7", "Eph 5:15–20", "John 6:51–58")),
    ("B", "Ordinary21", ("Josh 24:1–2a, 15–17, 18b", "Ps 34:2–3, 16–17, 18–19, 20–21", "Eph 5:21–32", "John 6:60–69")),
    ("B", "Ordinary22", ("Deut 4:1–2, 6–8", "Ps 15:2–3a, 3b–4a, 4b–5", "Jas 1:17–18, 21b–22, 27", "Mark 7:1–8, 14–15, 21–23")),
    ("B", "Ordinary23", ("Isa 35:4–7a", "Ps 146:6c–7, 8–9a, 9b–10", "Jas 2:1–5", "Mark 7:31–37")),
    ("B", "Ordinary24", ("Isa 50:4–9a", "Ps 116:1–2, 3–4, 5–6, 8–9", "Jas 2:14–18", "Mark 8:27–35")),
    ("B", "Ordinary25", ("Wis 2:12, 17–20", "Ps 54:3–4, 5, 6–8", "Jas 3:16–4:3", "Mark 9:30–37")),
    ("B", "Ordinary26", ("Num 11:25–29", "Ps 19:8, 10, 12–13, 14", "Jas 5:1–6", "Mark 9:38–43, 45, 47–48")),
    ("B", "Ordinary27", ("Gen 2:18–24", "Ps 128:1–2, 3, 4–5, 6", "Heb 2:9–11", "Mark 10:2–16")),
    ("B", "Ordinary28", ("Wis 7:7–11", "Ps 90:12–13, 14–15, 16–17", "Heb 4:12–13", "Mark 10:17–30")),
    ("B", "Ordinary29", ("Isa 53:10–11", "Ps 33:4–5, 18–19, 20+22", "Heb 4:14–16", "Mark 10:35–45")),
    ("B", "Ordinary30", ("Jer 31:7–9", "Ps 126:1–2a, 2b–3, 4–5, 6", "Heb 5:1–6", "Mark 10:46–52")),
    ("B", "Ordinary31", ("Deut 6:2–6", "Ps 18:2–3a, 3b–4, 47+51", "Heb 7:23–28", "Mark 12:28b–34")),
    ("B", "Ordinary32", ("1 Kgs 17:10–16", "Ps 146:6c–7, 8–9a, 9b–10", "Heb 9:24–28", "Mark 12:38–44")),
    ("B", "Ordinary33", ("Dan 12:1–3", "Ps 16:5+8, 9–10, 11", "Heb 10:11–14, 18", "Mark 13:24–32")),
    ("B", "Ordinary34", ("Dan 7:13–14", "Ps 93:1a, 1b–2, 5", "Rev 1:5–8", "John 18:33b–37")),

    // ── Ordinary Time, Year C (2–34) ─────────────────────────────────────
    ("C", "Ordinary2", ("Isa 62:1–5", "Ps 96:1–2a, 2b–3, 7–8, 9–10", "1 Cor 12:4–11", "John 2:1–11")),
    ("C", "Ordinary3", ("Neh 8:2–4a, 5–6, 8–10", "Ps 19:8, 9, 10, 15", "1 Cor 12:12–30 or 12:12–14, 27", "Luke 1:1–4; 4:14–21")),
    ("C", "Ordinary4", ("Jer 1:4–5, 17–19", "Ps 71:1–2, 3–4, 5–6, 15+17", "1 Cor 12:31–13:13 or 13:4–13", "Luke 4:21–30")),
    ("C", "Ordinary5", ("Isa 6:1–2a, 3–8", "Ps 138:1–2a, 2b–3, 4–5, 7–8", "1 Cor 15:1–11 or 15:3–8, 11", "Luke 5:1–11")),
    ("C", "Ordinary6", ("Jer 17:5–8", "Ps 1:1–2, 3, 4+6", "1 Cor 15:12, 16–20", "Luke 6:17, 20–26")),
    ("C", "Ordinary7", ("1 Sam 26:2, 7–9, 12–13, 22–23", "Ps 103:1–2, 3–4, 8+10, 12–13", "1 Cor 15:45–49", "Luke 6:27–38")),
    ("C", "Ordinary8", ("Sir 27:5–8", "Ps 92:2–3, 13–14, 15–16", "1 Cor 15:54–58", "Luke 6:39–45")),
    ("C", "Ordinary9", ("1 Kgs 8:41–43", "Ps 117:1–2", "Gal 1:1–2, 6–10", "Luke 7:1–10")),
    ("C", "Ordinary10", ("1 Kgs 17:17–24", "Ps 30:2+4, 5–6, 11–12a+13b", "Gal 1:11–19", "Luke 7:11–17")),
    ("C", "Ordinary11", ("2 Sam 12:7–10, 13", "Ps 32:1–2, 5, 7, 11", "Gal 2:16, 19–21", "Luke 7:36–8:3 or 7:36–50")),
    ("C", "Ordinary12", ("Zech 12:10–11; 13:1", "Ps 63:2, 3–4, 5–6, 8–9", "Gal 3:26–29", "Luke 9:18–24")),
    ("C", "Ordinary13", ("1 Kgs 19:16b, 19–21", "Ps 16:1–2a+5, 7–8, 9–10, 11", "Gal 5:1, 13–18", "Luke 9:51–62")),
    ("C", "Ordinary14", ("Isa 66:10–14c", "Ps 66:1–3, 4–5, 6–7, 16+20", "Gal 6:14–18", "Luke 10:1–12, 17–20 or 10:1–9")),
    ("C", "Ordinary15", ("Deut 30:10–14", "Ps 69:14+17, 30–31, 33–34, 36a+37 or Ps 19:8, 9, 10, 11", "Col 1:15–20", "Luke 10:25–37")),
    ("C", "Ordinary16", ("Gen 18:1–10a", "Ps 15:2–3a, 3b–4, 5", "Col 1:24–28", "Luke 10:38–42")),
    ("C", "Ordinary17", ("Gen 18:20–32", "Ps 138:1–2a, 2b–3, 6–7a, 7b–8", "Col 2:12–14", "Luke 11:1–13")),
    ("C", "Ordinary18", ("Eccl 1:2; 2:21–23", "Ps 90:3–4, 5–6, 12–13, 14+17", "Col 3:1–5, 9–11", "Luke 12:13–21")),
    ("C", "Ordinary19", ("Wis 18:6–9", "Ps 33:1+12, 18–19, 20–22", "Heb 11:1–2, 8–19 or 11:1–2, 8–12", "Luke 12:32–48 or 12:35–40")),
    ("C", "Ordinary20", ("Jer 38:4–6, 8–10", "Ps 40:2, 3, 4, 18", "Heb 12:1–4", "Luke 12:49–53")),
    ("C", "Ordinary21", ("Isa 66:18–21", "Ps 117:1, 2", "Heb 12:5–7, 11–13", "Luke 13:22–30")),
    ("C", "Ordinary22", ("Sir 3:17–18, 20, 28–29", "Ps 68:4–5, 6–7, 10–11", "Heb 12:18–19, 22–24a", "Luke 14:1, 7–14")),
    ("C", "Ordinary23", ("Wis 9:13–18b", "Ps 90:3–4, 5–6, 12–13, 14+17", "Phlm 9–10, 12–17", "Luke 14:25–33")),
    ("C", "Ordinary24", ("Exod 32:7–11, 13–14", "Ps 51:3–4, 12–13, 17+19", "1 Tim 1:12–17", "Luke 15:1–32 or 15:1–10")),
    ("C", "Ordinary25", ("Amos 8:4–7", "Ps 113:1–2, 4–6, 7–8", "1 Tim 2:1–8", "Luke 16:1–13 or 16:10–13")),
    ("C", "Ordinary26", ("Amos 6:1a, 4–7", "Ps 146:6c–7, 8–9a, 9b–10", "1 Tim 6:11–16", "Luke 16:19–31")),
    ("C", "Ordinary27", ("Hab 1:2–3; 2:2–4", "Ps 95:1–2, 6–7b, 7c–9", "2 Tim 1:6–8, 13–14", "Luke 17:5–10")),
    ("C", "Ordinary28", ("2 Kgs 5:14–17", "Ps 98:1, 2–3a, 3b–4", "2 Tim 2:8–13", "Luke 17:11–19")),
    ("C", "Ordinary29", ("Exod 17:8–13", "Ps 121:1–2, 3–4, 5–6, 7–8", "2 Tim 3:14–4:2", "Luke 18:1–8")),
    ("C", "Ordinary30", ("Sir 35:12–14, 16–18", "Ps 34:2–3, 17–18, 19+23", "2 Tim 4:6–8, 16–18", "Luke 18:9–14")),
    ("C", "Ordinary31", ("Wis 11:22–12:2", "Ps 145:1–2, 8–9, 10–11, 13b–14", "2 Thess 1:11–2:2", "Luke 19:1–10")),
    ("C", "Ordinary32", ("2 Macc 7:1–2, 9–14", "Ps 17:1, 5–6, 8+15", "2 Thess 2:16–3:5", "Luke 20:27–38 or 20:27, 34–38")),
    ("C", "Ordinary33", ("Mal 3:19–20a", "Ps 98:5–6, 7–8, 9", "2 Thess 3:7–12", "Luke 21:5–19")),
    ("C", "Ordinary34", ("2 Sam 5:1–3", "Ps 122:1–2, 3–4a, 4b–5", "Col 1:12–20", "Luke 23:35–43")),

    // ── Ash Wednesday ────────────────────────────────────────────────────
    ("ALL", "AshWednesday", ("Joel 2:12–18", "Ps 51:3–4, 5–6ab, 12–13, 14+17", "2 Cor 5:20–6:2", "Matt 6:1–6, 16–18")),

    // ── Lent (standard, non-Scrutiny) ────────────────────────────────────
    ("A", "Lent1", ("Gen 2:7–9; 3:1–7", "Ps 51:3–4, 5–6, 12–13, 14+17", "Rom 5:12–19 or 5:12, 17–19", "Matt 4:1–11")),
    ("A", "Lent2", ("Gen 12:1–4a", "Ps 33:4–5, 18–19, 20+22", "2 Tim 1:8b–10", "Matt 17:1–9")),
    // Also permitted in Years B/C for parishes with the elect (Scrutinies).
    ("A", "Lent3", ("Exod 17:3–7", "Ps 95:1–2, 6–7b, 7c–9", "Rom 5:1–2, 5–8", "John 4:5–42 or 4:5–15, 19b–26, 39a, 40–42")),
    ("A", "Lent4", ("1 Sam 16:1b, 6–7, 10–13a", "Ps 23:1–3a, 3b–4, 5, 6", "Eph 5:8–14", "John 9:1–41 or 9:1, 6–9, 13–17, 34–38")),
    ("A", "Lent5", ("Ezek 37:12–14", "Ps 130:1–2, 3–4, 5–6, 7–8", "Rom 8:8–11", "John 11:1–45 or 11:3–7, 17, 20–27, 33b–45")),
    ("B", "Lent1", ("Gen 9:8–15", "Ps 25:4–5, 6–7, 8–9", "1 Pet 3:18–22", "Mark 1:12–15")),
    ("B", "Lent2", ("Gen 22:1–2, 9a, 10–13, 15–18", "Ps 116:10+15, 16–17, 18–19", "Rom 8:31b–34", "Mark 9:2–10")),
    ("B", "Lent3", ("Exod 20:1–17 or 20:1–3, 7–8, 12–17", "Ps 19:8, 9, 10, 11", "1 Cor 1:22–25", "John 2:13–25")),
    ("B", "Lent4", ("2 Chr 36:14–16, 19–23", "Ps 137:1–2, 3, 4–5, 6", "Eph 2:4–10", "John 3:14–21")),
    ("B", "Lent5", ("Jer 31:31–34", "Ps 51:3–4, 12–13, 14–15", "Heb 5:7–9", "John 12:20–33")),
    ("C", "Lent1", ("Deut 26:4–10", "Ps 91:1–2, 10–11, 12–13, 14–15", "Rom 10:8–13", "Luke 4:1–13")),
    ("C", "Lent2", ("Gen 15:5–12, 17–18", "Ps 27:1, 7–8a, 8b–9, 13–14", "Phil 3:17–4:1 or 3:20–4:1", "Luke 9:28b–36")),
    ("C", "Lent3", ("Exod 3:1–8a, 13–15", "Ps 103:1–2, 3–4, 6–7, 8+11", "1 Cor 10:1–6, 10–12", "Luke 13:1–9")),
    ("C", "Lent4", ("Josh 5:9a, 10–12", "Ps 34:2–3, 4–5, 6–7", "2 Cor 5:17–21", "Luke 15:1–3, 11–32")),
    ("C", "Lent5", ("Isa 43:16–21", "Ps 126:1–2a, 2b–3, 4–5, 6", "Phil 3:8–14", "John 8:1–11")),

    // ── Palm Sunday of the Passion ───────────────────────────────────────
    // Gospel at the procession with palms (not tabulated in the 4-tuple
    // below, since that slot holds the Passion Gospel read at Mass):
    //   Year A: Matt 21:1–11 | Year B: Mark 11:1–10 or John 12:12–16 | Year C: Luke 19:28–40
    ("A", "PalmSunday", ("Isa 50:4–7", "Ps 22:8–9, 17–18, 19–20, 23–24", "Phil 2:6–11", "Matt 26:14–27:66 or 27:11–54")),
    ("B", "PalmSunday", ("Isa 50:4–7", "Ps 22:8–9, 17–18, 19–20, 23–24", "Phil 2:6–11", "Mark 14:1–15:47 or 15:1–39")),
    ("C", "PalmSunday", ("Isa 50:4–7", "Ps 22:8–9, 17–18, 19–20, 23–24", "Phil 2:6–11", "Luke 22:14–23:56 or 23:1–49")),

    // ── Triduum ──────────────────────────────────────────────────────────
    ("ALL", "HolyThursday", ("Exod 12:1–8, 11–14", "Ps 116:12–13, 15–16, 17–18", "1 Cor 11:23–26", "John 13:1–15")),
    ("ALL", "GoodFriday", ("Isa 52:13–53:12", "Ps 31:2+6, 12–13, 15–16, 17+25", "Heb 4:14–16; 5:7–9", "John 18:1–19:42")),
    // Easter Vigil — see file-level comment on the OT reading menu. Epistle
    // is fixed for all years; Gospel is fixed per year.
    ("A", "EasterVigil", ("Exod 14:15–15:1", "Exod 15:1–6, 17–18", "Rom 6:3–11", "Matt 28:1–10")),
    ("B", "EasterVigil", ("Exod 14:15–15:1", "Exod 15:1–6, 17–18", "Rom 6:3–11", "Mark 16:1–7")),
    ("C", "EasterVigil", ("Exod 14:15–15:1", "Exod 15:1–6, 17–18", "Rom 6:3–11", "Luke 24:1–12")),

    // ── Easter ───────────────────────────────────────────────────────────
    ("ALL", "Easter", ("Acts 10:34a, 37–43", "Ps 118:1–2, 16–17, 22–23", "Col 3:1–4 or 1 Cor 5:6b–8", "John 20:1–9 or Luke 24:13–35")),
    ("A", "Easter2", ("Acts 2:42–47", "Ps 118:2–4, 13–15, 22–24", "1 Pet 1:3–9", "John 20:19–31")),
    ("A", "Easter3", ("Acts 2:14, 22–33", "Ps 16:1–2a+5, 7–8, 9–10, 11", "1 Pet 1:17–21", "Luke 24:13–35")),
    ("A", "Easter4", ("Acts 2:14a, 36–41", "Ps 23:1–3a, 3b–4, 5, 6", "1 Pet 2:20b–25", "John 10:1–10")),
    ("A", "Easter5", ("Acts 6:1–7", "Ps 33:1–2, 4–5, 18–19", "1 Pet 2:4–9", "John 14:1–12")),
    ("A", "Easter6", ("Acts 8:5–8, 14–17", "Ps 66:1–3, 4–5, 6–7, 16+20", "1 Pet 3:15–18", "John 14:15–21")),
    ("A", "Easter7", ("Acts 1:12–14", "Ps 27:1, 4, 7–8", "1 Pet 4:13–16", "John 17:1–11a")),
    ("B", "Easter2", ("Acts 4:32–35", "Ps 118:2–4, 13–15, 22–24", "1 John 5:1–6", "John 20:19–31")),
    ("B", "Easter3", ("Acts 3:13–15, 17–19", "Ps 4:2, 4, 7–8, 9", "1 John 2:1–5a", "Luke 24:35–48")),
    ("B", "Easter4", ("Acts 4:8–12", "Ps 118:1, 8–9, 21–23, 26, 28–29", "1 John 3:1–2", "John 10:11–18")),
    ("B", "Easter5", ("Acts 9:26–31", "Ps 22:26–27, 28, 30, 31–32", "1 John 3:18–24", "John 15:1–8")),
    ("B", "Easter6", ("Acts 10:25–26, 34–35, 44–48", "Ps 98:1, 2–3, 3–4", "1 John 4:7–10", "John 15:9–17")),
    ("B", "Easter7", ("Acts 1:15–17, 20a, 20c–26", "Ps 103:1–2, 11–12, 19–20", "1 John 4:11–16", "John 17:11b–19")),
    ("C", "Easter2", ("Acts 5:12–16", "Ps 118:2–4, 13–15, 22–24", "Rev 1:9–11a, 12–13, 17–19", "John 20:19–31")),
    ("C", "Easter3", ("Acts 5:27–32, 40b–41", "Ps 30:2+4, 5–6, 11–12a+13b", "Rev 5:11–14", "John 21:1–19 or 21:1–14")),
    ("C", "Easter4", ("Acts 13:14, 43–52", "Ps 100:1–2, 3, 5", "Rev 7:9, 14b–17", "John 10:27–30")),
    ("C", "Easter5", ("Acts 14:21–27", "Ps 145:8–9, 10–11, 12–13", "Rev 21:1–5a", "John 13:31–33a, 34–35")),
    ("C", "Easter6", ("Acts 15:1–2, 22–29", "Ps 67:2–3, 5, 6+8", "Rev 21:10–14, 22–23", "John 14:23–29")),
    ("C", "Easter7", ("Acts 7:55–60", "Ps 97:1–2, 6–7, 9", "Rev 22:12–14, 16–17, 20", "John 17:20–26")),

    // ── Ascension (see file-level comment on the US transfer-to-Easter-7
    // ambiguity — this app must not silently pick one observance) ───────
    ("A", "Ascension", ("Acts 1:1–11", "Ps 47:2–3, 6–7, 8–9", "Eph 1:17–23", "Matt 28:16–20")),
    ("B", "Ascension", ("Acts 1:1–11", "Ps 47:2–3, 6–7, 8–9", "Eph 4:1–13 or 4:1–7, 11–13", "Mark 16:15–20")),
    ("C", "Ascension", ("Acts 1:1–11", "Ps 47:2–3, 6–7, 8–9", "Heb 9:24–28; 10:19–23", "Luke 24:46–53")),

    // ── Pentecost (Mass During the Day) ──────────────────────────────────
    // Year A's Epistle/Gospel are also permitted as the common set in B/C;
    // the year-specific alternates are given here as the primary data.
    ("A", "Pentecost", ("Acts 2:1–11", "Ps 104:1+24, 29–30, 31+34", "1 Cor 12:3b–7, 12–13", "John 20:19–23")),
    ("B", "Pentecost", ("Acts 2:1–11", "Ps 104:1+24, 29–30, 31+34", "Gal 5:16–25", "John 15:26–27; 16:12–15")),
    ("C", "Pentecost", ("Acts 2:1–11", "Ps 104:1+24, 29–30, 31+34", "Rom 8:8–17", "John 14:15–16, 23b–26")),

    // ── Trinity Sunday (overrides Ordinary numbering) ────────────────────
    ("A", "Trinity", ("Exod 34:4b–6, 8–9", "Dan 3:52, 53, 54, 55", "2 Cor 13:11–13", "John 3:16–18")),
    ("B", "Trinity", ("Deut 4:32–34, 39–40", "Ps 33:4–5, 6+9, 18–19, 20+22", "Rom 8:14–17", "Matt 28:16–20")),
    ("C", "Trinity", ("Prov 8:22–31", "Ps 8:4–5, 6–7, 8–9", "Rom 5:1–5", "John 16:12–15")),

    // ── Corpus Christi (overrides Ordinary numbering; US transfers the
    // public celebration from Thursday to the following Sunday) ────────
    ("A", "CorpusChristi", ("Deut 8:2–3, 14b–16a", "Ps 147:12–13, 14–15, 19–20", "1 Cor 10:16–17", "John 6:51–58")),
    ("B", "CorpusChristi", ("Exod 24:3–8", "Ps 116:12–13, 15–16, 17–18", "Heb 9:11–15", "Mark 14:12–16, 22–26")),
    ("C", "CorpusChristi", ("Gen 14:18–20", "Ps 110:1, 2, 3, 4", "1 Cor 11:23–26", "Luke 9:11b–17")),

    // Christ the King is always Ordinary Sunday 34 — see the "Ordinary34"
    // entries above (Year A/B/C) rather than a separate week id here.
];
