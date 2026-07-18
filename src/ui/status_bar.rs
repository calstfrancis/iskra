//! Bottom status bar: s.tags / t.tags chips (full editing lands in dev3) and
//! the version indicator on the far right (changelog window lands in dev3).

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Separator};

pub struct StatusBar {
    pub root: GtkBox,
}

impl StatusBar {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Horizontal, 8);
        root.set_margin_top(4);
        root.set_margin_bottom(4);
        root.set_margin_start(10);
        root.set_margin_end(10);

        let tags_label = Label::new(Some("No tags yet"));
        tags_label.add_css_class("dim-label");
        tags_label.add_css_class("caption");
        tags_label.set_hexpand(true);
        tags_label.set_xalign(0.0);
        root.append(&tags_label);

        let sep = Separator::new(Orientation::Vertical);
        sep.add_css_class("statusbar-sep");
        root.append(&sep);

        let version_label = Label::new(Some(&format!("v{}", env!("CARGO_PKG_VERSION"))));
        version_label.add_css_class("dim-label");
        version_label.add_css_class("caption");
        root.append(&version_label);

        Self { root }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
