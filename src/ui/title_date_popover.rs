//! Header-centre control: a `MenuButton` styled to read like `AdwWindowTitle`
//! (bold title + a subtitle line combining the planned date with a coloured
//! season dot) that opens a popover with a title `Entry` and a `gtk::Calendar`
//! for picking the date. Picking a day resolves `rcl::get_liturgical_info` and
//! routes the result through `Cmd::SetPlannedDate` — date and lectionary
//! snapshot change as one undo step.
//!
//! Built in two phases, like `ui::editor::Editor`: `new()` constructs the
//! widget tree so it can be placed in the header immediately, and `init()`
//! wires signal handlers against `apply` once the single door exists —
//! `apply` itself is built from a clone of this widget (for `refresh`), so
//! wiring can't happen inside `new()` without a cycle.

use std::cell::RefCell;
use std::rc::Rc;

use chrono::{Datelike, NaiveDate};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Calendar, Entry, Label, MenuButton, Orientation, Popover};

use crate::commands::Cmd;
use crate::model::Sermon;
use crate::rcl;
use crate::state::AppState;
use crate::ui::editor::ApplyFn;
use crate::ui::styles;

pub struct TitleDatePopover {
    pub button: MenuButton,
    title_label: Label,
    date_label: Label,
    dot: GtkBox,
    title_entry: Entry,
    calendar: Calendar,
    clear_btn: Button,
}

impl TitleDatePopover {
    pub fn new(sermon: &Sermon) -> Rc<Self> {
        let title_label = Label::new(Some(sermon.display_title()));
        title_label.add_css_class("title");
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        let dot = GtkBox::new(Orientation::Horizontal, 0);
        dot.add_css_class("season-dot");
        dot.set_valign(gtk4::Align::Center);
        dot.set_visible(false);

        let date_label = Label::new(Some("No date planned"));
        date_label.add_css_class("subtitle");
        date_label.add_css_class("dim-label");
        date_label.add_css_class("caption");

        let subtitle_row = GtkBox::new(Orientation::Horizontal, 4);
        subtitle_row.set_halign(gtk4::Align::Center);
        subtitle_row.append(&dot);
        subtitle_row.append(&date_label);

        let content = GtkBox::new(Orientation::Vertical, 1);
        content.set_valign(gtk4::Align::Center);
        content.append(&title_label);
        content.append(&subtitle_row);

        let button = MenuButton::new();
        button.add_css_class("flat");
        button.set_child(Some(&content));

        // ── Popover content ─────────────────────────────────────────────
        let popover_body = GtkBox::new(Orientation::Vertical, 8);
        popover_body.set_margin_top(10);
        popover_body.set_margin_bottom(10);
        popover_body.set_margin_start(10);
        popover_body.set_margin_end(10);

        let title_entry = Entry::new();
        title_entry.set_placeholder_text(Some("Sermon title…"));
        title_entry.set_text(&sermon.title);
        popover_body.append(&title_entry);

        let calendar = Calendar::new();
        if let Some(d) = sermon.planned_date {
            select_calendar_date(&calendar, d);
        }
        popover_body.append(&calendar);

        let clear_btn = Button::with_label("Clear date");
        clear_btn.add_css_class("flat");
        clear_btn.set_halign(gtk4::Align::Start);
        clear_btn.set_visible(sermon.planned_date.is_some());
        popover_body.append(&clear_btn);

        let popover = Popover::new();
        popover.set_child(Some(&popover_body));
        button.set_popover(Some(&popover));

        Rc::new(Self {
            button,
            title_label,
            date_label,
            dot,
            title_entry,
            calendar,
            clear_btn,
        })
    }

    /// Wires the title entry and calendar against `apply`. Call once, after
    /// `apply` exists.
    pub fn init(self: &Rc<Self>, state: &Rc<RefCell<AppState>>, apply: ApplyFn) {
        {
            let state = state.clone();
            let apply = apply.clone();
            self.title_entry.connect_changed(move |e| {
                let old = state.borrow().sermon.title.clone();
                let new = e.text().to_string();
                if old != new {
                    apply(Cmd::SetTitle { old, new });
                }
            });
        }
        {
            let state = state.clone();
            let focus_ctl = gtk4::EventControllerFocus::new();
            focus_ctl.connect_leave(move |_| state.borrow_mut().undo.break_coalescing());
            self.title_entry.add_controller(focus_ctl);
        }
        {
            let state = state.clone();
            let apply = apply.clone();
            let clear_btn = self.clear_btn.clone();
            self.calendar.connect_day_selected(move |cal| {
                clear_btn.set_visible(true);
                if let Some(new_date) = calendar_date(cal) {
                    apply_planned_date(&state, &apply, Some(new_date));
                }
            });
        }
        {
            let state = state.clone();
            let apply = apply.clone();
            self.clear_btn.connect_clicked(move |btn| {
                btn.set_visible(false);
                apply_planned_date(&state, &apply, None);
            });
        }
    }

    /// Syncs the header display and popover contents to `sermon`. Safe to
    /// call after every `apply()` — resetting the calendar's selection to a
    /// date it already shows is a no-op from the user's perspective.
    pub fn refresh(&self, sermon: &Sermon) {
        self.title_label.set_text(sermon.display_title());
        if self.title_entry.text() != sermon.title {
            self.title_entry.set_text(&sermon.title);
        }

        match sermon.planned_date {
            Some(d) => {
                self.date_label
                    .set_text(&d.format("%B %-d, %Y").to_string());
                select_calendar_date(&self.calendar, d);
                self.clear_btn.set_visible(true);
            }
            None => {
                self.date_label.set_text("No date planned");
                self.clear_btn.set_visible(false);
            }
        }

        match &sermon.lectionary {
            Some(link) => {
                self.dot.set_visible(true);
                self.dot
                    .set_css_classes(&["season-dot", styles::season_dot_class(&link.colour_hex)]);
                self.dot
                    .set_tooltip_text(Some(&format!("{} · {}", link.season, link.week)));
            }
            None => self.dot.set_visible(false),
        }
    }
}

fn apply_planned_date(state: &Rc<RefCell<AppState>>, apply: &ApplyFn, new_date: Option<NaiveDate>) {
    let old = {
        let st = state.borrow();
        (st.sermon.planned_date, st.sermon.lectionary.clone())
    };
    let new_link = new_date.map(|d| rcl::get_liturgical_info(d).into());
    let new = (new_date, new_link);
    if old != new {
        apply(Cmd::SetPlannedDate { old, new });
    }
}

fn select_calendar_date(calendar: &Calendar, d: NaiveDate) {
    calendar.set_year(d.year());
    calendar.set_month(d.month0() as i32);
    calendar.set_day(d.day() as i32);
}

fn calendar_date(calendar: &Calendar) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(
        calendar.year(),
        (calendar.month() + 1) as u32,
        calendar.day() as u32,
    )
}
