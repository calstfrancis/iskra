//! Export dialog: preview of the rendered `.sermon` Typst body with an
//! "include tags" toggle, and a save-to-file action. See `sermon_export.rs`
//! and `Plans/sermon-interchange-spec.md`.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, ScrolledWindow, Separator, Switch, TextView};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::model::Sermon;
use crate::sermon_export;

pub struct ExportDialog {
    window: adw::Window,
}

impl ExportDialog {
    pub fn new(parent: &impl IsA<gtk4::Window>, sermon: Sermon) -> Self {
        let window = adw::Window::builder()
            .title("Export Sermon")
            .transient_for(parent)
            .modal(true)
            .default_width(560)
            .default_height(520)
            .build();

        let header = adw::HeaderBar::new();

        let body = GtkBox::new(Orientation::Vertical, 10);
        body.set_margin_start(16);
        body.set_margin_end(16);
        body.set_margin_top(12);
        body.set_margin_bottom(12);

        let toggle_row = GtkBox::new(Orientation::Horizontal, 8);
        let toggle_label = Label::new(Some("Include idea/part tags"));
        toggle_label.set_hexpand(true);
        toggle_label.set_xalign(0.0);
        let toggle = Switch::new();
        toggle.set_valign(gtk4::Align::Center);
        toggle_row.append(&toggle_label);
        toggle_row.append(&toggle);
        body.append(&toggle_row);

        body.append(&Separator::new(Orientation::Horizontal));

        let preview = TextView::new();
        preview.set_editable(false);
        preview.set_cursor_visible(false);
        preview.set_monospace(true);
        preview.set_top_margin(6);
        preview.set_bottom_margin(6);
        preview.set_left_margin(6);
        preview.set_right_margin(6);
        preview.buffer().set_text(&sermon_export::render_typst_body(&sermon, false));

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_child(Some(&preview));
        body.append(&scroll);

        {
            let sermon = sermon.clone();
            let preview = preview.clone();
            toggle.connect_state_set(move |_, active| {
                preview
                    .buffer()
                    .set_text(&sermon_export::render_typst_body(&sermon, active));
                glib::Propagation::Proceed
            });
        }

        let footer = GtkBox::new(Orientation::Horizontal, 8);
        footer.set_halign(gtk4::Align::End);
        let cancel_btn = Button::with_label("Cancel");
        cancel_btn.add_css_class("flat");
        let export_btn = Button::with_label("Export…");
        export_btn.add_css_class("suggested-action");
        footer.append(&cancel_btn);
        footer.append(&export_btn);
        body.append(&footer);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&body));
        window.set_content(Some(&toolbar));

        {
            let window = window.clone();
            cancel_btn.connect_clicked(move |_| window.close());
        }
        {
            let window = window.clone();
            let toggle = toggle.clone();
            export_btn.connect_clicked(move |_| {
                save_to_file(&window, &sermon, toggle.is_active());
            });
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn save_to_file(window: &adw::Window, sermon: &Sermon, include_tags: bool) {
    let dialog = gtk4::FileDialog::new();
    dialog.set_title("Export Sermon");
    dialog.set_initial_name(Some(&sermon_export::export_filename(sermon)));

    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Sermon interchange (*.sermon)"));
    filter.add_pattern("*.sermon");
    let filters = gtk4::gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));

    let sermon = sermon.clone();
    let parent = window.clone();
    let window = window.clone();
    dialog.save(Some(&parent), gtk4::gio::Cancellable::NONE, move |result| {
        let Some(path) = result.ok().and_then(|f| f.path()) else {
            return;
        };
        let path = if path.extension().is_none() {
            path.with_extension("sermon")
        } else {
            path
        };
        let text = sermon_export::export_sermon(&sermon, include_tags);
        if crate::error::atomic_write(&path, text.as_bytes()).is_ok() {
            window.close();
        }
    });
}
