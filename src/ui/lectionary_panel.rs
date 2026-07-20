//! Sidebar panel — the lectionary sidebar's first (and so far only) tenant.
//! Shows a lectionary/track picker (hidden by default — see Simple Mode
//! below), the season + colour swatch, the week label, and that
//! lectionary's readings for the sermon's planned date. Empty state when no
//! date is set.
//!
//! **Simple Mode**: most users pick one lectionary and never touch it
//! again, so the picker is hidden by default (`Config::lectionary_simple_mode`,
//! on by default) and revealed via the "simple" toggle in the status bar
//! (`ui::status_bar`) — same on/off-by-font-weight convention as every
//! other status-bar toggle in this app.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, DropDown, Label, Orientation, Popover, StringList};
use libadwaita as adw;

use crate::commands::{Cmd, SermonTagKind};
use crate::lectionary::{self, LectionaryKind, RclTrack};
use crate::model::Sermon;
use crate::state::AppState;
use crate::ui::editor::ApplyFn;
use crate::ui::styles;

pub struct LectionaryPanel {
    pub root: GtkBox,
    empty_status: adw::StatusPage,
    content: GtkBox,
    picker_box: GtkBox,
    kind_dropdown: DropDown,
    track_dropdown: DropDown,
    dot: GtkBox,
    season_label: Label,
    week_label: Label,
    readings_box: GtkBox,
    current_s_tags: RefCell<Vec<String>>,
    apply: RefCell<Option<ApplyFn>>,
    state: RefCell<Option<Rc<RefCell<AppState>>>>,
    /// Guards the dropdowns' `connect_selected_notify` handlers against
    /// firing when `refresh` sets their selection to match config — without
    /// this, every refresh (e.g. after any unrelated edit) would re-dispatch
    /// a lectionary-switch command.
    updating: Cell<bool>,
}

impl LectionaryPanel {
    pub fn new() -> Rc<Self> {
        let root = GtkBox::new(Orientation::Vertical, 0);

        let header = Label::new(Some("Lectionary"));
        header.add_css_class("sidebar-header");
        header.set_xalign(0.0);
        root.append(&header);

        let empty_status = adw::StatusPage::new();
        empty_status.set_icon_name(Some("x-office-calendar-symbolic"));
        empty_status.set_title("No date planned");
        empty_status.set_description(Some("Readings appear here once a date is planned."));
        empty_status.set_vexpand(true);
        root.append(&empty_status);

        let content = GtkBox::new(Orientation::Vertical, 10);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(4);
        content.set_margin_bottom(12);
        content.set_visible(false);

        // ── Picker (hidden in Simple Mode) ───────────────────────────────
        let picker_box = GtkBox::new(Orientation::Vertical, 4);
        picker_box.set_margin_bottom(4);
        let kind_labels: Vec<&str> = LectionaryKind::ALL.iter().map(|k| k.label()).collect();
        let kind_dropdown = DropDown::new(Some(StringList::new(&kind_labels)), gtk4::Expression::NONE);
        kind_dropdown.set_tooltip_text(Some("Lectionary"));
        picker_box.append(&kind_dropdown);
        let track_labels: Vec<&str> = RclTrack::ALL.iter().map(|t| t.label()).collect();
        let track_dropdown = DropDown::new(Some(StringList::new(&track_labels)), gtk4::Expression::NONE);
        track_dropdown.set_tooltip_text(Some("RCL Track (Ordinary Time OT + Psalm pairing)"));
        picker_box.append(&track_dropdown);
        content.append(&picker_box);

        let season_row = GtkBox::new(Orientation::Horizontal, 6);
        let dot = GtkBox::new(Orientation::Horizontal, 0);
        dot.add_css_class("season-dot");
        dot.set_valign(gtk4::Align::Center);
        let season_label = Label::new(None);
        season_label.add_css_class("heading");
        season_label.set_xalign(0.0);
        season_row.append(&dot);
        season_row.append(&season_label);
        content.append(&season_row);

        let week_label = Label::new(None);
        week_label.add_css_class("dim-label");
        week_label.add_css_class("caption");
        week_label.set_xalign(0.0);
        week_label.set_wrap(true);
        content.append(&week_label);

        let readings_box = GtkBox::new(Orientation::Vertical, 6);
        readings_box.set_margin_top(6);
        content.append(&readings_box);

        root.append(&content);

        let panel = Rc::new(Self {
            root,
            empty_status,
            content,
            picker_box,
            kind_dropdown,
            track_dropdown,
            dot,
            season_label,
            week_label,
            readings_box,
            current_s_tags: RefCell::new(Vec::new()),
            apply: RefCell::new(None),
            state: RefCell::new(None),
            updating: Cell::new(false),
        });

        {
            let panel = panel.clone();
            panel.kind_dropdown.clone().connect_selected_notify(move |dd| {
                if panel.updating.get() {
                    return;
                }
                let new_kind = LectionaryKind::ALL[dd.selected() as usize];
                panel.on_kind_changed(new_kind);
            });
        }
        {
            let panel = panel.clone();
            panel.track_dropdown.clone().connect_selected_notify(move |dd| {
                if panel.updating.get() {
                    return;
                }
                let new_track = RclTrack::ALL[dd.selected() as usize];
                panel.on_track_changed(new_track);
            });
        }

        panel
    }

    /// Stores `apply`/`state` so the picker can route a lectionary/track
    /// switch through the single door and right-clicking a reading can add
    /// a scripture tag. Call once, after both exist.
    pub fn init(&self, apply: ApplyFn, state: Rc<RefCell<AppState>>) {
        *self.apply.borrow_mut() = Some(apply);
        *self.state.borrow_mut() = Some(state);
    }

    fn on_kind_changed(self: &Rc<Self>, new_kind: LectionaryKind) {
        let (Some(state), Some(apply)) = (self.state.borrow().clone(), self.apply.borrow().clone()) else {
            return;
        };
        let old_kind = state.borrow().config.selected_lectionary;
        if old_kind == new_kind {
            return;
        }
        state.borrow_mut().config.selected_lectionary = new_kind;
        let _ = state.borrow().config.save();
        self.recompute_lectionary(&state, &apply);
    }

    fn on_track_changed(self: &Rc<Self>, new_track: RclTrack) {
        let (Some(state), Some(apply)) = (self.state.borrow().clone(), self.apply.borrow().clone()) else {
            return;
        };
        let old_track = state.borrow().config.rcl_track;
        if old_track == new_track {
            return;
        }
        state.borrow_mut().config.rcl_track = new_track;
        let _ = state.borrow().config.save();
        self.recompute_lectionary(&state, &apply);
    }

    /// Re-resolves the open sermon's planned date under the now-current
    /// lectionary/track and routes it through `Cmd::SetPlannedDate` — reused
    /// rather than a new `Cmd` variant, since the date itself is unchanged
    /// and only the denormalized link needs recomputing.
    fn recompute_lectionary(&self, state: &Rc<RefCell<AppState>>, apply: &ApplyFn) {
        let (date, old_link) = {
            let st = state.borrow();
            (st.sermon.planned_date, st.sermon.lectionary.clone())
        };
        let Some(date) = date else { return };
        let (kind, track) = {
            let cfg = &state.borrow().config;
            (cfg.selected_lectionary, cfg.rcl_track)
        };
        let new_link = lectionary::get_info(kind, track, date);
        apply(Cmd::SetPlannedDate {
            old: (Some(date), old_link),
            new: (Some(date), Some(new_link)),
        });
    }

    pub fn refresh(self: &Rc<Self>, sermon: &Sermon) {
        *self.current_s_tags.borrow_mut() = sermon.s_tags.clone();

        if let Some(state) = self.state.borrow().clone() {
            let cfg = &state.borrow().config;
            let simple_mode = cfg.lectionary_simple_mode;
            let kind = cfg.selected_lectionary;
            let track = cfg.rcl_track;

            self.picker_box.set_visible(!simple_mode);
            self.track_dropdown.set_visible(kind == LectionaryKind::Rcl);

            self.updating.set(true);
            let kind_idx = LectionaryKind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
            if self.kind_dropdown.selected() != kind_idx as u32 {
                self.kind_dropdown.set_selected(kind_idx as u32);
            }
            let track_idx = RclTrack::ALL.iter().position(|t| *t == track).unwrap_or(0);
            if self.track_dropdown.selected() != track_idx as u32 {
                self.track_dropdown.set_selected(track_idx as u32);
            }
            self.updating.set(false);
        }

        match &sermon.lectionary {
            Some(link) => {
                self.empty_status.set_visible(false);
                self.content.set_visible(true);
                self.dot
                    .set_css_classes(&["season-dot", styles::season_dot_class(&link.colour_hex)]);
                self.season_label
                    .set_text(&format!("{} · {}", link.season, link.colour));
                self.week_label.set_text(&link.week);

                while let Some(child) = self.readings_box.first_child() {
                    self.readings_box.remove(&child);
                }
                for (label, citation) in link.readings_or_legacy() {
                    let (row, value) = reading_row(&label);
                    value.set_text(&citation);
                    wire_add_scripture_tag(self, &value);
                    self.readings_box.append(&row);
                }
            }
            None => {
                self.empty_status.set_visible(true);
                self.content.set_visible(false);
            }
        }
    }
}

/// Right-click a reading value to add its text as a scripture ("s.") tag —
/// a shortcut for the common case of tagging the sermon with the passage
/// it's actually about, without retyping the reference by hand in the
/// status bar's tag entry.
fn wire_add_scripture_tag(panel: &Rc<LectionaryPanel>, value: &Label) {
    let click = gtk4::GestureClick::new();
    click.set_button(gtk4::gdk::BUTTON_SECONDARY);
    {
        let panel = panel.clone();
        let value = value.clone();
        click.connect_pressed(move |gesture, _n_press, x, y| {
            let text = value.text().to_string();
            if text.is_empty() {
                return;
            }
            gesture.set_state(gtk4::EventSequenceState::Claimed);

            let popover = Popover::new();
            popover.set_parent(&value);
            popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.set_autohide(true);

            let already_tagged = panel.current_s_tags.borrow().contains(&text);
            let label = if already_tagged {
                "Already a scripture tag".to_string()
            } else {
                format!("Add \"{text}\" as scripture tag")
            };
            let btn = Button::with_label(&label);
            btn.add_css_class("flat");
            btn.set_sensitive(!already_tagged);
            popover.set_child(Some(&btn));
            {
                let panel = panel.clone();
                let popover_for_close = popover.clone();
                let text = text.clone();
                btn.connect_clicked(move |_| {
                    let Some(apply) = panel.apply.borrow().clone() else {
                        return;
                    };
                    let old = panel.current_s_tags.borrow().clone();
                    let mut new = old.clone();
                    new.push(text.clone());
                    apply(Cmd::SetSermonTags { kind: SermonTagKind::S, old, new });
                    popover_for_close.popdown();
                });
            }
            {
                let popover_for_closed = popover.clone();
                popover.connect_closed(move |_| popover_for_closed.unparent());
            }
            popover.popup();
        });
    }
    value.add_controller(click);
}

fn reading_row(label: &str) -> (GtkBox, Label) {
    let row = GtkBox::new(Orientation::Vertical, 1);
    let caption = Label::new(Some(label));
    caption.add_css_class("caption-heading");
    caption.add_css_class("dim-label");
    caption.set_xalign(0.0);
    let value = Label::new(None);
    value.set_xalign(0.0);
    value.set_wrap(true);
    value.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    row.append(&caption);
    row.append(&value);
    (row, value)
}
