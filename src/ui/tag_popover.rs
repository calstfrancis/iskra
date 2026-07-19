//! Free-form tag entry for the idea/part tabs on each idea row. Shows a row
//! of quick-pick chips for the library's most frequently used values as soon
//! as the popover opens, plus filtered autocomplete suggestions once you
//! start typing — both sourced from the library's tag census (every
//! idea/part tag value used anywhere in `work_dir/sermons/`, not just the
//! open sermon) — see `library::LibraryIndex::idea_tag_census`/`part_tag_census`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, FlowBox, Orientation, Popover, SelectionMode};

const MAX_QUICK_PICKS: usize = 6;

pub struct TagPopover {
    popover: Popover,
    entry: Entry,
    quick_picks_box: FlowBox,
    suggestions_box: GtkBox,
    rename_everywhere_btn: Button,
    census: RefCell<Vec<(String, usize)>>,
    /// The tag value as it was when this popover last opened — captured in
    /// `popup_on`, distinct from `entry`'s live (possibly mid-edit) text, so
    /// "rename everywhere" knows what old value to match against even after
    /// the entry itself has already been retyped to the new one.
    original_text: RefCell<String>,
    on_rename_everywhere: RefCell<Option<Box<dyn Fn(String, String)>>>,
}

impl TagPopover {
    pub fn new(placeholder: &str) -> Rc<Self> {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(placeholder));
        entry.set_width_chars(16);

        let quick_picks_box = FlowBox::new();
        quick_picks_box.set_selection_mode(SelectionMode::None);
        quick_picks_box.set_max_children_per_line(3);
        quick_picks_box.set_row_spacing(3);
        quick_picks_box.set_column_spacing(3);
        quick_picks_box.set_visible(false);

        let suggestions_box = GtkBox::new(Orientation::Vertical, 0);
        suggestions_box.set_visible(false);

        let rename_everywhere_btn = Button::new();
        rename_everywhere_btn.add_css_class("flat");
        rename_everywhere_btn.set_halign(gtk4::Align::Fill);
        if let Some(label) = rename_everywhere_btn.child().and_downcast_ref::<gtk4::Label>() {
            label.set_xalign(0.0);
            label.set_wrap(true);
        }
        rename_everywhere_btn.set_visible(false);

        let content = GtkBox::new(Orientation::Vertical, 4);
        content.set_margin_top(6);
        content.set_margin_bottom(6);
        content.set_margin_start(6);
        content.set_margin_end(6);
        content.append(&entry);
        content.append(&quick_picks_box);
        content.append(&suggestions_box);
        content.append(&rename_everywhere_btn);

        let popover = Popover::new();
        popover.set_child(Some(&content));

        let this = Rc::new(Self {
            popover,
            entry: entry.clone(),
            quick_picks_box,
            suggestions_box,
            rename_everywhere_btn,
            census: RefCell::new(Vec::new()),
            original_text: RefCell::new(String::new()),
            on_rename_everywhere: RefCell::new(None),
        });

        {
            let this = this.clone();
            entry.connect_changed(move |_| this.refresh());
        }
        {
            // Fires however the popover was actually shown — the `MenuButton`
            // it's attached to opens it internally via `set_popover`, so this
            // module never gets a call through `popup_on` (dead code below,
            // pre-existing) to hook the "just opened" moment itself.
            let this = this.clone();
            let popover = this.popover.clone();
            popover.connect_show(move |_| {
                let current = this.entry.text().to_string();
                *this.original_text.borrow_mut() = current.clone();
                this.rename_everywhere_btn.set_visible(!current.is_empty());
                this.rename_everywhere_btn
                    .set_label(&format!("Rename \"{current}\" everywhere…"));
            });
        }
        {
            let this = this.clone();
            let rename_everywhere_btn = this.rename_everywhere_btn.clone();
            rename_everywhere_btn.connect_clicked(move |_| {
                let old = this.original_text.borrow().clone();
                let new = this.entry.text().to_string();
                if let Some(f) = this.on_rename_everywhere.borrow().as_ref() {
                    f(old, new);
                }
                this.popover.popdown();
            });
        }

        this
    }

    /// Called once the tag ids/callbacks it needs are wired — see the
    /// `idea_row.rs` call sites (mirrors `refresh_tag_button`'s caller-owns-
    /// the-wiring convention elsewhere in this file).
    pub fn set_on_rename_everywhere(&self, f: impl Fn(String, String) + 'static) {
        *self.on_rename_everywhere.borrow_mut() = Some(Box::new(f));
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

    /// Sets the full library-wide census for this tag kind, as
    /// (value, occurrence count) pairs. Called every time the library index
    /// is rebuilt.
    pub fn set_census(self: &Rc<Self>, values: Vec<(String, usize)>) {
        *self.census.borrow_mut() = values;
        self.refresh();
    }

    fn refresh(self: &Rc<Self>) {
        self.refresh_quick_picks();
        self.refresh_suggestions();
    }

    /// The library's most-used values for this tag kind, immediately
    /// clickable — visible only while the entry is empty, since once
    /// there's a query the filtered suggestion list below takes over.
    fn refresh_quick_picks(self: &Rc<Self>) {
        while let Some(child) = self.quick_picks_box.first_child() {
            self.quick_picks_box.remove(&child);
        }

        if !self.entry.text().is_empty() {
            self.quick_picks_box.set_visible(false);
            return;
        }

        let mut sorted = self.census.borrow().clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        for (value, _) in sorted.iter().take(MAX_QUICK_PICKS) {
            let btn = Button::with_label(value);
            btn.add_css_class("flat");
            btn.add_css_class("tag-quick-pick");
            let this = self.clone();
            let value = value.clone();
            btn.connect_clicked(move |_| {
                this.entry.set_text(&value);
                this.popover.popdown();
            });
            self.quick_picks_box.insert(&btn, -1);
        }
        self.quick_picks_box.set_visible(!sorted.is_empty());
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
            .map(|(v, _)| v)
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
