//! Narrative Lectionary (NL) readings — Luther Seminary / Working Preacher.
//!
//! Source data fetched 2026-07-19 from:
//!   - https://www.workingpreacher.org/home-narrative-lectionary (index page)
//!   - https://www.workingpreacher.org/narrative-faq (structure/FAQ)
//!   - https://www.workingpreacher.org/wp-content/uploads/2025/10/narrative_lectionary_matthew_2026-27.pdf
//!     ("Narrative Lectionary 2026-27, Readings for Year 1 (Matthew)", revised 10/10/2025)
//!   - https://www.workingpreacher.org/wp-content/uploads/2024/02/narrative_lectionary_mark_2023-24_rev2.pdf
//!     ("Narrative Lectionary 2023-24, Readings for Year 2 (Mark)", revised 2/6/24)
//!   - https://www.workingpreacher.org/wp-content/uploads/2023/09/WP-Narrative-Lectionary-2024-25-Luke.pdf
//!     ("Narrative Lectionary 2024-25, Readings for Year 3 (Luke)", revised 8/28/2023)
//!   - https://www.workingpreacher.org/wp-content/uploads/2025/02/WP-Narrative-Lectionary-2025-26-John.pdf
//!     ("Narrative Lectionary 2025-26, Readings for Year 4 (John)", revised 2/20/25)
//!   - https://www.workingpreacher.org/wp-content/uploads/2026/01/NL-summer-readings-2026_rev.pdf
//!     ("Narrative Lectionary Summer 2026 Readings", revised 1/8/2026)
//!
//! narrativelectionary.org 301-redirects to workingpreacher.org/home-narrative-lectionary,
//! which is the live authoritative host for the current reading lists.
//!
//! ## Real structure (as verified from the sources above — do not assume RCL's shape)
//!
//! - **Cycle length and start rule**: a 4-year cycle (Year 1–4), each year keyed to one
//!   gospel — Year 1 = Matthew, Year 2 = Mark, Year 3 = Luke, Year 4 = John. The core
//!   narrative cycle "runs from the Sunday after Labor Day to the Day of Pentecost"
//!   (quoted directly from the Summer 2026 readings PDF). The remaining stretch of the
//!   year — the First Sunday after Pentecost through the Sunday before Labor Day, i.e.
//!   the summer — falls **outside** the 4-year narrative cycle entirely: it's filled by
//!   separate, non-year-specific preaching series (see `SUMMER_2026` below) that Working
//!   Preacher publishes on their own cadence, not tied to Year 1/2/3/4.
//! - **Week identity**: each entry carries an `NLxxx` code (Year 1 → NL1xx, Year 2 →
//!   NL2xx, Year 3 → NL3xx, Year 4 → NL4xx) plus a short pericope title (e.g. "Flood and
//!   Promise", "Binding of Isaac") and a liturgical-season label (e.g. "Sixteenth Sunday
//!   after Pentecost", "First Sunday of Advent", "Ash Wednesday", "Palm/Passion Sunday",
//!   "Day of Pentecost"). The season label is the durable, date-independent key —
//!   analogous to RCL's `Advent1`/`Proper4` — since the literal calendar date in each
//!   source PDF is specific to that one program year and shifts year to year.
//!   **Numbering has intentional gaps** (e.g. Year 1 has NL111 then jumps to NL113;
//!   NL124 then jumps to NL129). Luther Seminary reserves fixed slot numbers per
//!   liturgical occasion across all four years; a slot is only populated in years whose
//!   calendar actually needs that extra Sunday (this happens most often around the
//!   Epiphany season, whose length varies with the date of Easter). The gaps are real
//!   and not a transcription error.
//! - **Readings per week — NOT RCL's fixed 4-slot shape**: most weeks have exactly one
//!   primary preaching text (often a multi-passage citation spanning several verse
//!   ranges) plus one optional "accompanying reading" — a brief gospel excerpt on weeks
//!   before Christmas and after Easter, or a psalm excerpt from Christmas through Easter.
//!   The accompanying reading is explicitly optional per every source PDF's cover note.
//!   A handful of weeks have **two co-equal primary texts** (e.g. Year 1's NL107 pairs
//!   2 Samuel with a psalm; Year 2's NL236 pairs the Triumphal Entry with the Anointing
//!   at Bethany as alternative Palm/Passion Sunday options; Pentecost every year pairs
//!   Acts 2 with an epistle reading) — modeled here as a slice, not a fixed tuple. Two
//!   entries (Year 2 NL218, Year 4 NL436) list an explicit "or"/"Opt." alternate primary
//!   text distinct from the accompanying reading — modeled as `alt_primary`.
//! - **Fall (Sunday after Labor Day → just before Advent)**: Old Testament texts carry
//!   the story from Creation through the united/divided kingdom and exile, walking
//!   Scripture's "overarching story" as advertised; a brief gospel excerpt from the
//!   year's assigned gospel accompanies each week.
//! - **Advent → Christmas**: NL has its own Advent readings, distinct from RCL's Advent
//!   texts — continuing the Old Testament sequence for the first three Sundays of
//!   Advent, then switching the *gospel* reading to primary on the Fourth Sunday of
//!   Advent (an infancy-narrative or prologue text) — explicitly called out on every
//!   year's PDF cover note. Christmas Eve/Day and the one or two Sundays of Christmas
//!   then continue sequentially through the assigned gospel's birth narrative.
//! - **Epiphany → Lent → Holy Week → Easter**: continuous, sequential reading through
//!   the year's assigned gospel, with liturgical labels (Baptism of Our Lord,
//!   Transfiguration, Ash Wednesday, First–Fifth Sunday in Lent, Palm/Passion Sunday,
//!   Maundy Thursday, Good Friday, Resurrection Sunday) attached to whatever gospel
//!   passage falls next in sequence — the passage assigned to e.g. Ash Wednesday is
//!   not the same across all four years, because it's just "whatever comes next" in
//!   that gospel's continuous read-through, not a season-specific set text.
//! - **Easter → Pentecost**: readings shift to Acts and epistles (which epistle depends
//!   on the year), continuing through Pentecost itself (which always pairs Acts 2 with
//!   an epistle reading).
//! - **Summer**: entirely outside the 4-year cycle. Congregations "switch back to RCL"
//!   or use freestanding topical/book-based preaching series that Working Preacher
//!   publishes separately (Ruth & Esther, 1–2 Timothy, Ten Commandments, catechetical
//!   series on the Creeds/Sacraments/Lord's Prayer, etc. — see the Narrative FAQ page for
//!   the back catalog). `SUMMER_2026` below captures the specific summer 2026 lineup as
//!   published; it is explicitly **not** a fixed year-keyed part of the 4-year cycle and
//!   will differ in other calendar years.
//!
//! ## Rust representation chosen here
//!
//! `Week` models one entry in a year's sequence: an NL code, title, season label, one or
//! more primary reading citations (a slice, since some weeks are two-text), an optional
//! alternate primary text (rare), an optional accompanying reading, and the literal
//! month/day the reading fell on in the *source* program year — kept only for
//! traceability back to the PDF, not as an authoritative calendar date (the season label
//! is authoritative; actual date computation for a given real year — anchoring on the
//! Sunday after Labor Day and walking forward, same as RCL's Advent-anchored math in
//! `rcl.rs` — is a separate concern left to whatever calendar module consumes this data).
//!
//! Citation style matches `rcl.rs`: en dash "–" for verse/chapter ranges, same book
//! abbreviations (Isa, Ps, Gen, Exod, 1 Sam, 1 Kgs, Matt, Rom, 1 Cor, Phlm, etc.).
//!
//! Every entry below was transcribed directly from the PDFs cited above. Nothing was
//! reconstructed from memory. Entries that could not be verified are marked `// VERIFY:`
//! — there are none in this file as delivered; all four years and the Summer 2026 series
//! were fetched and confirmed in full.

pub struct Week {
    pub code: &'static str,
    pub title: &'static str,
    pub season: &'static str,
    pub primary: &'static [&'static str],
    pub alt_primary: Option<&'static str>,
    pub accompanying: Option<&'static str>,
    /// Month/day this entry fell on in the specific source program year (see module
    /// doc). Traceability only — not an authoritative calendar date.
    pub source_date: &'static str,
}

/// Year 1 (Matthew) — "Narrative Lectionary 2026-27", 41 weeks, Sept 13, 2026 – May 16, 2027.
pub static YEAR1_MATTHEW: &[Week] = &[
    Week { code: "NL101", title: "Flood and Promise", season: "Sixteenth Sunday after Pentecost", primary: &["Gen 6:5–22; 8:6–12; 9:8–17"], alt_primary: None, accompanying: Some("Matt 8:24–27"), source_date: "September 13" },
    Week { code: "NL102", title: "Call of Abraham", season: "Seventeenth Sunday after Pentecost", primary: &["Gen 12:1–9"], alt_primary: None, accompanying: Some("Matt 28:19–20"), source_date: "September 20" },
    Week { code: "NL103", title: "Joseph in Prison", season: "Eighteenth Sunday after Pentecost", primary: &["Gen 39:1–23"], alt_primary: None, accompanying: Some("Matt 5:11–12"), source_date: "September 27" },
    Week { code: "NL104", title: "Rescue at the Sea", season: "Nineteenth Sunday after Pentecost", primary: &["Exod 14:5–7, 10–14, 21–29"], alt_primary: None, accompanying: Some("Matt 2:13–15"), source_date: "October 4" },
    Week { code: "NL105", title: "Covenant and Commandments", season: "Twentieth Sunday after Pentecost", primary: &["Exod 19:3–7; 20:1–17"], alt_primary: None, accompanying: Some("Matt 5:17"), source_date: "October 11" },
    Week { code: "NL106", title: "Joshua Renews the Covenant", season: "Twenty-first Sunday after Pentecost", primary: &["Josh 24:1–15 [16–26]"], alt_primary: None, accompanying: Some("Matt 4:8–10"), source_date: "October 18" },
    Week { code: "NL107", title: "David and Bathsheba", season: "Twenty-second Sunday after Pentecost", primary: &["2 Sam 11:1–5, 26–27; 12:1–9", "Ps 51:1–9"], alt_primary: None, accompanying: Some("Matt 21:33–41"), source_date: "October 25" },
    Week { code: "NL108", title: "Solomon's Wisdom", season: "Twenty-third Sunday after Pentecost", primary: &["1 Kgs 3:4–9, [10–15], 16–28"], alt_primary: None, accompanying: Some("Matt 6:9–10"), source_date: "November 1" },
    Week { code: "NL109", title: "Elisha Heals Naaman", season: "Twenty-fourth Sunday after Pentecost", primary: &["2 Kgs 5:1–15a"], alt_primary: None, accompanying: Some("Matt 8:2–3"), source_date: "November 8" },
    Week { code: "NL110", title: "Micah", season: "Twenty-fifth Sunday after Pentecost", primary: &["Mic [1:3–5]; 5:2–5a; 6:6–8"], alt_primary: None, accompanying: Some("Matt 9:13"), source_date: "November 15" },
    Week { code: "NL111", title: "Swords into Plowshares", season: "Christ the King / Reign of Christ Sunday", primary: &["Isa 36:1–3, 13–20; 37:1–7; 2:1–4"], alt_primary: None, accompanying: Some("Matt 5:14"), source_date: "November 22" },
    Week { code: "NL113", title: "Faith as a Way of Life", season: "First Sunday of Advent", primary: &["Hab 1:1–7; 2:1–4; 3:[3b–6], 17–19"], alt_primary: None, accompanying: Some("Matt 26:36–38"), source_date: "November 29" },
    Week { code: "NL114", title: "Esther", season: "Second Sunday of Advent", primary: &["Esth 4:1–17"], alt_primary: None, accompanying: Some("Matt 5:13–16"), source_date: "December 6" },
    Week { code: "NL115", title: "Light to the Nations", season: "Third Sunday of Advent", primary: &["Isa 42:1–9"], alt_primary: None, accompanying: Some("Matt 12:15–21"), source_date: "December 13" },
    Week { code: "NL116", title: "Jesus as Immanuel", season: "Fourth Sunday of Advent", primary: &["Matt 1:18–25"], alt_primary: None, accompanying: Some("Ps 23:1–4, or 23:4"), source_date: "December 20" },
    Week { code: "NL117", title: "Birth of Jesus", season: "Christmas Eve", primary: &["Luke 2:1–14 [15–20]"], alt_primary: None, accompanying: Some("Ps 96:10–13"), source_date: "December 24" },
    Week { code: "NL118", title: "Shepherds Visit", season: "Christmas Day", primary: &["Luke 2:8–20"], alt_primary: None, accompanying: Some("Ps 95:6–7"), source_date: "December 25" },
    Week { code: "NL119", title: "Genealogy of Jesus", season: "First Sunday of Christmas", primary: &["Matt 1:1–17"], alt_primary: None, accompanying: Some("Ps 132:11–12"), source_date: "December 27" },
    Week { code: "NL120", title: "Flight to Egypt", season: "Second Sunday of Christmas", primary: &["Matt 2:1–23"], alt_primary: None, accompanying: Some("Ps 96:10–13 [opt: Ps 11:1–3]"), source_date: "January 3" },
    Week { code: "NL121", title: "Jesus' Baptism: Matthew", season: "Baptism of Our Lord", primary: &["Matt 3:1–17"], alt_primary: None, accompanying: Some("Ps 2:7–8"), source_date: "January 10" },
    Week { code: "NL122", title: "Tempted in the Wilderness", season: "Second Sunday after Epiphany", primary: &["Matt 4:1–17"], alt_primary: None, accompanying: Some("Ps 91:9–12"), source_date: "January 17" },
    Week { code: "NL123", title: "Beatitudes", season: "Third Sunday after Epiphany", primary: &["Matt 5:1–20"], alt_primary: None, accompanying: Some("Ps 1:1–3"), source_date: "January 24" },
    Week { code: "NL124", title: "Treasure in Heaven", season: "Fourth Sunday after Epiphany", primary: &["Matt 6:7–21 [25–34]"], alt_primary: None, accompanying: Some("Ps 20:7"), source_date: "January 31" },
    Week { code: "NL129", title: "Transfiguration", season: "Transfiguration", primary: &["Matt 16:24–17:8"], alt_primary: None, accompanying: Some("Ps 41:7–10"), source_date: "February 7" },
    Week { code: "NL130", title: "Who Is the Greatest?", season: "Ash Wednesday", primary: &["Matt 18:1–9"], alt_primary: None, accompanying: Some("Ps 146:7c–10 or 51:1–3"), source_date: "February 10" },
    Week { code: "NL131", title: "Forgiveness", season: "First Sunday in Lent", primary: &["Matt 18:15–35"], alt_primary: None, accompanying: Some("Ps 32:1–2"), source_date: "February 14" },
    Week { code: "NL132", title: "Laborers in the Vineyard", season: "Second Sunday in Lent", primary: &["Matt 20:1–16"], alt_primary: None, accompanying: Some("Ps 16:5–8"), source_date: "February 21" },
    Week { code: "NL133", title: "Wedding Banquet", season: "Third Sunday in Lent", primary: &["Matt 22:1–14"], alt_primary: None, accompanying: Some("Ps 45:6–7"), source_date: "February 28" },
    Week { code: "NL134", title: "Bridesmaids (and Talents)", season: "Fourth Sunday in Lent", primary: &["Matt 25:1–13 [14–30]"], alt_primary: None, accompanying: Some("Ps 43:3–4"), source_date: "March 7" },
    Week { code: "NL135", title: "Last Judgment", season: "Fifth Sunday in Lent", primary: &["Matt 25:31–46"], alt_primary: None, accompanying: Some("Ps 98:7–9"), source_date: "March 14" },
    Week { code: "NL136", title: "Triumphal Entry", season: "Palm Sunday", primary: &["Matt 21:1–17"], alt_primary: None, accompanying: Some("Ps 118:25–29"), source_date: "March 21" },
    Week { code: "NL137", title: "Words of Institution", season: "Maundy Thursday", primary: &["Matt 26:17–30"], alt_primary: None, accompanying: Some("Ps 116:12–15"), source_date: "March 25" },
    Week { code: "NL138", title: "Crucifixion: Matthew", season: "Good Friday", primary: &["Matt 27:27–61"], alt_primary: None, accompanying: Some("Ps 22:1–2, [14–18]"), source_date: "March 26" },
    Week { code: "NL139", title: "Easter: Matthew", season: "Easter", primary: &["Matt 28:1–10"], alt_primary: None, accompanying: Some("Ps 118:19–24"), source_date: "March 28" },
    Week { code: "NL140", title: "Great Commission", season: "Second Sunday of Easter", primary: &["Matt 28:16–20"], alt_primary: None, accompanying: Some("Ps 40:9–10"), source_date: "April 4" },
    Week { code: "NL141", title: "Peter's Vision", season: "Third Sunday of Easter", primary: &["Acts 10:1–17, 34–48"], alt_primary: None, accompanying: Some("Matt 9:36–37"), source_date: "April 11" },
    Week { code: "NL142", title: "Paul's Mission", season: "Fourth Sunday of Easter", primary: &["Acts 13:1–3; 14:8–18"], alt_primary: None, accompanying: Some("Matt 10:40–42"), source_date: "April 18" },
    Week { code: "NL143", title: "Gospel as Salvation", season: "Fifth Sunday of Easter", primary: &["Rom 1:1–17"], alt_primary: None, accompanying: Some("Matt 9:10–13"), source_date: "April 25" },
    Week { code: "NL144", title: "God's Love Poured Out", season: "Sixth Sunday of Easter", primary: &["Rom [3:28–30] 5:1–11"], alt_primary: None, accompanying: Some("Matt 11:28–30"), source_date: "May 2" },
    Week { code: "NL145", title: "Hope of Resurrection", season: "Seventh Sunday of Easter", primary: &["Rom 6:1–14"], alt_primary: None, accompanying: Some("Matt 6:24"), source_date: "May 9" },
    Week { code: "NL146", title: "Nothing Can Separate Us", season: "Pentecost", primary: &["Acts 2:1–4", "Rom 8:14–39"], alt_primary: None, accompanying: Some("Matt 28:16–20"), source_date: "May 16" },
];

/// Year 2 (Mark) — "Narrative Lectionary 2023-24", 42 weeks, Sept 10, 2023 – May 19, 2024.
pub static YEAR2_MARK: &[Week] = &[
    Week { code: "NL201", title: "Garden of Eden", season: "Fifteenth Sunday after Pentecost", primary: &["Gen 2:4b–25"], alt_primary: None, accompanying: Some("Mark 1:16–20 or Mark 10:6–8"), source_date: "September 10" },
    Week { code: "NL202", title: "Isaac Born to Sarah", season: "Sixteenth Sunday after Pentecost", primary: &["Gen 18:1–15; 21:1–7"], alt_primary: None, accompanying: Some("Mark 10:27"), source_date: "September 17" },
    Week { code: "NL203", title: "Jacob Wrestles God", season: "Seventeenth Sunday after Pentecost", primary: &["Gen 32:[9–13] 22–30"], alt_primary: None, accompanying: Some("Mark 14:32–36"), source_date: "September 24" },
    Week { code: "NL204", title: "Moses and God's Name", season: "Eighteenth Sunday after Pentecost", primary: &["Exod 1:8–14 [1:15–2:10]; 3:1–15"], alt_primary: None, accompanying: Some("Mark 12:26–27a"), source_date: "October 1" },
    Week { code: "NL205", title: "Hear O Israel", season: "Nineteenth Sunday after Pentecost", primary: &["Deut 5:1–21; 6:4–9"], alt_primary: None, accompanying: Some("Mark 12:28–31"), source_date: "October 8" },
    Week { code: "NL206", title: "Ruth", season: "Twentieth Sunday after Pentecost", primary: &["Ruth 1:1–17 [4:13–17]"], alt_primary: None, accompanying: Some("Mark 3:33–35"), source_date: "October 15" },
    Week { code: "NL207", title: "David Anointed King", season: "Twenty-first Sunday after Pentecost", primary: &["2 Sam 5:1–5; 6:1–5", "Ps 150"], alt_primary: None, accompanying: Some("Mark 11:8–10"), source_date: "October 22" },
    Week { code: "NL208", title: "Kingdom Divided", season: "Twenty-second Sunday after Pentecost", primary: &["1 Kgs 12:1–17, 25–29"], alt_primary: None, accompanying: Some("Mark 10:42–45"), source_date: "October 29" },
    Week { code: "NL209", title: "Elijah at Mount Carmel", season: "Twenty-third Sunday after Pentecost", primary: &["1 Kgs 18:[17–19] 20–39"], alt_primary: None, accompanying: Some("Mark 9:2–4"), source_date: "November 5" },
    Week { code: "NL210", title: "Hosea", season: "Twenty-fourth Sunday after Pentecost", primary: &["Hos 11:1–9"], alt_primary: None, accompanying: Some("Mark 10:13–14"), source_date: "November 12" },
    Week { code: "NL211", title: "Isaiah's Vineyard Song", season: "Twenty-fifth Sunday after Pentecost", primary: &["Isa 5:1–7; 11:1–5"], alt_primary: None, accompanying: Some("Mark 12:1–3"), source_date: "November 19" },
    Week { code: "NL212", title: "Josiah's Reform", season: "Christ the King Sunday", primary: &["2 Kgs 22:1–10 [11–20]; 23:1–3"], alt_primary: None, accompanying: Some("Luke 24:30–32"), source_date: "November 26" },
    Week { code: "NL213", title: "Promise of the Messiah", season: "First Sunday of Advent", primary: &["Jer 33:[10–11], 14–18"], alt_primary: None, accompanying: Some("Mark 8:27–29"), source_date: "December 3" },
    Week { code: "NL214", title: "Isaiah of the Exile", season: "Second Sunday of Advent", primary: &["Isa 40:1–11"], alt_primary: None, accompanying: Some("Mark 1:1–4"), source_date: "December 10" },
    Week { code: "NL215", title: "Rebuilding the Temple", season: "Third Sunday of Advent", primary: &["Ezra 1:1–4; 3:1–4, 10–13"], alt_primary: None, accompanying: Some("Luke 2:25–32"), source_date: "December 17" },
    Week { code: "NL216", title: "Zechariah's Song", season: "Fourth Sunday of Advent", primary: &["Luke 1:5–13 [14–25] 57–80"], alt_primary: None, accompanying: Some("Ps 113"), source_date: "December 24" },
    Week { code: "NL217", title: "Birth of Jesus", season: "Christmas Eve", primary: &["Luke 2:1–14 [15–20]"], alt_primary: None, accompanying: Some("Ps 146:5–10 or Luke 1:46–55"), source_date: "December 24" },
    Week { code: "NL218", title: "Shepherds Visit", season: "Christmas Day", primary: &["Luke 2:8–20"], alt_primary: Some("Matt 2:1–12 (The Magi)"), accompanying: Some("Ps 108:1–4"), source_date: "December 25" },
    Week { code: "NL219", title: "Beginning of Good News", season: "First Sunday of Christmas", primary: &["Mark 1:1–20"], alt_primary: None, accompanying: Some("Ps 91:9–12"), source_date: "December 31" },
    Week { code: "NL221", title: "Jesus Heals and Teaches", season: "Baptism of the Lord", primary: &["Mark 2:1–22"], alt_primary: None, accompanying: Some("Ps 103:6–14"), source_date: "January 7" },
    Week { code: "NL222", title: "Parables in Mark", season: "Second Sunday after Epiphany", primary: &["Mark 4:1–34"], alt_primary: None, accompanying: Some("Ps 126"), source_date: "January 14" },
    Week { code: "NL223", title: "Jesus and the Gerasene Demoniac", season: "Third Sunday after Epiphany", primary: &["Mark 5:1–20"], alt_primary: None, accompanying: Some("Ps 89:1–4"), source_date: "January 21" },
    Week { code: "NL224", title: "Jairus' Daughter Healed", season: "Fourth Sunday after Epiphany", primary: &["Mark 5:21–43"], alt_primary: None, accompanying: Some("Ps 131"), source_date: "January 28" },
    Week { code: "NL225", title: "Death of John the Baptist", season: "Fifth Sunday after Epiphany", primary: &["Mark 6:1–29"], alt_primary: None, accompanying: Some("Ps 122"), source_date: "February 4" },
    Week { code: "NL229", title: "Transfiguration", season: "Transfiguration Sunday", primary: &["Mark 8:27–9:8"], alt_primary: None, accompanying: Some("Ps 27:1–4"), source_date: "February 11" },
    Week { code: "NL230", title: "Passion Prediction", season: "Ash Wednesday", primary: &["Mark 9:30–37"], alt_primary: None, accompanying: Some("Ps 32:1–5"), source_date: "February 14" },
    Week { code: "NL231", title: "First Last and Last First", season: "First Sunday in Lent", primary: &["Mark 10:17–31"], alt_primary: None, accompanying: Some("Ps 19:7–10"), source_date: "February 18" },
    Week { code: "NL232", title: "Bartimaeus Healed", season: "Second Sunday in Lent", primary: &["Mark 10:32–52"], alt_primary: None, accompanying: Some("Ps 34:11–14"), source_date: "February 25" },
    Week { code: "NL233", title: "Parable of the Tenants [Taxes to Caesar]", season: "Third Sunday in Lent", primary: &["Mark 12:1–12 [13–17]"], alt_primary: None, accompanying: Some("Ps 80:8–13"), source_date: "March 3" },
    Week { code: "NL234", title: "Great Commandment", season: "Fourth Sunday in Lent", primary: &["Mark 12:28–44"], alt_primary: None, accompanying: Some("Ps 89:1–4"), source_date: "March 10" },
    Week { code: "NL235", title: "End of the Age", season: "Fifth Sunday in Lent", primary: &["Mark 13:1–8, 24–37"], alt_primary: None, accompanying: Some("Ps 102:12–17"), source_date: "March 17" },
    Week { code: "NL236", title: "Triumphal Entry (or Anointing at Bethany)", season: "Palm/Passion Sunday", primary: &["Mark 11:1–11", "Mark 14:3–9"], alt_primary: None, accompanying: Some("Ps 118:25–29"), source_date: "March 24" },
    Week { code: "NL237", title: "Lord's Supper, Prayer in Gethsemane", season: "Maundy Thursday", primary: &["Mark 14:22–42"], alt_primary: None, accompanying: Some("Ps 116:12–19"), source_date: "March 28" },
    Week { code: "NL238", title: "Crucifixion", season: "Good Friday", primary: &["Mark 15:16–39"], alt_primary: None, accompanying: Some("Ps 22:1–2, 14–21"), source_date: "March 29" },
    Week { code: "NL239", title: "Resurrection", season: "Easter Sunday", primary: &["Mark 16:1–8"], alt_primary: None, accompanying: Some("Ps 118:21–27"), source_date: "March 31" },
    Week { code: "NL240", title: "You Shall Be My Witnesses", season: "Second Sunday of Easter", primary: &["Acts 1:1–14"], alt_primary: None, accompanying: Some("Mark 6:7–13"), source_date: "April 7" },
    Week { code: "NL241", title: "Peter Heals in Jerusalem", season: "Third Sunday of Easter", primary: &["Acts 3:1–10"], alt_primary: None, accompanying: Some("Mark 6:53–56"), source_date: "April 14" },
    Week { code: "NL242", title: "Church at Thessalonica", season: "Fourth Sunday of Easter", primary: &["Acts 17:1–9", "1 Thess 1:1–10"], alt_primary: None, accompanying: Some("Mark 13:9–11"), source_date: "April 21" },
    Week { code: "NL243", title: "Church at Corinth", season: "Fifth Sunday of Easter", primary: &["Acts 18:1–4", "1 Cor 1:10–18"], alt_primary: None, accompanying: Some("Mark 9:34–35"), source_date: "April 28" },
    Week { code: "NL244", title: "Faith, Hope, and Love", season: "Sixth Sunday of Easter", primary: &["1 Cor 13:1–13"], alt_primary: None, accompanying: Some("Mark 12:28–31"), source_date: "May 5" },
    Week { code: "NL245", title: "Death Swallowed in Life", season: "Seventh Sunday of Easter", primary: &["1 Cor 15:1–26, 51–57"], alt_primary: None, accompanying: Some("Mark 12:26–27a"), source_date: "May 12" },
    Week { code: "NL246", title: "Gifts of the Spirit", season: "Day of Pentecost", primary: &["Acts 2:1–4", "1 Cor 12:1–13"], alt_primary: None, accompanying: Some("Mark 1:4–8"), source_date: "May 19" },
];

/// Year 3 (Luke) — "Narrative Lectionary 2024-25", 45 weeks, Sept 8, 2024 – June 8, 2025.
pub static YEAR3_LUKE: &[Week] = &[
    Week { code: "NL301", title: "Creation and Fall", season: "Sixteenth Sunday after Pentecost", primary: &["Gen 2:4b–7, 15–17; 3:1–8"], alt_primary: None, accompanying: Some("Luke 11:4"), source_date: "September 8" },
    Week { code: "NL302", title: "God's Promise to Abraham", season: "Seventeenth Sunday after Pentecost", primary: &["Gen 15:1–6"], alt_primary: None, accompanying: Some("Luke 3:8"), source_date: "September 15" },
    Week { code: "NL303", title: "God Works through Joseph", season: "Eighteenth Sunday after Pentecost", primary: &["Gen 37:3–8, 17b–22, 26–34; 50:15–21"], alt_primary: None, accompanying: Some("Luke 6:35"), source_date: "September 22" },
    Week { code: "NL304", title: "The Promise of Passover", season: "Nineteenth Sunday after Pentecost", primary: &["Exod 12:1–13; 13:1–8"], alt_primary: None, accompanying: Some("Luke 22:14–20"), source_date: "September 29" },
    Week { code: "NL305", title: "Golden Calf", season: "Twentieth Sunday after Pentecost", primary: &["Exod 32:1–14"], alt_primary: None, accompanying: Some("Luke 23:34"), source_date: "October 6" },
    Week { code: "NL306", title: "God Answers Hannah", season: "Twenty-first Sunday after Pentecost", primary: &["1 Sam 1:9–11, 19–20; 2:1–10"], alt_primary: None, accompanying: Some("Luke 1:46–55"), source_date: "October 13" },
    Week { code: "NL307", title: "God's Promise to David", season: "Twenty-second Sunday after Pentecost", primary: &["2 Sam 7:1–17"], alt_primary: None, accompanying: Some("Luke 1:30–33"), source_date: "October 20" },
    Week { code: "NL308", title: "Solomon Dedicates the Temple", season: "Twenty-third Sunday after Pentecost (or Reformation)", primary: &["1 Kgs 5:1–5; 8:27–30, 41–43"], alt_primary: None, accompanying: Some("Luke 19:45–46"), source_date: "October 27" },
    Week { code: "NL309", title: "God's Care for the Widow", season: "Twenty-fourth Sunday after Pentecost (or All Saints)", primary: &["1 Kgs 17:1–16 [17–24]"], alt_primary: None, accompanying: Some("Luke 4:24–26"), source_date: "November 3" },
    Week { code: "NL310", title: "Jonah and God's Mercy", season: "Twenty-fifth Sunday after Pentecost", primary: &["Jonah 1:1–17; 3:1–10 [4:1–11]"], alt_primary: None, accompanying: Some("Luke 18:13"), source_date: "November 10" },
    Week { code: "NL311", title: "God Calls Isaiah", season: "Twenty-sixth Sunday after Pentecost", primary: &["Isa 6:1–8"], alt_primary: None, accompanying: Some("Luke 5:8–10"), source_date: "November 17" },
    Week { code: "NL312", title: "God Promises a New Covenant", season: "Christ the King / Reign of Christ Sunday", primary: &["Jer 36:1–8, 21–23, 27–28; 31:31–34"], alt_primary: None, accompanying: Some("Luke 22:19–20"), source_date: "November 24" },
    Week { code: "NL313", title: "Daniel's Hope in God", season: "First Sunday of Advent", primary: &["Dan 6:6–27"], alt_primary: None, accompanying: Some("Luke 23:1–5"), source_date: "December 1" },
    Week { code: "NL314", title: "Joel: God's Promised Spirit", season: "Second Sunday of Advent", primary: &["Joel 2:12–13, 28–29"], alt_primary: None, accompanying: Some("Luke 11:13"), source_date: "December 8" },
    Week { code: "NL315", title: "Spirit of the Lord upon Me", season: "Third Sunday of Advent", primary: &["Isa 61:1–11"], alt_primary: None, accompanying: Some("Luke 4:16–21"), source_date: "December 15" },
    Week { code: "NL316", title: "Jesus' Birth Announced", season: "Fourth Sunday of Advent", primary: &["Luke 1:26–45 [46–56]"], alt_primary: None, accompanying: Some("Ps 113"), source_date: "December 22" },
    Week { code: "NL317", title: "Birth of Jesus", season: "Christmas Eve", primary: &["Luke 2:1–14 [15–20]"], alt_primary: None, accompanying: Some("Ps 96"), source_date: "December 24" },
    Week { code: "NL318", title: "Shepherds Visit", season: "Christmas Day", primary: &["Luke 2:8–20"], alt_primary: None, accompanying: Some("Ps 123:1–2"), source_date: "December 25" },
    Week { code: "NL319", title: "Simeon and Anna", season: "First Sunday of Christmas", primary: &["Luke 2:21–38"], alt_primary: None, accompanying: Some("Ps 131"), source_date: "December 29" },
    Week { code: "NL320", title: "Boy in the Temple", season: "Second Sunday of Christmas", primary: &["Luke 2:41–52"], alt_primary: None, accompanying: Some("Ps 2:7–8"), source_date: "January 5" },
    Week { code: "NL321", title: "Jesus' Baptism", season: "Baptism of Our Lord", primary: &["Luke 3:1–22"], alt_primary: None, accompanying: Some("Ps 51:6–17"), source_date: "January 12" },
    Week { code: "NL322", title: "Sermon at Nazareth", season: "Second Sunday after Epiphany", primary: &["Luke 4:14–30"], alt_primary: None, accompanying: Some("Ps 146"), source_date: "January 19" },
    Week { code: "NL323", title: "Fish for People", season: "Third Sunday after Epiphany", primary: &["Luke 5:1–11"], alt_primary: None, accompanying: Some("Ps 90:14–17"), source_date: "January 26" },
    Week { code: "NL324", title: "Healing on the Sabbath", season: "Fourth Sunday after Epiphany", primary: &["Luke 6:1–16"], alt_primary: None, accompanying: Some("Ps 92"), source_date: "February 2" },
    Week { code: "NL325", title: "Raising the Widow's Son", season: "Fifth Sunday after Epiphany", primary: &["Luke 7:1–17"], alt_primary: None, accompanying: Some("Ps 119:105–107"), source_date: "February 9" },
    Week { code: "NL326", title: "More than a Prophet", season: "Sixth Sunday after Epiphany", primary: &["Luke 7:18–35"], alt_primary: None, accompanying: Some("Ps 146:5–10"), source_date: "February 16" },
    Week { code: "NL327", title: "Forgiven at Jesus' Feet", season: "Seventh Sunday after Epiphany", primary: &["Luke 7:36–50"], alt_primary: None, accompanying: Some("Ps 103:3–6"), source_date: "February 23" },
    Week { code: "NL329", title: "Transfiguration", season: "Transfiguration of Our Lord", primary: &["Luke 9:28–45"], alt_primary: None, accompanying: Some("Ps 36:5–10"), source_date: "March 2" },
    Week { code: "NL330", title: "Jesus Turns to Jerusalem", season: "Ash Wednesday", primary: &["Luke 9:51–62"], alt_primary: None, accompanying: Some("Ps 5:7–8"), source_date: "March 5" },
    Week { code: "NL331", title: "Good Samaritan", season: "First Sunday in Lent", primary: &["Luke 10:25–42"], alt_primary: None, accompanying: Some("Ps 15"), source_date: "March 9" },
    Week { code: "NL332", title: "Lament over Jerusalem", season: "Second Sunday in Lent", primary: &["Luke 13:1–9, 31–35"], alt_primary: None, accompanying: Some("Ps 122"), source_date: "March 16" },
    Week { code: "NL333", title: "Lost Sheep, Coin, Son", season: "Third Sunday in Lent", primary: &["Luke 15:1–32"], alt_primary: None, accompanying: Some("Ps 119:167–176"), source_date: "March 23" },
    Week { code: "NL334", title: "Rich Man and Lazarus", season: "Fourth Sunday in Lent", primary: &["Luke 16:19–31"], alt_primary: None, accompanying: Some("Ps 41:1–3"), source_date: "March 30" },
    Week { code: "NL335", title: "Zacchaeus", season: "Fifth Sunday in Lent", primary: &["Luke 18:31–19:10"], alt_primary: None, accompanying: Some("Ps 84:1–4, 10–12"), source_date: "April 6" },
    Week { code: "NL336", title: "Triumphal Entry", season: "Palm/Passion Sunday", primary: &["Luke 19:29–44"], alt_primary: None, accompanying: Some("Ps 118:19–23"), source_date: "April 13" },
    Week { code: "NL337", title: "Last Supper", season: "Maundy Thursday", primary: &["Luke 22:1–27"], alt_primary: None, accompanying: Some("Ps 34:8–10"), source_date: "April 17" },
    Week { code: "NL338", title: "Crucifixion", season: "Good Friday", primary: &["Luke 23:32–47"], alt_primary: None, accompanying: Some("Ps 31:5–13"), source_date: "April 18" },
    Week { code: "NL339", title: "Resurrection", season: "Resurrection of Our Lord", primary: &["Luke 24:1–12"], alt_primary: None, accompanying: Some("Ps 118:17, 21–24"), source_date: "April 20" },
    Week { code: "NL340", title: "Emmaus Road", season: "Second Sunday of Easter", primary: &["Luke 24:13–35"], alt_primary: None, accompanying: Some("Ps 30"), source_date: "April 27" },
    Week { code: "NL341", title: "Stephen's Witness", season: "Third Sunday of Easter", primary: &["Acts 6:1–7:2a, 44–60"], alt_primary: None, accompanying: Some("Luke 23:33–34a, 46"), source_date: "May 4" },
    Week { code: "NL342", title: "Ethiopian Eunuch Baptized", season: "Fourth Sunday of Easter", primary: &["Acts 8:26–39"], alt_primary: None, accompanying: Some("Luke 24:44–47"), source_date: "May 11" },
    Week { code: "NL343", title: "Council at Jerusalem", season: "Fifth Sunday of Easter", primary: &["Acts 15:1–18"], alt_primary: None, accompanying: Some("Luke 2:29–32"), source_date: "May 18" },
    Week { code: "NL344", title: "Living by Faith", season: "Sixth Sunday of Easter", primary: &["Gal 1:13–17; 2:11–21"], alt_primary: None, accompanying: Some("Luke 18:9–14"), source_date: "May 25" },
    Week { code: "NL345", title: "One in Christ", season: "Seventh Sunday of Easter", primary: &["Gal 3:1–9, 23–29"], alt_primary: None, accompanying: Some("Luke 1:68–79"), source_date: "June 1" },
    Week { code: "NL346", title: "Pentecost; Fruits of the Spirit", season: "Day of Pentecost", primary: &["Acts 2:1–4", "Gal 4:1–7 [5:16–26]"], alt_primary: None, accompanying: Some("Luke 11:11–13"), source_date: "June 8" },
];

/// Year 4 (John) — "Narrative Lectionary 2025-26", 43 weeks, Sept 7, 2025 – May 24, 2026.
pub static YEAR4_JOHN: &[Week] = &[
    Week { code: "NL401", title: "Creation by the Word", season: "Thirteenth Sunday after Pentecost", primary: &["Gen 1:1–2:4a"], alt_primary: None, accompanying: Some("John 1:1–5"), source_date: "September 7" },
    Week { code: "NL402", title: "Binding of Isaac", season: "Fourteenth Sunday after Pentecost", primary: &["Gen 21:1–3; 22:1–14"], alt_primary: None, accompanying: Some("John 1:29"), source_date: "September 14" },
    Week { code: "NL403", title: "Jacob's Dream", season: "Fifteenth Sunday after Pentecost", primary: &["Gen 27:1–4, 15–23; 28:10–17"], alt_primary: None, accompanying: Some("John 1:50–51"), source_date: "September 21" },
    Week { code: "NL404", title: "God's Name Is Revealed", season: "Sixteenth Sunday after Pentecost", primary: &["Exod 2:23–25; 3:1–15; 4:10–17"], alt_primary: None, accompanying: Some("John 8:58"), source_date: "September 28" },
    Week { code: "NL405", title: "God Provides Manna", season: "Seventeenth Sunday after Pentecost", primary: &["Exod 16:1–18"], alt_primary: None, accompanying: Some("John 6:51"), source_date: "October 5" },
    Week { code: "NL406", title: "God Calls Samuel", season: "Eighteenth Sunday after Pentecost", primary: &["1 Sam 3:1–21"], alt_primary: None, accompanying: Some("John 20:21–23"), source_date: "October 12" },
    Week { code: "NL407", title: "God Calls David", season: "Nineteenth Sunday after Pentecost", primary: &["1 Sam 16:1–13", "Ps 51:10–14"], alt_primary: None, accompanying: Some("John 7:24"), source_date: "October 19" },
    Week { code: "NL408", title: "Solomon's Temple", season: "Twentieth Sunday after Pentecost (or Reformation Sunday)", primary: &["1 Kgs 5:1–5; 8:1–13"], alt_primary: None, accompanying: Some("John 2:19–21"), source_date: "October 26" },
    Week { code: "NL409", title: "God Speaks to Elijah", season: "Twenty-first Sunday after Pentecost (or All Saints Sunday)", primary: &["1 Kgs 19:1–18"], alt_primary: None, accompanying: Some("John 12:27–28"), source_date: "November 2" },
    Week { code: "NL410", title: "Amos: Justice Rolls Down", season: "Twenty-second Sunday after Pentecost", primary: &["Amos 1:1–2; 5:14–15, 21–24"], alt_primary: None, accompanying: Some("John 7:37–38"), source_date: "November 9" },
    Week { code: "NL411", title: "Isaiah: A Child Is Born", season: "Twenty-third Sunday after Pentecost", primary: &["Isa 9:1–7"], alt_primary: None, accompanying: Some("John 8:12"), source_date: "November 16" },
    Week { code: "NL412", title: "Jeremiah's Letter to Exiles", season: "Christ the King / Reign of Christ Sunday", primary: &["Jer 29:1, 4–14"], alt_primary: None, accompanying: Some("John 14:27"), source_date: "November 23" },
    Week { code: "NL413", title: "Daniel", season: "First Sunday of Advent", primary: &["Dan 3:1, [2–7] 8–30"], alt_primary: None, accompanying: Some("John 18:36–37"), source_date: "November 30" },
    Week { code: "NL414", title: "Ezekiel: Valley of Dry Bones", season: "Second Sunday of Advent", primary: &["Ezek 37:1–14"], alt_primary: None, accompanying: Some("John 11:25–26"), source_date: "December 7" },
    Week { code: "NL415", title: "Word Accomplishes God's Purpose", season: "Third Sunday of Advent", primary: &["Isa 55:1–13"], alt_primary: None, accompanying: Some("John 4:13–14"), source_date: "December 14" },
    Week { code: "NL416", title: "Word Made Flesh", season: "Fourth Sunday of Advent", primary: &["John 1:1–18"], alt_primary: None, accompanying: Some("Ps 130:5–8"), source_date: "December 21" },
    Week { code: "NL417", title: "Birth of Jesus", season: "Christmas Eve", primary: &["Luke 2:1–14 [15–20]"], alt_primary: None, accompanying: Some("Ps 96:7–10"), source_date: "December 24" },
    Week { code: "NL418", title: "Shepherds Visit", season: "Christmas Day", primary: &["Luke 2:8–20"], alt_primary: None, accompanying: Some("Ps 123:1–2 or 123:2"), source_date: "December 25" },
    Week { code: "NL419", title: "A Voice in the Wilderness", season: "First Sunday of Christmas", primary: &["John 1:19–34"], alt_primary: None, accompanying: Some("Ps 32:1–2"), source_date: "December 28" },
    Week { code: "NL420", title: "Jesus Says Come and See", season: "Second Sunday of Christmas", primary: &["John 1:35–51"], alt_primary: None, accompanying: Some("Ps 66:1–5"), source_date: "January 4" },
    Week { code: "NL421", title: "Wedding at Cana", season: "Baptism of Our Lord", primary: &["John 2:1–11"], alt_primary: None, accompanying: Some("Ps 104:14–16"), source_date: "January 11" },
    Week { code: "NL422", title: "Jesus Cleanses the Temple", season: "Second Sunday after Epiphany", primary: &["John 2:13–25"], alt_primary: None, accompanying: Some("Ps 127:1–2"), source_date: "January 18" },
    Week { code: "NL423", title: "Nicodemus", season: "Third Sunday after Epiphany", primary: &["John 3:1–21"], alt_primary: None, accompanying: Some("Ps 139:13–18"), source_date: "January 25" },
    Week { code: "NL424", title: "The Woman at the Well", season: "Fourth Sunday after Epiphany", primary: &["John 4:1–42"], alt_primary: None, accompanying: Some("Ps 42:1–3"), source_date: "February 1" },
    Week { code: "NL425", title: "Healing Stories", season: "Fifth Sunday after Epiphany", primary: &["John 4:46–54 [5:1–18]"], alt_primary: None, accompanying: Some("Ps 40:1–5"), source_date: "February 8" },
    Week { code: "NL429", title: "The Man Born Blind", season: "Transfiguration of Our Lord", primary: &["John 9:1–41"], alt_primary: None, accompanying: Some("Ps 27:1–4"), source_date: "February 15" },
    Week { code: "NL430", title: "The Good Shepherd", season: "Ash Wednesday", primary: &["John 10:1–18"], alt_primary: None, accompanying: Some("Ps 23"), source_date: "February 18" },
    Week { code: "NL431", title: "Jesus Raises Lazarus", season: "First Sunday in Lent", primary: &["John 11:1–44"], alt_primary: None, accompanying: Some("Ps 104:27–30"), source_date: "February 22" },
    Week { code: "NL432", title: "Jesus Washes Feet", season: "Second Sunday in Lent", primary: &["John 13:1–17"], alt_primary: None, accompanying: Some("Ps 51:7–12"), source_date: "March 1" },
    Week { code: "NL433", title: "Peter's Denial", season: "Third Sunday in Lent", primary: &["John 18:12–27"], alt_primary: None, accompanying: Some("Ps 17:1–7"), source_date: "March 8" },
    Week { code: "NL434", title: "Jesus and Pilate", season: "Fourth Sunday in Lent", primary: &["John 18:28–40"], alt_primary: None, accompanying: Some("Ps 145:10–13"), source_date: "March 15" },
    Week { code: "NL435", title: "Jesus Condemned", season: "Fifth Sunday in Lent", primary: &["John 19:1–16a"], alt_primary: None, accompanying: Some("Ps 146"), source_date: "March 22" },
    Week { code: "NL436", title: "The Crucified Messiah", season: "Palm/Passion Sunday", primary: &["John 19:16b–22"], alt_primary: Some("John 12:12–27 (Triumphal Entry)"), accompanying: Some("Ps 24"), source_date: "March 29" },
    Week { code: "NL437", title: "Jesus' Last Words", season: "Maundy Thursday", primary: &["John 19:23–30"], alt_primary: None, accompanying: Some("Ps 26:3"), source_date: "April 2" },
    Week { code: "NL438", title: "Jesus the Passover Lamb", season: "Good Friday", primary: &["John 19:31–42"], alt_primary: None, accompanying: Some("Ps 31:9–18"), source_date: "April 3" },
    Week { code: "NL439", title: "Resurrection", season: "Resurrection of Our Lord", primary: &["John 20:1–18"], alt_primary: None, accompanying: Some("Ps 118:21–29"), source_date: "April 5" },
    Week { code: "NL440", title: "Thomas", season: "Second Sunday of Easter", primary: &["John 20:19–31"], alt_primary: None, accompanying: Some("Ps 145:13–21"), source_date: "April 12" },
    Week { code: "NL441", title: "Paul's Conversion", season: "Third Sunday of Easter", primary: &["Acts 9:1–19a"], alt_primary: None, accompanying: Some("Matt 6:24"), source_date: "April 19" },
    Week { code: "NL442", title: "Paul and Silas", season: "Fourth Sunday of Easter", primary: &["Acts 16:16–34"], alt_primary: None, accompanying: Some("Luke 6:18–19, 22–23"), source_date: "April 26" },
    Week { code: "NL443", title: "Paul's Sermon at Athens", season: "Fifth Sunday of Easter", primary: &["Acts 17:16–31"], alt_primary: None, accompanying: Some("John 1:16–18"), source_date: "May 3" },
    Week { code: "NL444", title: "Partnership in the Gospel", season: "Sixth Sunday of Easter", primary: &["Phil 1:1–18a"], alt_primary: None, accompanying: Some("Luke 9:46–48"), source_date: "May 10" },
    Week { code: "NL445", title: "The Christ Hymn", season: "Seventh Sunday of Easter", primary: &["Phil 2:1–13"], alt_primary: None, accompanying: Some("Luke 6:43–45"), source_date: "May 17" },
    Week { code: "NL446", title: "Pentecost; Rejoice in the Lord", season: "Day of Pentecost", primary: &["Acts 2:1–21", "Phil 4:4–7"], alt_primary: None, accompanying: Some("John 14:16–17"), source_date: "May 24" },
];

/// One non-year-keyed transitional week, outside the 4-year cycle (see module doc).
pub struct SummerWeek {
    pub date: &'static str,
    pub series: &'static str,
    pub title: &'static str,
    pub primary: &'static str,
    pub accompanying: &'static str,
}

/// Summer 2026 readings (May 31 – Sept 6, 2026), between Year 4 (John)'s Pentecost and
/// Year 1 (Matthew)'s first Sunday. Three freestanding series, not part of the 4-year
/// cycle — see module doc. Source: "Narrative Lectionary Summer 2026 Readings", revised
/// 1/8/2026, https://www.workingpreacher.org/wp-content/uploads/2026/01/NL-summer-readings-2026_rev.pdf
pub static SUMMER_2026: &[SummerWeek] = &[
    SummerWeek { date: "5/31/2026", series: "Ruth & Esther", title: "Loss and Loyalty", primary: "Ruth 1:1–22", accompanying: "Matt 5:3–9 (The Beatitudes)" },
    SummerWeek { date: "6/7/2026", series: "Ruth & Esther", title: "Gleaning and Hope", primary: "Ruth 2:1–23", accompanying: "Luke 6:36–38 (Give and It Will Be Given)" },
    SummerWeek { date: "6/14/2026", series: "Ruth & Esther", title: "Daring to Act", primary: "Ruth 3:1–18", accompanying: "Matt 7:7–8 (Ask and You Will Receive)" },
    SummerWeek { date: "6/21/2026", series: "Ruth & Esther", title: "New Life", primary: "Ruth 4:1–22", accompanying: "Luke 1:46–55 (Mary Praises God)" },
    SummerWeek { date: "6/28/2026", series: "Ruth & Esther", title: "For a Moment Like This", primary: "Esth 4:1–17", accompanying: "Luke 9:23–24 (Call to Discipleship)" },
    SummerWeek { date: "7/5/2026", series: "Ruth & Esther", title: "From Sadness to Joy", primary: "Esth 7:1–10; 9:1–2, 20–22, 29–32", accompanying: "Luke 1:68–72 (God Saves Israel)" },
    SummerWeek { date: "7/12/2026", series: "1 & 2 Timothy", title: "Have Confidence in the Gospel", primary: "1 Tim 1:12–17", accompanying: "Luke 15:4–7" },
    SummerWeek { date: "7/19/2026", series: "1 & 2 Timothy", title: "Take Hold of Eternal Life", primary: "1 Tim 6:6–19", accompanying: "Luke 16:27–30" },
    SummerWeek { date: "7/26/2026", series: "1 & 2 Timothy", title: "Rekindle the Gift of God", primary: "2 Tim 1:1–14", accompanying: "Luke 17:5–6" },
    SummerWeek { date: "8/2/2026", series: "1 & 2 Timothy", title: "Remember Jesus Christ, Raised from the Dead", primary: "2 Tim 2:8–15", accompanying: "Luke 17:16–19" },
    SummerWeek { date: "8/9/2026", series: "1 & 2 Timothy", title: "The Lord Stood by Me", primary: "2 Tim 3:14–4:8, 16–18", accompanying: "Luke 18:6–8" },
    SummerWeek { date: "8/16/2026", series: "Ten Commandments", title: "Nineteen Comes before Twenty", primary: "Exod 19:1–6; 20:1–2", accompanying: "Matt 22:34–40 (The Great Commandment)" },
    SummerWeek { date: "8/23/2026", series: "Ten Commandments", title: "Tuned into God", primary: "Exod 20:3–11", accompanying: "Matt 22:34–40 (The Great Commandment)" },
    SummerWeek { date: "8/30/2026", series: "Ten Commandments", title: "Tuned toward the Neighbor", primary: "Exod 20:12–16", accompanying: "Matt 22:34–40 (The Great Commandment)" },
    SummerWeek { date: "9/6/2026", series: "Ten Commandments", title: "Do Not Covet", primary: "Exod 20:17", accompanying: "Matt 22:34–40 (The Great Commandment)" },
];
