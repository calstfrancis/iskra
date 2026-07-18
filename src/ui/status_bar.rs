//! Bottom status bar: sermon-level tag chips (scripture "s." tags and theme
//! "t." tags — distinct from the per-idea idea/part tags on each idea row,
//! see Plans/iskra-kickoff-prompt.md §4.6) and the version indicator on the
//! far right, which opens the changelog window.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Label, MenuButton, Orientation, Popover, Separator};

use crate::commands::{Cmd, SermonTagKind};
use crate::model::Sermon;
use crate::ui::editor::ApplyFn;

pub struct StatusBar {
    pub root: GtkBox,
    pub version_btn: Button,
    s_tags_box: GtkBox,
    t_tags_box: GtkBox,
    apply: RefCell<Option<ApplyFn>>,
}

impl StatusBar {
    pub fn new() -> Rc<Self> {
        let root = GtkBox::new(Orientation::Horizontal, 8);
        root.set_margin_top(4);
        root.set_margin_bottom(4);
        root.set_margin_start(10);
        root.set_margin_end(10);

        let s_group = Label::new(Some("s."));
        s_group.add_css_class("dim-label");
        s_group.add_css_class("caption");
        s_group.set_tooltip_text(Some("Scripture tags"));
        root.append(&s_group);

        let s_tags_box = GtkBox::new(Orientation::Horizontal, 4);
        root.append(&s_tags_box);

        root.append(&Separator::new(Orientation::Vertical));

        let t_group = Label::new(Some("t."));
        t_group.add_css_class("dim-label");
        t_group.add_css_class("caption");
        t_group.set_tooltip_text(Some("Theme tags"));
        root.append(&t_group);

        let t_tags_box = GtkBox::new(Orientation::Horizontal, 4);
        root.append(&t_tags_box);

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        root.append(&spacer);

        let sep = Separator::new(Orientation::Vertical);
        sep.add_css_class("statusbar-sep");
        root.append(&sep);

        let version_btn = Button::with_label(&format!("v{}", env!("CARGO_PKG_VERSION")));
        version_btn.add_css_class("flat");
        version_btn.add_css_class("dim-label");
        version_btn.add_css_class("caption");
        version_btn.set_tooltip_text(Some("View changelog"));
        root.append(&version_btn);

        Rc::new(Self {
            root,
            version_btn,
            s_tags_box,
            t_tags_box,
            apply: RefCell::new(None),
        })
    }

    /// Stores `apply` so tag chips built by `refresh` can route add/remove
    /// through the single door. Call once, after `apply` exists.
    pub fn init(&self, apply: ApplyFn) {
        *self.apply.borrow_mut() = Some(apply);
    }

    pub fn refresh(&self, sermon: &Sermon) {
        let apply = self.apply.borrow().clone();
        let Some(apply) = apply else { return };
        rebuild_tag_group(&self.s_tags_box, &sermon.s_tags, SermonTagKind::S, &apply);
        rebuild_tag_group(&self.t_tags_box, &sermon.t_tags, SermonTagKind::T, &apply);
    }
}

fn rebuild_tag_group(container: &GtkBox, tags: &[String], kind: SermonTagKind, apply: &ApplyFn) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    for tag in tags {
        let chip = GtkBox::new(Orientation::Horizontal, 2);
        chip.add_css_class("tag-chip");

        let label = Label::new(Some(tag));
        chip.append(&label);

        let remove_btn = Button::from_icon_name("window-close-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.set_valign(gtk4::Align::Center);
        {
            let old: Vec<String> = tags.to_vec();
            let tag = tag.clone();
            let apply = apply.clone();
            remove_btn.connect_clicked(move |_| {
                let mut new = old.clone();
                new.retain(|t| t != &tag);
                apply(Cmd::SetSermonTags {
                    kind,
                    old: old.clone(),
                    new,
                });
            });
        }
        chip.append(&remove_btn);

        container.append(&chip);
    }

    let add_btn = MenuButton::new();
    add_btn.set_icon_name("list-add-symbolic");
    add_btn.add_css_class("flat");
    add_btn.set_tooltip_text(Some("Add tag"));

    let entry = Entry::new();
    entry.set_placeholder_text(Some("New tag…"));
    entry.set_width_chars(14);
    let popover = Popover::new();
    popover.set_child(Some(&entry));
    add_btn.set_popover(Some(&popover));

    {
        let old: Vec<String> = tags.to_vec();
        let apply = apply.clone();
        let popover = popover.clone();
        entry.connect_activate(move |e| {
            let text = e.text().trim().to_string();
            if !text.is_empty() && !old.contains(&text) {
                let mut new = old.clone();
                new.push(text);
                apply(Cmd::SetSermonTags {
                    kind,
                    old: old.clone(),
                    new,
                });
            }
            e.set_text("");
            popover.popdown();
        });
    }

    container.append(&add_btn);
}
