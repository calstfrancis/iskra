//! Revised Common Lectionary readings + liturgical calendar logic.
//!
//! Mechanical port of Rubric's `rcl_data.py` (stdlib-only, no shared crate
//! possible across the Python/Rust boundary — see Plans/plan.md §1 recon
//! answer 2). OT/Psalm in Ordinary Time follows the Semicontinuous track
//! (UCC default), matching Rubric exactly so a date's readings/season/colour
//! agree between the two apps.

use chrono::{Datelike, NaiveDate};

pub type Readings = (&'static str, &'static str, &'static str, &'static str);

pub static READINGS: &[(&str, &str, Readings)] = &[
    ("A", "Advent1", ("Isa 2:1–5", "Ps 122", "Rom 13:11–14", "Matt 24:36–44")),
    ("A", "Advent2", ("Isa 11:1–10", "Ps 72:1–7, 18–19", "Rom 15:4–13", "Matt 3:1–12")),
    ("A", "Advent3", ("Isa 35:1–10", "Ps 146:5–10", "Jas 5:7–10", "Matt 11:2–11")),
    ("A", "Advent4", ("Isa 7:10–16", "Ps 80:1–7, 17–19", "Rom 1:1–7", "Matt 1:18–25")),
    ("B", "Advent1", ("Isa 64:1–9", "Ps 80:1–7, 17–19", "1 Cor 1:3–9", "Mark 13:24–37")),
    ("B", "Advent2", ("Isa 40:1–11", "Ps 85:1–2, 8–13", "2 Pet 3:8–15a", "Mark 1:1–8")),
    ("B", "Advent3", ("Isa 61:1–4, 8–11", "Ps 126", "1 Thess 5:16–24", "John 1:6–8, 19–28")),
    ("B", "Advent4", ("2 Sam 7:1–11, 16", "Luke 1:47–55", "Rom 16:25–27", "Luke 1:26–38")),
    ("C", "Advent1", ("Jer 33:14–16", "Ps 25:1–10", "1 Thess 3:9–13", "Luke 21:25–36")),
    ("C", "Advent2", ("Mal 3:1–4", "Luke 1:68–79", "Phil 1:3–11", "Luke 3:1–6")),
    ("C", "Advent3", ("Zeph 3:14–20", "Isa 12:2–6", "Phil 4:4–7", "Luke 3:7–18")),
    ("C", "Advent4", ("Mic 5:2–5a", "Luke 1:47–55", "Heb 10:5–10", "Luke 1:39–45")),
    ("ALL", "Christmas1A", ("Isa 63:7–9", "Ps 148", "Heb 2:10–18", "Matt 2:13–23")),
    ("ALL", "Christmas1B", ("Isa 61:10–62:3", "Ps 148", "Gal 4:4–7", "Luke 2:22–40")),
    ("ALL", "Christmas1C", ("1 Sam 2:18–20, 26", "Ps 148", "Col 3:12–17", "Luke 2:41–52")),
    ("ALL", "Christmas2", ("Jer 31:7–14", "Ps 147:12–20", "Eph 1:3–14", "John 1:[1–9]10–18")),
    ("A", "Epiphany1", ("Isa 42:1–9", "Ps 29", "Acts 10:34–43", "Matt 3:13–17")),
    ("B", "Epiphany1", ("Gen 1:1–5", "Ps 29", "Acts 19:1–7", "Mark 1:4–11")),
    ("C", "Epiphany1", ("Isa 43:1–7", "Ps 29", "Acts 8:14–17", "Luke 3:15–17, 21–22")),
    ("A", "Epiphany2", ("Isa 49:1–7", "Ps 40:1–11", "1 Cor 1:1–9", "John 1:29–42")),
    ("B", "Epiphany2", ("1 Sam 3:1–10", "Ps 139:1–6, 13–18", "1 Cor 6:12–20", "John 1:43–51")),
    ("C", "Epiphany2", ("Isa 62:1–5", "Ps 36:5–10", "1 Cor 12:1–11", "John 2:1–11")),
    ("A", "Epiphany3", ("Isa 9:1–4", "Ps 27:1, 4–9", "1 Cor 1:10–18", "Matt 4:12–23")),
    ("B", "Epiphany3", ("Jon 3:1–5, 10", "Ps 62:5–12", "1 Cor 7:29–31", "Mark 1:14–20")),
    ("C", "Epiphany3", ("Neh 8:1–3, 5–6, 8–10", "Ps 19", "1 Cor 12:12–31a", "Luke 4:14–21")),
    ("A", "Epiphany4", ("Mic 6:1–8", "Ps 15", "1 Cor 1:18–31", "Matt 5:1–12")),
    ("B", "Epiphany4", ("Deut 18:15–20", "Ps 111", "1 Cor 8:1–13", "Mark 1:21–28")),
    ("C", "Epiphany4", ("Jer 1:4–10", "Ps 71:1–6", "1 Cor 13:1–13", "Luke 4:21–30")),
    ("A", "Epiphany5", ("Isa 58:1–9a", "Ps 112:1–9", "1 Cor 2:1–12", "Matt 5:13–20")),
    ("B", "Epiphany5", ("Isa 40:21–31", "Ps 147:1–11, 20c", "1 Cor 9:16–23", "Mark 1:29–39")),
    ("C", "Epiphany5", ("Isa 6:1–8", "Ps 138", "1 Cor 15:1–11", "Luke 5:1–11")),
    ("A", "Epiphany6", ("Deut 30:15–20", "Ps 119:1–8", "1 Cor 3:1–9", "Matt 5:21–37")),
    ("B", "Epiphany6", ("2 Kgs 5:1–14", "Ps 30", "1 Cor 9:24–27", "Mark 1:40–45")),
    ("C", "Epiphany6", ("Jer 17:5–10", "Ps 1", "1 Cor 15:12–20", "Luke 6:17–26")),
    ("A", "Epiphany7", ("Lev 19:1–2, 9–18", "Ps 119:33–40", "1 Cor 3:10–11, 16–23", "Matt 5:38–48")),
    ("B", "Epiphany7", ("Isa 43:18–25", "Ps 41", "2 Cor 1:18–22", "Mark 2:1–12")),
    ("C", "Epiphany7", ("Gen 45:3–11, 15", "Ps 37:1–11, 39–40", "1 Cor 15:35–38, 42–50", "Luke 6:27–38")),
    ("A", "Epiphany8", ("Isa 49:8–16a", "Ps 131", "1 Cor 4:1–5", "Matt 6:24–34")),
    ("B", "Epiphany8", ("Hos 2:14–20", "Ps 103:1–13, 22", "2 Cor 3:1–6", "Mark 2:13–22")),
    ("C", "Epiphany8", ("Isa 55:10–13", "Ps 92:1–4, 12–15", "1 Cor 15:51–58", "Luke 6:39–49")),
    ("A", "Transfiguration", ("Exod 24:12–18", "Ps 2", "2 Pet 1:16–21", "Matt 17:1–9")),
    ("B", "Transfiguration", ("2 Kgs 2:1–12", "Ps 50:1–6", "2 Cor 4:3–6", "Mark 9:2–9")),
    ("C", "Transfiguration", ("Exod 34:29–35", "Ps 99", "2 Cor 3:12–4:2", "Luke 9:28–36")),
    ("ALL", "AshWednesday", ("Joel 2:1–2, 12–17", "Ps 51:1–17", "2 Cor 5:20b–6:10", "Matt 6:1–6, 16–21")),
    ("A", "Lent1", ("Gen 2:15–17; 3:1–7", "Ps 32", "Rom 5:12–19", "Matt 4:1–11")),
    ("B", "Lent1", ("Gen 9:8–17", "Ps 25:1–10", "1 Pet 3:18–22", "Mark 1:9–15")),
    ("C", "Lent1", ("Deut 26:1–11", "Ps 91:1–2, 9–16", "Rom 10:8b–13", "Luke 4:1–13")),
    ("A", "Lent2", ("Gen 12:1–4a", "Ps 121", "Rom 4:1–5, 13–17", "John 3:1–17")),
    ("B", "Lent2", ("Gen 17:1–7, 15–16", "Ps 22:23–31", "Rom 4:13–25", "Mark 8:31–38")),
    ("C", "Lent2", ("Gen 15:1–12, 17–18", "Ps 27", "Phil 3:17–4:1", "Luke 13:31–35")),
    ("A", "Lent3", ("Exod 17:1–7", "Ps 95", "Rom 5:1–11", "John 4:5–42")),
    ("B", "Lent3", ("Exod 20:1–17", "Ps 19", "1 Cor 1:18–25", "John 2:13–22")),
    ("C", "Lent3", ("Isa 55:1–9", "Ps 63:1–8", "1 Cor 10:1–13", "Luke 13:1–9")),
    ("A", "Lent4", ("1 Sam 16:1–13", "Ps 23", "Eph 5:8–14", "John 9:1–41")),
    ("B", "Lent4", ("Num 21:4–9", "Ps 107:1–3, 17–22", "Eph 2:1–10", "John 3:14–21")),
    ("C", "Lent4", ("Josh 5:9–12", "Ps 32", "2 Cor 5:16–21", "Luke 15:1–3, 11b–32")),
    ("A", "Lent5", ("Ezek 37:1–14", "Ps 130", "Rom 8:6–11", "John 11:1–45")),
    ("B", "Lent5", ("Jer 31:31–34", "Ps 51:1–12", "Heb 5:5–10", "John 12:20–33")),
    ("C", "Lent5", ("Isa 43:16–21", "Ps 126", "Phil 3:4b–14", "John 12:1–8")),
    ("A", "PalmSunday", ("Isa 50:4–9a", "Ps 31:9–16", "Phil 2:5–11", "Matt 26:14–27:66")),
    ("B", "PalmSunday", ("Isa 50:4–9a", "Ps 31:9–16", "Phil 2:5–11", "Mark 14:1–15:47")),
    ("C", "PalmSunday", ("Isa 50:4–9a", "Ps 31:9–16", "Phil 2:5–11", "Luke 22:14–23:56")),
    ("ALL", "HolyThursday", ("Exod 12:1–4, 11–14", "Ps 116:1–2, 12–19", "1 Cor 11:23–26", "John 13:1–17, 31b–35")),
    ("ALL", "GoodFriday", ("Isa 52:13–53:12", "Ps 22", "Heb 10:16–25", "John 18:1–19:42")),
    ("A", "Easter", ("Acts 10:34–43", "Ps 118:1–2, 14–24", "Col 3:1–4", "John 20:1–18")),
    ("B", "Easter", ("Acts 10:34–43", "Ps 118:1–2, 14–24", "1 Cor 15:1–11", "John 20:1–18")),
    ("C", "Easter", ("Acts 10:34–43", "Ps 118:1–2, 14–24", "1 Cor 15:19–26", "John 20:1–18")),
    ("A", "Easter2", ("Acts 2:14a, 22–32", "Ps 16", "1 Pet 1:3–9", "John 20:19–31")),
    ("B", "Easter2", ("Acts 4:32–35", "Ps 133", "1 John 1:1–2:2", "John 20:19–31")),
    ("C", "Easter2", ("Acts 5:27–32", "Ps 118:14–29", "Rev 1:4–8", "John 20:19–31")),
    ("A", "Easter3", ("Acts 2:14a, 36–41", "Ps 116:1–4, 12–19", "1 Pet 1:17–23", "Luke 24:13–35")),
    ("B", "Easter3", ("Acts 3:12–19", "Ps 4", "1 John 3:1–7", "Luke 24:36b–48")),
    ("C", "Easter3", ("Acts 9:1–6", "Ps 30", "Rev 5:11–14", "John 21:1–19")),
    ("A", "Easter4", ("Acts 2:42–47", "Ps 23", "1 Pet 2:19–25", "John 10:1–10")),
    ("B", "Easter4", ("Acts 4:5–12", "Ps 23", "1 John 3:16–24", "John 10:11–18")),
    ("C", "Easter4", ("Acts 9:36–43", "Ps 23", "Rev 7:9–17", "John 10:22–30")),
    ("A", "Easter5", ("Acts 7:55–60", "Ps 31:1–5, 15–16", "1 Pet 2:2–10", "John 14:1–14")),
    ("B", "Easter5", ("Acts 8:26–40", "Ps 22:25–31", "1 John 4:7–21", "John 15:1–8")),
    ("C", "Easter5", ("Acts 11:1–18", "Ps 148", "Rev 21:1–6", "John 13:31–35")),
    ("A", "Easter6", ("Acts 17:22–31", "Ps 66:8–20", "1 Pet 3:13–22", "John 14:15–21")),
    ("B", "Easter6", ("Acts 10:44–48", "Ps 98", "1 John 5:1–6", "John 15:9–17")),
    ("C", "Easter6", ("Acts 16:9–15", "Ps 67", "Rev 21:10, 22–22:5", "John 14:23–29")),
    ("ALL", "Ascension", ("Acts 1:1–11", "Ps 47", "Eph 1:15–23", "Luke 24:44–53")),
    ("A", "Easter7", ("Acts 1:6–14", "Ps 68:1–10, 32–35", "1 Pet 4:12–14; 5:6–11", "John 17:1–11")),
    ("B", "Easter7", ("Acts 1:15–17, 21–26", "Ps 1", "1 John 5:9–13", "John 17:6–19")),
    ("C", "Easter7", ("Acts 16:16–34", "Ps 97", "Rev 22:12–14, 16–17, 20–21", "John 17:20–26")),
    ("A", "Pentecost", ("Acts 2:1–21", "Ps 104:24–34, 35b", "1 Cor 12:3b–13", "John 20:19–23")),
    ("B", "Pentecost", ("Acts 2:1–21", "Ps 104:24–34, 35b", "Rom 8:22–27", "John 15:26–27; 16:4b–15")),
    ("C", "Pentecost", ("Acts 2:1–21", "Ps 104:24–34, 35b", "Rom 8:14–17", "John 14:8–17")),
    ("A", "Trinity", ("Gen 1:1–2:4a", "Ps 8", "2 Cor 13:11–13", "Matt 28:16–20")),
    ("B", "Trinity", ("Isa 6:1–8", "Ps 29", "Rom 8:12–17", "John 3:1–17")),
    ("C", "Trinity", ("Prov 8:1–4, 22–31", "Ps 8", "Rom 5:1–5", "John 16:12–15")),
    ("A", "Proper4", ("Gen 6:9–22; 7:24; 8:14–19", "Ps 46", "Rom 1:16–17; 3:22b–28", "Matt 7:21–29")),
    ("B", "Proper4", ("1 Sam 3:1–10", "Ps 139:1–6, 13–18", "2 Cor 4:5–12", "Mark 2:23–3:6")),
    ("C", "Proper4", ("1 Kgs 18:20–21, 30–39", "Ps 96", "Gal 1:1–12", "Luke 7:1–10")),
    ("A", "Proper5", ("Gen 12:1–9", "Ps 33:1–12", "Rom 4:13–25", "Matt 9:9–13, 18–26")),
    ("B", "Proper5", ("1 Sam 8:4–11, 16–20", "Ps 138", "2 Cor 4:13–5:1", "Mark 3:20–35")),
    ("C", "Proper5", ("1 Kgs 17:8–16", "Ps 146", "Gal 1:11–24", "Luke 7:11–17")),
    ("A", "Proper6", ("Gen 18:1–15", "Ps 116:1–2, 12–19", "Rom 5:1–8", "Matt 9:35–10:8")),
    ("B", "Proper6", ("1 Sam 15:34–16:13", "Ps 20", "2 Cor 5:6–10, 14–17", "Mark 4:26–34")),
    ("C", "Proper6", ("1 Kgs 21:1–10, 15–21a", "Ps 5:1–8", "Gal 2:15–21", "Luke 7:36–8:3")),
    ("A", "Proper7", ("Gen 21:8–21", "Ps 86:1–10, 16–17", "Rom 6:1b–11", "Matt 10:24–39")),
    ("B", "Proper7", ("1 Sam 17:32–49", "Ps 9:9–20", "2 Cor 6:1–13", "Mark 4:35–41")),
    ("C", "Proper7", ("1 Kgs 19:1–4, 8–15a", "Ps 42", "Gal 3:23–29", "Luke 8:26–39")),
    ("A", "Proper8", ("Gen 22:1–14", "Ps 13", "Rom 6:12–23", "Matt 10:40–42")),
    ("B", "Proper8", ("2 Sam 1:1, 17–27", "Ps 130", "2 Cor 8:7–15", "Mark 5:21–43")),
    ("C", "Proper8", ("2 Kgs 2:1–2, 6–14", "Ps 77:1–2, 11–20", "Gal 5:1, 13–25", "Luke 9:51–62")),
    ("A", "Proper9", ("Gen 24:34–38, 42–49, 58–67", "Ps 45:10–17", "Rom 7:15–25a", "Matt 11:16–19, 25–30")),
    ("B", "Proper9", ("2 Sam 5:1–5, 9–10", "Ps 48", "2 Cor 12:2–10", "Mark 6:1–13")),
    ("C", "Proper9", ("2 Kgs 5:1–14", "Ps 30", "Gal 6:1–16", "Luke 10:1–11, 16–20")),
    ("A", "Proper10", ("Gen 25:19–34", "Ps 119:105–112", "Rom 8:1–11", "Matt 13:1–9, 18–23")),
    ("B", "Proper10", ("2 Sam 6:1–5, 12b–19", "Ps 24", "Eph 1:3–14", "Mark 6:14–29")),
    ("C", "Proper10", ("Amos 7:7–17", "Ps 82", "Col 1:1–14", "Luke 10:25–37")),
    ("A", "Proper11", ("Gen 28:10–19a", "Ps 139:1–12, 23–24", "Rom 8:12–25", "Matt 13:24–30, 36–43")),
    ("B", "Proper11", ("2 Sam 7:1–14a", "Ps 89:20–37", "Eph 2:11–22", "Mark 6:30–34, 53–56")),
    ("C", "Proper11", ("Amos 8:1–12", "Ps 52", "Col 1:15–28", "Luke 10:38–42")),
    ("A", "Proper12", ("Gen 29:15–28", "Ps 105:1–11, 45b", "Rom 8:26–39", "Matt 13:31–33, 44–52")),
    ("B", "Proper12", ("2 Sam 11:1–15", "Ps 14", "Eph 3:14–21", "John 6:1–21")),
    ("C", "Proper12", ("Hos 1:2–10", "Ps 85", "Col 2:6–15", "Luke 11:1–13")),
    ("A", "Proper13", ("Gen 32:22–31", "Ps 17:1–7, 15", "Rom 9:1–5", "Matt 14:13–21")),
    ("B", "Proper13", ("2 Sam 11:26–12:13a", "Ps 51:1–12", "Eph 4:1–16", "John 6:24–35")),
    ("C", "Proper13", ("Hos 11:1–11", "Ps 107:1–9, 43", "Col 3:1–11", "Luke 12:13–21")),
    ("A", "Proper14", ("Gen 37:1–4, 12–28", "Ps 105:1–6, 16–22", "Rom 10:5–15", "Matt 14:22–33")),
    ("B", "Proper14", ("2 Sam 18:5–9, 15, 31–33", "Ps 130", "Eph 4:25–5:2", "John 6:35, 41–51")),
    ("C", "Proper14", ("Isa 1:1, 10–20", "Ps 50:1–8, 22–23", "Heb 11:1–3, 8–16", "Luke 12:32–40")),
    ("A", "Proper15", ("Gen 45:1–15", "Ps 133", "Rom 11:1–2a, 29–32", "Matt 15:21–28")),
    ("B", "Proper15", ("1 Kgs 2:10–12; 3:3–14", "Ps 111", "Eph 5:15–20", "John 6:51–58")),
    ("C", "Proper15", ("Isa 5:1–7", "Ps 80:1–2, 8–19", "Heb 11:29–12:2", "Luke 12:49–56")),
    ("A", "Proper16", ("Exod 1:8–2:10", "Ps 124", "Rom 12:1–8", "Matt 16:13–20")),
    ("B", "Proper16", ("1 Kgs 8:1, 6, 10–11, 22–30", "Ps 84", "Eph 6:10–20", "John 6:56–69")),
    ("C", "Proper16", ("Isa 58:9b–14", "Ps 103:1–8", "Heb 12:18–29", "Luke 13:10–17")),
    ("A", "Proper17", ("Exod 3:1–15", "Ps 105:1–6, 23–26", "Rom 12:9–21", "Matt 16:21–28")),
    ("B", "Proper17", ("Song 2:8–13", "Ps 45:1–2, 6–9", "Jas 1:17–27", "Mark 7:1–8, 14–15, 21–23")),
    ("C", "Proper17", ("Jer 2:4–13", "Ps 81:1, 10–16", "Heb 13:1–8, 15–16", "Luke 14:1, 7–14")),
    ("A", "Proper18", ("Exod 12:1–14", "Ps 149", "Rom 13:8–14", "Matt 18:15–20")),
    ("B", "Proper18", ("Prov 22:1–2, 8–9, 22–23", "Ps 125", "Jas 2:1–17", "Mark 7:24–37")),
    ("C", "Proper18", ("Jer 18:1–11", "Ps 139:1–6, 13–18", "Phlm 1–21", "Luke 14:25–33")),
    ("A", "Proper19", ("Exod 14:19–31", "Ps 114", "Rom 14:1–12", "Matt 18:21–35")),
    ("B", "Proper19", ("Prov 1:20–33", "Ps 19", "Jas 3:1–12", "Mark 8:27–38")),
    ("C", "Proper19", ("Jer 4:11–12, 22–28", "Ps 14", "1 Tim 1:12–17", "Luke 15:1–10")),
    ("A", "Proper20", ("Exod 16:2–15", "Ps 105:1–6, 37–45", "Phil 1:21–30", "Matt 20:1–16")),
    ("B", "Proper20", ("Prov 31:10–31", "Ps 1", "Jas 3:13–4:3, 7–8a", "Mark 9:30–37")),
    ("C", "Proper20", ("Jer 8:18–9:1", "Ps 79:1–9", "1 Tim 2:1–7", "Luke 16:1–13")),
    ("A", "Proper21", ("Exod 17:1–7", "Ps 78:1–4, 12–16", "Phil 2:1–13", "Matt 21:23–32")),
    ("B", "Proper21", ("Esth 7:1–6, 9–10; 9:20–22", "Ps 124", "Jas 5:13–20", "Mark 9:38–50")),
    ("C", "Proper21", ("Jer 32:1–3a, 6–15", "Ps 91:1–6, 14–16", "1 Tim 6:6–19", "Luke 16:19–31")),
    ("A", "Proper22", ("Exod 20:1–4, 7–9, 12–20", "Ps 19", "Phil 3:4b–14", "Matt 21:33–46")),
    ("B", "Proper22", ("Job 1:1; 2:1–10", "Ps 26", "Heb 1:1–4; 2:5–12", "Mark 10:2–16")),
    ("C", "Proper22", ("Lam 1:1–6", "Ps 137", "2 Tim 1:1–14", "Luke 17:5–10")),
    ("A", "Proper23", ("Exod 32:1–14", "Ps 106:1–6, 19–23", "Phil 4:1–9", "Matt 22:1–14")),
    ("B", "Proper23", ("Job 23:1–9, 16–17", "Ps 22:1–15", "Heb 4:12–16", "Mark 10:17–31")),
    ("C", "Proper23", ("Jer 29:1, 4–7", "Ps 66:1–12", "2 Tim 2:8–15", "Luke 17:11–19")),
    ("A", "Proper24", ("Exod 33:12–23", "Ps 99", "1 Thess 1:1–10", "Matt 22:15–22")),
    ("B", "Proper24", ("Job 38:1–7", "Ps 104:1–9, 24, 35c", "Heb 5:1–10", "Mark 10:35–45")),
    ("C", "Proper24", ("Jer 31:27–34", "Ps 119:97–104", "2 Tim 3:14–4:5", "Luke 18:1–8")),
    ("A", "Proper25", ("Deut 34:1–12", "Ps 90:1–6, 13–17", "1 Thess 2:1–8", "Matt 22:34–46")),
    ("B", "Proper25", ("Job 42:1–6, 10–17", "Ps 34:1–8", "Heb 7:23–28", "Mark 10:46–52")),
    ("C", "Proper25", ("Joel 2:23–32", "Ps 65", "2 Tim 4:6–8, 16–18", "Luke 18:9–14")),
    ("A", "Proper26", ("Josh 3:7–17", "Ps 107:1–7, 33–37", "1 Thess 2:9–13", "Matt 23:1–12")),
    ("B", "Proper26", ("Ruth 1:1–18", "Ps 146", "Heb 9:11–14", "Mark 12:28–34")),
    ("C", "Proper26", ("Hab 1:1–4; 2:1–4", "Ps 119:137–144", "2 Thess 1:1–4, 11–12", "Luke 19:1–10")),
    ("A", "Proper27", ("Josh 24:1–3a, 14–25", "Ps 78:1–7", "1 Thess 4:13–18", "Matt 25:1–13")),
    ("B", "Proper27", ("Ruth 3:1–5; 4:13–17", "Ps 127", "Heb 9:24–28", "Mark 12:38–44")),
    ("C", "Proper27", ("Hag 1:15b–2:9", "Ps 145:1–5, 17–21", "2 Thess 2:1–5, 13–17", "Luke 20:27–38")),
    ("A", "Proper28", ("Judg 4:1–7", "Ps 123", "1 Thess 5:1–11", "Matt 25:14–30")),
    ("B", "Proper28", ("1 Sam 1:4–20", "1 Sam 2:1–10", "Heb 10:11–14, 19–25", "Mark 13:1–8")),
    ("C", "Proper28", ("Isa 65:17–25", "Isa 12", "2 Thess 3:6–13", "Luke 21:5–19")),
    ("A", "Proper29", ("Ezek 34:11–16, 20–24", "Ps 95:1–7a", "Eph 1:15–23", "Matt 25:31–46")),
    ("B", "Proper29", ("2 Sam 23:1–7", "Ps 132:1–12", "Rev 1:4b–8", "John 18:33–37")),
    ("C", "Proper29", ("Jer 23:1–6", "Ps 46", "Col 1:11–20", "Luke 23:33–43")),
];
pub static COLOURS: &[(&str, (&str, &str))] = &[
    ("Advent", ("Purple", "#6B21A8")),
    ("Christmas", ("White / Gold", "#B45309")),
    ("Epiphany", ("Green", "#15803D")),
    ("Baptism", ("White / Gold", "#B45309")),
    ("Transfiguration", ("White / Gold", "#B45309")),
    ("Lent", ("Purple", "#6B21A8")),
    ("Palm Sunday", ("Scarlet", "#B91C1C")),
    ("Holy Thursday", ("White / Gold", "#B45309")),
    ("Good Friday", ("Black", "#111827")),
    ("Easter", ("White / Gold", "#B45309")),
    ("Pentecost", ("Red", "#B91C1C")),
    ("Trinity", ("White / Gold", "#B45309")),
    ("Ordinary", ("Green", "#15803D")),
    ("Christ the King", ("White / Gold", "#B45309")),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiturgicalInfo {
    pub season: String,
    pub week: String,
    pub year: String,
    pub colour: String,
    pub colour_hex: String,
    pub ot: String,
    pub psalm: String,
    pub epistle: String,
    pub gospel: String,
    pub found: bool,
}

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
pub fn lectionary_year(d: NaiveDate) -> &'static str {
    let adv = advent_sunday(d.year());
    let base = if d >= adv { d.year() + 1 } else { d.year() };
    const YEARS: [&str; 3] = ["A", "B", "C"];
    YEARS[(base - 2023).rem_euclid(3) as usize]
}

/// Python's `date.weekday()`: Monday = 0 .. Sunday = 6. Chrono's
/// `num_days_from_monday()` agrees, so this is just a naming/type bridge to
/// keep the calendar math below a direct line-by-line mirror of the source.
fn python_weekday(d: NaiveDate) -> i64 {
    d.weekday().num_days_from_monday() as i64
}

fn is_sunday(d: NaiveDate) -> bool {
    (python_weekday(d) + 1) % 7 == 0
}

/// Return the Sunday nearest to `d` (or `d` itself if Sunday).
fn nearest_sunday(d: NaiveDate) -> NaiveDate {
    let wd = (python_weekday(d) + 1) % 7; // 0 = Sunday
    if wd <= 3 {
        d - chrono::Duration::days(wd)
    } else {
        d + chrono::Duration::days(7 - wd)
    }
}

fn sunday_on_or_before(d: NaiveDate) -> NaiveDate {
    let wd = (python_weekday(d) + 1) % 7;
    d - chrono::Duration::days(wd)
}

/// Proper number (4–29) assigned to the Sunday nearest a given date in
/// Ordinary Time after Pentecost. The date ranges mark the CENTRE of each
/// Proper's week.
const PROPER_CENTRES: &[(u32, (u32, u32))] = &[
    (4, (5, 31)),
    (5, (6, 7)),
    (6, (6, 14)),
    (7, (6, 21)),
    (8, (6, 28)),
    (9, (7, 5)),
    (10, (7, 12)),
    (11, (7, 19)),
    (12, (7, 26)),
    (13, (8, 2)),
    (14, (8, 9)),
    (15, (8, 16)),
    (16, (8, 23)),
    (17, (8, 30)),
    (18, (9, 6)),
    (19, (9, 13)),
    (20, (9, 20)),
    (21, (9, 27)),
    (22, (10, 4)),
    (23, (10, 11)),
    (24, (10, 18)),
    (25, (10, 25)),
    (26, (11, 1)),
    (27, (11, 8)),
    (28, (11, 15)),
    (29, (11, 22)),
];

/// Return Proper number (4–29) for a date in Ordinary Time, or `None`.
fn proper_for_date(d: NaiveDate) -> Option<u32> {
    let year = d.year();
    let mut best_proper = None;
    let mut best_diff = 999i64;
    for &(proper_num, (mo, dy)) in PROPER_CENTRES {
        let centre = NaiveDate::from_ymd_opt(year, mo, dy).unwrap();
        let diff = (d - centre).num_days().abs();
        if diff < best_diff {
            best_diff = diff;
            best_proper = Some(proper_num);
        }
    }
    if best_diff <= 3 {
        best_proper
    } else {
        None
    }
}

fn lookup(key_year: &str, sunday_id: &str) -> Option<Readings> {
    READINGS
        .iter()
        .find(|&&(yk, sid, _)| yk == key_year && sid == sunday_id)
        .map(|&(_, _, r)| r)
        .or_else(|| {
            READINGS
                .iter()
                .find(|&&(yk, sid, _)| yk == "ALL" && sid == sunday_id)
                .map(|&(_, _, r)| r)
        })
}

fn colour_for(season_key: &str) -> (&'static str, &'static str) {
    COLOURS
        .iter()
        .find(|&&(k, _)| k == season_key)
        .map(|&(_, c)| c)
        .unwrap_or(("Green", "#15803D"))
}

fn result_for(
    lec_year: &str,
    season_key: &str,
    week_label: &str,
    sunday_id: &str,
    year_key: Option<&str>,
) -> LiturgicalInfo {
    let yk = year_key.unwrap_or(lec_year);
    let r = lookup(yk, sunday_id);
    let (colour_name, colour_hex) = colour_for(season_key);
    LiturgicalInfo {
        season: season_key.to_string(),
        week: week_label.to_string(),
        year: lec_year.to_string(),
        colour: colour_name.to_string(),
        colour_hex: colour_hex.to_string(),
        ot: r.map(|r| r.0).unwrap_or("—").to_string(),
        psalm: r.map(|r| r.1).unwrap_or("—").to_string(),
        epistle: r.map(|r| r.2).unwrap_or("—").to_string(),
        gospel: r.map(|r| r.3).unwrap_or("—").to_string(),
        found: r.is_some(),
    }
}

/// Return liturgical information (season, week, colour, readings) for a
/// given date. Mirrors Rubric's `get_liturgical_info` field-for-field so the
/// two apps agree on any given Sunday.
pub fn get_liturgical_info(d: NaiveDate) -> LiturgicalInfo {
    let year = d.year();
    let lec_year = lectionary_year(d);

    let e = easter(year);
    let prev_e = easter(year - 1);
    let adv = advent_sunday(year);
    let prev_adv = advent_sunday(year - 1);

    let ash_wed = e - chrono::Duration::days(46);
    let palm_sun = e - chrono::Duration::days(7);
    let holy_thu = e - chrono::Duration::days(3);
    let good_fri = e - chrono::Duration::days(2);
    let pentecost = e + chrono::Duration::days(49);
    let trinity = pentecost + chrono::Duration::days(7);
    let christ_king = adv - chrono::Duration::days(7);

    let prev_pentecost = prev_e + chrono::Duration::days(49);
    let prev_trinity = prev_pentecost + chrono::Duration::days(7);
    let _prev_christ_king = prev_adv - chrono::Duration::days(7);

    // ── Special fixed days ──────────────────────────────────────────────
    if d == good_fri {
        return result_for(lec_year, "Good Friday", "Good Friday", "GoodFriday", None);
    }
    if d == holy_thu {
        return result_for(
            lec_year,
            "Holy Thursday",
            "Maundy Thursday",
            "HolyThursday",
            None,
        );
    }
    if d == ash_wed {
        return result_for(lec_year, "Lent", "Ash Wednesday", "AshWednesday", None);
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
                None,
            );
        }
    }

    // ── Christmas ────────────────────────────────────────────────────────
    let xmas = NaiveDate::from_ymd_opt(year, 12, 25).unwrap();
    let dec31 = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
    if xmas <= d && d <= dec31 {
        if d == xmas {
            let (colour_name, colour_hex) = colour_for("Christmas");
            return LiturgicalInfo {
                season: "Christmas".to_string(),
                week: "Christmas Day".to_string(),
                year: lec_year.to_string(),
                colour: colour_name.to_string(),
                colour_hex: colour_hex.to_string(),
                ot: "Isa 9:2–7".to_string(),
                psalm: "Ps 96".to_string(),
                epistle: "Titus 2:11–14".to_string(),
                gospel: "Luke 2:1–14".to_string(),
                found: true,
            };
        }
        let days_to_next_sun = (6 - python_weekday(d)) % 7;
        let next_sun = d + chrono::Duration::days(days_to_next_sun);
        if next_sun.month() == 12 && next_sun.day() <= 31 {
            return result_for(
                lec_year,
                "Christmas",
                &format!("Christmas 1, Year {lec_year}"),
                &format!("Christmas1{lec_year}"),
                None,
            );
        }
        let (colour_name, colour_hex) = colour_for("Christmas");
        return LiturgicalInfo {
            season: "Christmas".to_string(),
            week: "Christmastide".to_string(),
            year: lec_year.to_string(),
            colour: colour_name.to_string(),
            colour_hex: colour_hex.to_string(),
            ot: "—".to_string(),
            psalm: "—".to_string(),
            epistle: "—".to_string(),
            gospel: "—".to_string(),
            found: false,
        };
    }

    let jan1 = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
    let jan5 = NaiveDate::from_ymd_opt(year, 1, 5).unwrap();
    if jan1 <= d && d <= jan5 && is_sunday(d) {
        return result_for(lec_year, "Christmas", "Christmas 2", "Christmas2", None);
    }

    // ── Epiphany season ──────────────────────────────────────────────────
    let epiphany = NaiveDate::from_ymd_opt(year, 1, 6).unwrap();

    if d == epiphany {
        let (colour_name, colour_hex) = colour_for("Christmas");
        return LiturgicalInfo {
            season: "Epiphany".to_string(),
            week: "Epiphany (Jan 6)".to_string(),
            year: lec_year.to_string(),
            colour: colour_name.to_string(),
            colour_hex: colour_hex.to_string(),
            ot: "Isa 60:1–6".to_string(),
            psalm: "Ps 72:1–7, 10–14".to_string(),
            epistle: "Eph 3:1–12".to_string(),
            gospel: "Matt 2:1–12".to_string(),
            found: true,
        };
    }

    if d == nearest_sunday(epiphany) {
        return result_for(
            lec_year,
            "Baptism",
            &format!("Baptism of the Lord, Year {lec_year}"),
            "Epiphany1",
            None,
        );
    }

    // Transfiguration = Sunday before Ash Wednesday
    let transfig = sunday_on_or_before(ash_wed - chrono::Duration::days(1));
    if d == transfig {
        return result_for(
            lec_year,
            "Transfiguration",
            &format!("Transfiguration, Year {lec_year}"),
            "Transfiguration",
            None,
        );
    }

    if epiphany < d && d < ash_wed && is_sunday(d) {
        let ep1 = nearest_sunday(epiphany);
        let weeks_after = (d - ep1).num_days() / 7;
        let ep_num = weeks_after + 1;
        if (1..=8).contains(&ep_num) {
            return result_for(
                lec_year,
                "Epiphany",
                &format!("Epiphany {ep_num}, Year {lec_year}"),
                &format!("Epiphany{ep_num}"),
                None,
            );
        }
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
                    &format!("Palm / Passion Sunday, Year {lec_year}"),
                    "PalmSunday",
                    None,
                );
            }
            return result_for(
                lec_year,
                "Lent",
                &format!("Lent {lent_week}, Year {lec_year}"),
                &format!("Lent{lent_week}"),
                None,
            );
        }
    }

    // ── Easter ───────────────────────────────────────────────────────────
    if d == e {
        return result_for(
            lec_year,
            "Easter",
            &format!("Easter Sunday, Year {lec_year}"),
            "Easter",
            None,
        );
    }

    for easter_week in 2..=7i64 {
        let easter_sun = e + chrono::Duration::weeks(easter_week - 1);
        if d == easter_sun {
            if easter_week == 7 {
                return result_for(
                    lec_year,
                    "Easter",
                    &format!("Easter 7 (Ascension Sunday), Year {lec_year}"),
                    "Easter7",
                    None,
                );
            }
            return result_for(
                lec_year,
                "Easter",
                &format!("Easter {easter_week}, Year {lec_year}"),
                &format!("Easter{easter_week}"),
                None,
            );
        }
    }

    let ascension = e + chrono::Duration::days(39);
    if d == ascension {
        return result_for(lec_year, "Easter", "Ascension of the Lord", "Ascension", None);
    }

    // ── Pentecost / Trinity / Ordinary Time ─────────────────────────────
    if d == pentecost {
        return result_for(
            lec_year,
            "Pentecost",
            &format!("Day of Pentecost, Year {lec_year}"),
            "Pentecost",
            None,
        );
    }

    if d == trinity {
        return result_for(
            lec_year,
            "Trinity",
            &format!("Trinity Sunday, Year {lec_year}"),
            "Trinity",
            None,
        );
    }

    if d == christ_king {
        return result_for(
            lec_year,
            "Christ the King",
            &format!("Reign of Christ / Christ the King, Year {lec_year}"),
            "Proper29",
            None,
        );
    }

    // Ordinary Time after Pentecost
    if trinity < d && d < adv {
        if let Some(proper) = proper_for_date(d) {
            let label = if proper == 29 {
                "Christ the King".to_string()
            } else {
                format!("Ordinary {proper}")
            };
            let sid = format!("Proper{proper}");
            return result_for(
                lec_year,
                "Ordinary",
                &format!("{label}, Year {lec_year}"),
                &sid,
                None,
            );
        }
    }

    // Previous year's Ordinary Time (for dates Jan–May before current Lent)
    if prev_trinity <= d && d < ash_wed {
        if let Some(proper) = proper_for_date(d) {
            let prev_year = lectionary_year(d);
            return result_for(
                lec_year,
                "Ordinary",
                &format!("Ordinary {proper}, Year {prev_year}"),
                &format!("Proper{proper}"),
                Some(prev_year),
            );
        }
    }

    // Fallback
    let (colour_name, colour_hex) = colour_for("Ordinary");
    LiturgicalInfo {
        season: "Ordinary".to_string(),
        week: "Ordinary Time".to_string(),
        year: lec_year.to_string(),
        colour: colour_name.to_string(),
        colour_hex: colour_hex.to_string(),
        ot: "—".to_string(),
        psalm: "—".to_string(),
        epistle: "—".to_string(),
        gospel: "—".to_string(),
        found: false,
    }
}

impl From<LiturgicalInfo> for crate::model::LectionaryLink {
    fn from(info: LiturgicalInfo) -> Self {
        crate::model::LectionaryLink {
            week: info.week,
            season: info.season,
            year: info.year,
            colour: info.colour,
            colour_hex: info.colour_hex,
            ot: info.ot,
            psalm: info.psalm,
            epistle: info.epistle,
            gospel: info.gospel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // ── Easter ───────────────────────────────────────────────────────────

    #[test]
    fn easter_2024() {
        assert_eq!(easter(2024), d(2024, 3, 31));
    }

    #[test]
    fn easter_2025() {
        assert_eq!(easter(2025), d(2025, 4, 20));
    }

    #[test]
    fn easter_2026() {
        assert_eq!(easter(2026), d(2026, 4, 5));
    }

    #[test]
    fn easter_2027() {
        assert_eq!(easter(2027), d(2027, 3, 28));
    }

    #[test]
    fn easter_2028() {
        assert_eq!(easter(2028), d(2028, 4, 16));
    }

    // ── Advent Sunday ────────────────────────────────────────────────────

    #[test]
    fn advent_2024() {
        assert_eq!(advent_sunday(2024), d(2024, 12, 1));
    }

    #[test]
    fn advent_2025() {
        assert_eq!(advent_sunday(2025), d(2025, 11, 30));
    }

    #[test]
    fn advent_2026() {
        assert_eq!(advent_sunday(2026), d(2026, 11, 29));
    }

    #[test]
    fn advent_is_sunday() {
        for year in 2020..2030 {
            let day = advent_sunday(year);
            assert_eq!(
                day.weekday(),
                chrono::Weekday::Sun,
                "Advent {year} is not a Sunday: {day}"
            );
        }
    }

    // ── Lectionary year ──────────────────────────────────────────────────

    #[test]
    fn year_b_advent_2023() {
        assert_eq!(lectionary_year(d(2023, 12, 3)), "B");
    }

    #[test]
    fn year_b_christmas_2023() {
        assert_eq!(lectionary_year(d(2023, 12, 25)), "B");
    }

    #[test]
    fn year_c_advent_2024() {
        assert_eq!(lectionary_year(d(2024, 12, 1)), "C");
    }

    #[test]
    fn year_b_mid_2024() {
        assert_eq!(lectionary_year(d(2024, 6, 1)), "B");
    }

    #[test]
    fn year_c_mid_2025() {
        assert_eq!(lectionary_year(d(2025, 6, 1)), "C");
    }

    #[test]
    fn year_a_mid_2026() {
        assert_eq!(lectionary_year(d(2026, 6, 1)), "A");
    }

    #[test]
    fn three_year_cycle() {
        let years: std::collections::HashSet<_> =
            (0..6).map(|i| lectionary_year(d(2020 + i, 6, 1))).collect();
        assert_eq!(years.len(), 3);
    }

    // ── Proper for date ──────────────────────────────────────────────────

    #[test]
    fn proper_4_june_1() {
        assert_eq!(proper_for_date(d(2024, 6, 1)), Some(4));
    }

    #[test]
    fn proper_4_may_28() {
        assert_eq!(proper_for_date(d(2024, 5, 28)), Some(4));
    }

    #[test]
    fn proper_29_nov_23() {
        assert_eq!(proper_for_date(d(2024, 11, 23)), Some(29));
    }

    #[test]
    fn proper_29_nov_19() {
        assert_eq!(proper_for_date(d(2024, 11, 19)), Some(29));
    }

    #[test]
    fn proper_in_range() {
        for date in [d(2024, 6, 1), d(2024, 7, 1), d(2024, 9, 1), d(2024, 11, 1)] {
            let p = proper_for_date(date).unwrap();
            assert!((4..=29).contains(&p));
        }
    }

    // ── get_liturgical_info: Lent ────────────────────────────────────────

    #[test]
    fn ash_wednesday_2024_season() {
        assert_eq!(get_liturgical_info(d(2024, 2, 14)).season, "Lent");
    }

    #[test]
    fn ash_wednesday_2024_week() {
        assert_eq!(get_liturgical_info(d(2024, 2, 14)).week, "Ash Wednesday");
    }

    #[test]
    fn lent_has_readings() {
        let info = get_liturgical_info(d(2024, 3, 10)); // Lent 4
        assert!(info.found);
        assert_ne!(info.gospel, "—");
    }

    // ── Holy Week ────────────────────────────────────────────────────────

    #[test]
    fn palm_sunday_2024_season() {
        assert_eq!(get_liturgical_info(d(2024, 3, 24)).season, "Palm Sunday");
    }

    #[test]
    fn good_friday_2024() {
        let info = get_liturgical_info(d(2024, 3, 29));
        assert!(info.found);
        assert_eq!(info.season, "Good Friday");
    }

    // ── Easter ───────────────────────────────────────────────────────────

    #[test]
    fn easter_sunday_2024_season() {
        assert_eq!(get_liturgical_info(d(2024, 3, 31)).season, "Easter");
    }

    #[test]
    fn easter_sunday_2024_week() {
        assert!(get_liturgical_info(d(2024, 3, 31)).week.contains("Easter Sunday"));
    }

    #[test]
    fn easter_has_readings() {
        let info = get_liturgical_info(d(2024, 3, 31));
        assert!(info.found);
        assert_ne!(info.gospel, "—");
    }

    // ── Advent ───────────────────────────────────────────────────────────

    #[test]
    fn advent_1_2024_season() {
        assert_eq!(get_liturgical_info(d(2024, 12, 1)).season, "Advent");
    }

    #[test]
    fn advent_has_readings() {
        let info = get_liturgical_info(d(2024, 12, 1));
        assert!(info.found);
        assert_ne!(info.gospel, "—");
    }

    // ── Christmas ────────────────────────────────────────────────────────

    #[test]
    fn christmas_2024() {
        let info = get_liturgical_info(d(2024, 12, 25));
        assert_eq!(info.season, "Christmas");
        assert!(info.found);
    }

    // ── Ordinary Time ────────────────────────────────────────────────────

    #[test]
    fn ordinary_time_season() {
        assert_eq!(get_liturgical_info(d(2024, 7, 14)).season, "Ordinary");
    }

    #[test]
    fn ordinary_time_has_proper() {
        assert!(get_liturgical_info(d(2024, 7, 14)).week.contains("Ordinary"));
    }

    #[test]
    fn ordinary_time_has_readings() {
        assert!(get_liturgical_info(d(2024, 7, 14)).found);
    }

    // ── Colour ───────────────────────────────────────────────────────────

    #[test]
    fn advent_colour_purple() {
        let colour = get_liturgical_info(d(2024, 12, 1)).colour.to_lowercase();
        assert!(colour == "purple" || colour == "violet");
    }

    #[test]
    fn easter_colour_white() {
        let colour = get_liturgical_info(d(2024, 3, 31)).colour.to_lowercase();
        assert!(colour.contains("white"));
    }

    #[test]
    fn ordinary_colour_green() {
        assert_eq!(get_liturgical_info(d(2024, 7, 14)).colour.to_lowercase(), "green");
    }

    #[test]
    fn lent_colour_purple() {
        let colour = get_liturgical_info(d(2024, 3, 10)).colour.to_lowercase();
        assert!(colour == "purple" || colour == "violet");
    }

    // ── Lectionary year in info ──────────────────────────────────────────

    #[test]
    fn year_in_info() {
        let year = get_liturgical_info(d(2024, 7, 14)).year;
        assert!(year == "A" || year == "B" || year == "C");
    }

    #[test]
    fn year_b_easter_2024() {
        assert_eq!(get_liturgical_info(d(2024, 3, 31)).year, "B");
    }

    // ── Regression: Jan–May weekdays used to crash (month + 12 overflow) ──

    #[test]
    fn epiphany_season_weekdays_do_not_crash() {
        let start = d(2026, 1, 1);
        for i in 0..90 {
            let _ = get_liturgical_info(start + chrono::Duration::days(i));
        }
    }

    #[test]
    fn january_weekday_no_crash_specific() {
        let info = get_liturgical_info(d(2026, 1, 13));
        assert!(!info.season.is_empty());
    }

    // ── Regression: Christmas Day on a Sunday pushed Dec 26-31 into ───────
    // ── "Ordinary Time" instead of staying in the Christmas season ────────

    #[test]
    fn dec26_31_stays_christmas_when_christmas_day_is_sunday() {
        assert_eq!(d(2033, 12, 25).weekday(), chrono::Weekday::Sun);
        for day in 26..=31 {
            let info = get_liturgical_info(d(2033, 12, day));
            assert_eq!(info.season, "Christmas");
        }
    }

    // ── Golden test: every Sunday of a full year resolves without panicking
    // and agrees with a hand-checked set of expected seasons ──────────────

    #[test]
    fn golden_year_of_sundays_2026() {
        // 2026: Easter is April 5, Advent Sunday is Nov 29 (see easter_2026 /
        // advent_2026 above). Walk every Sunday in the civil year and assert
        // season transitions land where a preacher would expect them.
        let expectations: &[(&str, &str)] = &[
            ("2026-01-04", "Christmas"), // Christmas 2 (Sun before Epiphany)
            // Epiphany (Jan 6, 2026) falls on a Tuesday, so its nearest
            // Sunday is Jan 4 — already claimed by the Christmas 2 branch
            // above, which is checked first. That means 2026 has no
            // separate "Baptism of the Lord" Sunday at all; the season
            // goes straight from Christmas 2 to Epiphany 2. Matches
            // Rubric's rcl_data.get_liturgical_info exactly (verified
            // against the Python implementation for this date).
            ("2026-01-11", "Epiphany"),
            ("2026-01-18", "Epiphany"),
            ("2026-01-25", "Epiphany"),
            ("2026-02-01", "Epiphany"),
            ("2026-02-08", "Epiphany"),
            ("2026-02-15", "Transfiguration"),
            ("2026-02-22", "Lent"),
            ("2026-03-01", "Lent"),
            ("2026-03-08", "Lent"),
            ("2026-03-15", "Lent"),
            ("2026-03-22", "Lent"),
            ("2026-03-29", "Palm Sunday"),
            ("2026-04-05", "Easter"),
            ("2026-04-12", "Easter"),
            ("2026-04-19", "Easter"),
            ("2026-04-26", "Easter"),
            ("2026-05-03", "Easter"),
            ("2026-05-10", "Easter"),
            ("2026-05-17", "Easter"), // Easter 7 / Ascension Sunday
            ("2026-05-24", "Pentecost"),
            ("2026-05-31", "Trinity"),
            ("2026-11-22", "Christ the King"),
            ("2026-11-29", "Advent"),
            ("2026-12-06", "Advent"),
            ("2026-12-13", "Advent"),
            ("2026-12-20", "Advent"),
            ("2026-12-25", "Christmas"),
            ("2026-12-27", "Christmas"),
        ];
        for &(date_str, expected_season) in expectations {
            let parts: Vec<i32> = date_str.split('-').map(|p| p.parse().unwrap()).collect();
            let date = d(parts[0], parts[1] as u32, parts[2] as u32);
            let info = get_liturgical_info(date);
            assert_eq!(
                info.season, expected_season,
                "{date_str}: expected season {expected_season}, got {}",
                info.season
            );
        }
    }

    #[test]
    fn golden_year_all_sundays_resolve_without_panic_and_have_valid_year() {
        let mut day = d(2026, 1, 1);
        let end = d(2027, 1, 1);
        while day < end {
            if is_sunday(day) {
                let info = get_liturgical_info(day);
                assert!(matches!(info.year.as_str(), "A" | "B" | "C"));
            }
            day += chrono::Duration::days(1);
        }
    }
}
