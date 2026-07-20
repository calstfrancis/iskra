//! Multi-lectionary dispatch. `rcl` is the original (and still default)
//! Revised Common Lectionary implementation; `catholic` and `narrative` are
//! sibling lectionaries selectable from the sidebar picker
//! (`ui::lectionary_panel`). Each owns its own calendar/date logic under
//! this module, since only RCL and Catholic share a compatible one (see
//! `calendar.rs`) — the Narrative Lectionary's calendar is structurally
//! different (see `narrative.rs`).

pub mod calendar;
pub mod catholic;
mod catholic_data;
pub mod narrative;
mod narrative_data;
pub mod rcl;
mod rcl_complementary_data;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LectionaryKind {
    #[default]
    Rcl,
    Catholic,
    Narrative,
}

impl LectionaryKind {
    pub const ALL: [LectionaryKind; 3] =
        [LectionaryKind::Rcl, LectionaryKind::Catholic, LectionaryKind::Narrative];

    pub fn label(&self) -> &'static str {
        match self {
            LectionaryKind::Rcl => "Revised Common Lectionary",
            LectionaryKind::Catholic => "Roman Catholic Lectionary",
            LectionaryKind::Narrative => "Narrative Lectionary",
        }
    }
}

/// RCL's Ordinary Time OT+Psalm pairing — only meaningful when
/// `LectionaryKind::Rcl` is selected; ignored by the other two lectionaries.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RclTrack {
    #[default]
    Semicontinuous,
    Complementary,
}

impl RclTrack {
    pub const ALL: [RclTrack; 2] = [RclTrack::Semicontinuous, RclTrack::Complementary];

    pub fn label(&self) -> &'static str {
        match self {
            RclTrack::Semicontinuous => "Semicontinuous",
            RclTrack::Complementary => "Complementary",
        }
    }
}

/// Resolved lectionary data for a given date, in generic (label, citation)
/// pairs rather than fixed OT/Psalm/Epistle/Gospel fields, since the
/// Narrative Lectionary doesn't share RCL/Catholic's four-slot shape.
#[derive(Debug, Clone, PartialEq)]
pub struct LiturgicalInfo {
    pub season: String,
    pub week: String,
    pub year: String,
    pub colour: String,
    pub colour_hex: String,
    pub readings: Vec<(String, String)>,
    pub found: bool,
}

impl LiturgicalInfo {
    pub fn reading(&self, label: &str) -> &str {
        self.readings
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, v)| v.as_str())
            .unwrap_or("—")
    }
    pub fn ot(&self) -> &str {
        self.reading("OT")
    }
    pub fn psalm(&self) -> &str {
        self.reading("Psalm")
    }
    pub fn epistle(&self) -> &str {
        self.reading("Epistle")
    }
    pub fn gospel(&self) -> &str {
        self.reading("Gospel")
    }
}

impl From<LiturgicalInfo> for crate::model::LectionaryLink {
    fn from(info: LiturgicalInfo) -> Self {
        crate::model::LectionaryLink {
            source: LectionaryKind::Rcl, // overwritten by `get_info` below
            week: info.week,
            season: info.season,
            year: info.year,
            colour: info.colour,
            colour_hex: info.colour_hex,
            readings: info.readings,
            ot: String::new(),
            psalm: String::new(),
            epistle: String::new(),
            gospel: String::new(),
        }
    }
}

/// Resolves the given lectionary for `d`, tagging the result with `kind` so
/// the sidebar/export code knows which lectionary produced it. `track` only
/// affects `LectionaryKind::Rcl`.
pub fn get_info(kind: LectionaryKind, track: RclTrack, d: NaiveDate) -> crate::model::LectionaryLink {
    let info = match kind {
        LectionaryKind::Rcl => rcl::get_liturgical_info(d, track),
        LectionaryKind::Catholic => catholic::get_liturgical_info(d),
        LectionaryKind::Narrative => narrative::get_liturgical_info(d),
    };
    let mut link: crate::model::LectionaryLink = info.into();
    link.source = kind;
    link
}
