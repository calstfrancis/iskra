//! RCL Track 2 (Complementary) Old Testament + Psalm pairings for Ordinary
//! Time (Propers 4–29), Years A/B/C. Track 1 (Semicontinuous) data lives in
//! `rcl.rs::READINGS`; Epistle and Gospel are identical between tracks and
//! not repeated here.
//!
//! Where the official table offers a choice between a deuterocanonical
//! (Apocrypha) reading and a canonical alternative — Year A Proper 11 and
//! Proper 27, Year B Proper 8 and Proper 20, Year C Proper 17 and Proper 25 —
//! the canonical alternative is used here, to stay consistent with the rest
//! of this file (Protestant canon only).
//!
//! Source(s), cross-checked against each other:
//! - The Revised Common Lectionary, Consultation on Common Texts (the RCL's
//!   publisher), official Year A/B/C tables:
//!   <https://www.commontexts.org/wp-content/uploads/2015/11/RCL_YearA_Web.pdf>
//!   <https://www.commontexts.org/wp-content/uploads/2015/11/RCL_YearB_Web.pdf>
//!   <https://www.commontexts.org/wp-content/uploads/2015/11/RCL_YearC_Web.pdf>
//! - Presbyterian Church (U.S.A.), Office of Theology and Worship / Faith and
//!   Order, "RCL Full Year" annual reprints (used to corroborate the CCT
//!   tables and to fill in Sundays a single calendar year skips):
//!   <https://irp.cdn-website.com/050afefb/files/uploaded/2024-RCL-Full-Year.pdf> (Year B)
//!   <https://pcusa.org/sites/default/files/2024-12/2025%20RCL%20Full%20Year.pdf> (Year C)
//!   <https://pcusa.org/sites/default/files/2025-12/Revised-Common-Lectionary-Full-Year_2026.pdf> (Year A)
//! - The Lectionary Page (Episcopal RCL reprint), used to spot-check
//!   individual Sundays: <https://www.lectionarypage.net/>
//!
//! Fetched 2026-07-19.

pub static COMPLEMENTARY: &[(&str, &str, (&str, &str))] = &[
    ("A", "Proper4", ("Deut 11:18–21, 26–28", "Ps 31:1–5, 19–24")),
    ("B", "Proper4", ("Deut 5:12–15", "Ps 81:1–10")),
    ("C", "Proper4", ("1 Kgs 8:22–23, 41–43", "Ps 96:1–9")),
    ("A", "Proper5", ("Hos 5:15–6:6", "Ps 50:7–15")),
    ("B", "Proper5", ("Gen 3:8–15", "Ps 130")),
    ("C", "Proper5", ("1 Kgs 17:17–24", "Ps 30")),
    ("A", "Proper6", ("Exod 19:2–8a", "Ps 100")),
    ("B", "Proper6", ("Ezek 17:22–24", "Ps 92:1–4, 12–15")),
    ("C", "Proper6", ("2 Sam 11:26–12:10, 13–15", "Ps 32")),
    ("A", "Proper7", ("Jer 20:7–13", "Ps 69:7–10[11–15]16–18")),
    ("B", "Proper7", ("Job 38:1–11", "Ps 107:1–3, 23–32")),
    ("C", "Proper7", ("Isa 65:1–9", "Ps 22:19–28")),
    ("A", "Proper8", ("Jer 28:5–9", "Ps 89:1–4, 15–18")),
    ("B", "Proper8", ("Lam 3:22–33", "Ps 30")),
    ("C", "Proper8", ("1 Kgs 19:15–16, 19–21", "Ps 16")),
    ("A", "Proper9", ("Zech 9:9–12", "Ps 145:8–14")),
    ("B", "Proper9", ("Ezek 2:1–5", "Ps 123")),
    ("C", "Proper9", ("Isa 66:10–14", "Ps 66:1–9")),
    ("A", "Proper10", ("Isa 55:10–13", "Ps 65:[1–8]9–13")),
    ("B", "Proper10", ("Amos 7:7–15", "Ps 85:8–13")),
    ("C", "Proper10", ("Deut 30:9–14", "Ps 25:1–10")),
    ("A", "Proper11", ("Isa 44:6–8", "Ps 86:11–17")),
    ("B", "Proper11", ("Jer 23:1–6", "Ps 23")),
    ("C", "Proper11", ("Gen 18:1–10a", "Ps 15")),
    ("A", "Proper12", ("1 Kgs 3:5–12", "Ps 119:129–136")),
    ("B", "Proper12", ("2 Kgs 4:42–44", "Ps 145:10–18")),
    ("C", "Proper12", ("Gen 18:20–32", "Ps 138")),
    ("A", "Proper13", ("Isa 55:1–5", "Ps 145:8–9, 14–21")),
    ("B", "Proper13", ("Exod 16:2–4, 9–15", "Ps 78:23–29")),
    ("C", "Proper13", ("Eccl 1:2, 12–14; 2:18–23", "Ps 49:1–12")),
    ("A", "Proper14", ("1 Kgs 19:9–18", "Ps 85:8–13")),
    ("B", "Proper14", ("1 Kgs 19:4–8", "Ps 34:1–8")),
    ("C", "Proper14", ("Gen 15:1–6", "Ps 33:12–22")),
    ("A", "Proper15", ("Isa 56:1, 6–8", "Ps 67")),
    ("B", "Proper15", ("Prov 9:1–6", "Ps 34:9–14")),
    ("C", "Proper15", ("Jer 23:23–29", "Ps 82")),
    ("A", "Proper16", ("Isa 51:1–6", "Ps 138")),
    ("B", "Proper16", ("Josh 24:1–2a, 14–18", "Ps 34:15–22")),
    ("C", "Proper16", ("Isa 58:9b–14", "Ps 103:1–8")),
    ("A", "Proper17", ("Jer 15:15–21", "Ps 26:1–8")),
    ("B", "Proper17", ("Deut 4:1–2, 6–9", "Ps 15")),
    ("C", "Proper17", ("Prov 25:6–7", "Ps 112")),
    ("A", "Proper18", ("Ezek 33:7–11", "Ps 119:33–40")),
    ("B", "Proper18", ("Isa 35:4–7a", "Ps 146")),
    ("C", "Proper18", ("Deut 30:15–20", "Ps 1")),
    ("A", "Proper19", ("Gen 50:15–21", "Ps 103:[1–7]8–13")),
    ("B", "Proper19", ("Isa 50:4–9a", "Ps 116:1–9")),
    ("C", "Proper19", ("Exod 32:7–14", "Ps 51:1–10")),
    ("A", "Proper20", ("Jon 3:10–4:11", "Ps 145:1–8")),
    ("B", "Proper20", ("Jer 11:18–20", "Ps 54")),
    ("C", "Proper20", ("Amos 8:4–7", "Ps 113")),
    ("A", "Proper21", ("Ezek 18:1–4, 25–32", "Ps 25:1–9")),
    ("B", "Proper21", ("Num 11:4–6, 10–16, 24–29", "Ps 19:7–14")),
    ("C", "Proper21", ("Amos 6:1a, 4–7", "Ps 146")),
    ("A", "Proper22", ("Isa 5:1–7", "Ps 80:7–15")),
    ("B", "Proper22", ("Gen 2:18–24", "Ps 8")),
    ("C", "Proper22", ("Hab 1:1–4; 2:1–4", "Ps 37:1–9")),
    ("A", "Proper23", ("Isa 25:1–9", "Ps 23")),
    ("B", "Proper23", ("Amos 5:6–7, 10–15", "Ps 90:12–17")),
    ("C", "Proper23", ("2 Kgs 5:1–3, 7–15c", "Ps 111")),
    ("A", "Proper24", ("Isa 45:1–7", "Ps 96:1–9[10–13]")),
    ("B", "Proper24", ("Isa 53:4–12", "Ps 91:9–16")),
    ("C", "Proper24", ("Gen 32:22–31", "Ps 121")),
    ("A", "Proper25", ("Lev 19:1–2, 15–18", "Ps 1")),
    ("B", "Proper25", ("Jer 31:7–9", "Ps 126")),
    ("C", "Proper25", ("Jer 14:7–10, 19–22", "Ps 84:1–7")),
    ("A", "Proper26", ("Mic 3:5–12", "Ps 43")),
    ("B", "Proper26", ("Deut 6:1–9", "Ps 119:1–8")),
    ("C", "Proper26", ("Isa 1:10–18", "Ps 32:1–7")),
    ("A", "Proper27", ("Amos 5:18–24", "Ps 70")),
    ("B", "Proper27", ("1 Kgs 17:8–16", "Ps 146")),
    ("C", "Proper27", ("Job 19:23–27a", "Ps 17:1–9")),
    ("A", "Proper28", ("Zeph 1:7, 12–18", "Ps 90:1–8[9–11]12")),
    ("B", "Proper28", ("Dan 12:1–3", "Ps 16")),
    ("C", "Proper28", ("Mal 4:1–2a", "Ps 98")),
    ("A", "Proper29", ("Ezek 34:11–16, 20–24", "Ps 95:1–7a")),
    ("B", "Proper29", ("Dan 7:9–10, 13–14", "Ps 93")),
    ("C", "Proper29", ("Jer 23:1–6", "Ps 46")),
];
