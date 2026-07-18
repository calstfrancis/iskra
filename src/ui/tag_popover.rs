//! Free-form tag entry for the idea/part tabs on each idea row, with
//! autocomplete suggestions sourced from the library's tag census (every
//! idea/part tag value used anywhere in `work_dir/sermons/`, not just the
//! open sermon) — see `library::LibraryIndex::idea_tag_census`/`part_tag_census`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, Orientation, Popover};

pub struct TagPopover {
    popover: Popover,
    entry: Entry,
    suggestions_box: GtkBox,
    census: RefCell<Vec<String>>,
}

impl TagPopover {
    pub fn new(placeholder: &str) -> Rc<Self> {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(placeholder));
        entry.set_width_chars(16);

        let suggestions_box = GtkBox::new(Orientation::Vertical, 0);
        suggestions_box.set_visible(false);

        let content = GtkBox::new(Orientation::Vertical, 4);
        content.set_margin_top(6);
        content.set_margin_bottom(6);
        content.set_margin_start(6);
        content.set_margin_end(6);
        content.append(&entry);
        content.append(&suggestions_box);

        let popover = Popover::new();
        popover.set_child(Some(&content));

        let this = Rc::new(Self {
            popover,
            entry: entry.clone(),
            suggestions_box,
            census: RefCell::new(Vec::new()),
        });

        {
            let this = this.clone();
            entry.connect_changed(move |_| this.refresh_suggestions());
        }

        this
    }

    pub fn popover(&self) -> &Popover {
        &self.popover
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn set_text(&self, text: &str) {
        self.entry.set_text(text);
    }

    /// Sets the full set of known values for this tag kind (library-wide
    /// census). Called every time the library index is rebuilt.
    pub fn set_census(self: &Rc<Self>, values: Vec<String>) {
        *self.census.borrow_mut() = values;
        self.refresh_suggestions();
    }

    fn refresh_suggestions(self: &Rc<Self>) {
        while let Some(child) = self.suggestions_box.first_child() {
            self.suggestions_box.remove(&child);
        }

        let query = self.entry.text().to_lowercase();
        let matches: Vec<String> = self
            .census
            .borrow()
            .iter()
            .filter(|v| !query.is_empty() && v.to_lowercase().contains(&query) && v.to_lowercase() != query)
            .take(6)
            .cloned()
            .collect();

        for value in &matches {
            let btn = Button::with_label(value);
            btn.add_css_class("flat");
            btn.set_halign(gtk4::Align::Fill);
            if let Some(label) = btn.child().and_downcast_ref::<gtk4::Label>() {
                label.set_xalign(0.0);
            }
            let this = self.clone();
            let value = value.clone();
            btn.connect_clicked(move |_| {
                this.entry.set_text(&value);
                this.popover.popdown();
            });
            self.suggestions_box.append(&btn);
        }
        self.suggestions_box.set_visible(!matches.is_empty());
    }

    pub fn popup_on(&self, parent: &impl IsA<gtk4::Widget>) {
        self.popover.set_parent(parent);
        self.entry.set_text(&self.entry.text());
        self.popover.popup();
        self.entry.grab_focus();
    }
}
