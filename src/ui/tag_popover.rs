//! Free-form tag entry for the idea/part tabs on each idea row. Autocomplete
//! suggestions (sourced from the library tag census) land in dev4; for dev1
//! this is a plain text entry in a popover.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Orientation, Popover};

pub struct TagPopover {
    popover: Popover,
    entry: Entry,
}

impl TagPopover {
    pub fn new(placeholder: &str) -> Self {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(placeholder));
        entry.set_width_chars(16);

        let content = GtkBox::new(Orientation::Vertical, 4);
        content.set_margin_top(6);
        content.set_margin_bottom(6);
        content.set_margin_start(6);
        content.set_margin_end(6);
        content.append(&entry);

        let popover = Popover::new();
        popover.set_child(Some(&content));

        Self { popover, entry }
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

    pub fn popup_on(&self, parent: &impl IsA<gtk4::Widget>) {
        self.popover.set_parent(parent);
        self.entry.set_text(&self.entry.text());
        self.popover.popup();
        self.entry.grab_focus();
    }
}
