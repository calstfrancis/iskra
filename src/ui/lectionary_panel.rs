//! Sidebar panel — the lectionary sidebar's first (and so far only) tenant.
//! Shows the season + colour swatch, the week label, and the four RCL
//! readings for the sermon's planned date. Empty state when no date is set.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Popover};
use libadwaita as adw;

use crate::commands::{Cmd, SermonTagKind};
use crate::model::Sermon;
use crate::ui::editor::ApplyFn;
use crate::ui::styles;

pub struct LectionaryPanel {
    pub root: GtkBox,
    empty_status: adw::StatusPage,
    content: GtkBox,
    dot: GtkBox,
    season_label: Label,
    week_label: Label,
    ot_value: Label,
    psalm_value: Label,
    epistle_value: Label,
    gospel_value: Label,
    current_s_tags: RefCell<Vec<String>>,
    apply: RefCell<Option<ApplyFn>>,
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
        let (ot_row, ot_value) = reading_row("OT");
        let (psalm_row, psalm_value) = reading_row("Psalm");
        let (epistle_row, epistle_value) = reading_row("Epistle");
        let (gospel_row, gospel_value) = reading_row("Gospel");
        readings_box.append(&ot_row);
        readings_box.append(&psalm_row);
        readings_box.append(&epistle_row);
        readings_box.append(&gospel_row);
        content.append(&readings_box);

        root.append(&content);

        let panel = Rc::new(Self {
            root,
            empty_status,
            content,
            dot,
            season_label,
            week_label,
            ot_value,
            psalm_value,
            epistle_value,
            gospel_value,
            current_s_tags: RefCell::new(Vec::new()),
            apply: RefCell::new(None),
        });

        for value in [&panel.ot_value, &panel.psalm_value, &panel.epistle_value, &panel.gospel_value] {
            wire_add_scripture_tag(&panel, value);
        }

        panel
    }

    /// Stores `apply` so right-clicking a reading can route the resulting
    /// scripture tag through the single door. Call once, after `apply`
    /// exists — mirrors `StatusBar::init`.
    pub fn init(&self, apply: ApplyFn) {
        *self.apply.borrow_mut() = Some(apply);
    }

    pub fn refresh(&self, sermon: &Sermon) {
        *self.current_s_tags.borrow_mut() = sermon.s_tags.clone();
        match &sermon.lectionary {
            Some(link) => {
                self.empty_status.set_visible(false);
                self.content.set_visible(true);
                self.dot
                    .set_css_classes(&["season-dot", styles::season_dot_class(&link.colour_hex)]);
                self.season_label
                    .set_text(&format!("{} · {}", link.season, link.colour));
                self.week_label.set_text(&link.week);
                self.ot_value.set_text(&link.ot);
                self.psalm_value.set_text(&link.psalm);
                self.epistle_value.set_text(&link.epistle);
                self.gospel_value.set_text(&link.gospel);
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
