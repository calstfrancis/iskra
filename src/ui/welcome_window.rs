//! Shown on first run (as a welcome) and after a version upgrade (as
//! "What's New"), sourcing the version/release name from the same constant
//! used at release time — see the root CLAUDE.md's Release workflow.
//! Adapted from Zerkalo's `welcome_window.rs`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, ScrolledWindow, Separator};
use libadwaita as adw;
use libadwaita::prelude::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASE_NAME: &str = "First Light";

pub struct WelcomeWindow {
    window: adw::Window,
    on_dismissed: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl WelcomeWindow {
    /// True when no marker exists — the very first launch.
    pub fn is_first_run() -> bool {
        !glib::user_data_dir().join("iskra/.welcome_version").exists()
    }

    pub fn new(parent: &impl IsA<gtk4::Window>, is_first_run: bool) -> Self {
        let on_dismissed: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        let title = if is_first_run { "Welcome to Iskra" } else { "What's New" };
        let window = adw::Window::builder()
            .title(title)
            .transient_for(parent)
            .modal(true)
            .default_width(480)
            .default_height(560)
            .build();

        let header = adw::HeaderBar::new();

        let outer = GtkBox::new(Orientation::Vertical, 0);

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);

        let body = GtkBox::new(Orientation::Vertical, 12);
        body.set_margin_start(24);
        body.set_margin_end(24);
        body.set_margin_top(20);
        body.set_margin_bottom(20);

        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(460);
        clamp.set_child(Some(&body));

        let app_title = Label::new(Some("Iskra"));
        app_title.add_css_class("title-1");
        app_title.set_halign(Align::Center);

        let sub_lbl = Label::new(Some(&format!("Version {VERSION} \"{RELEASE_NAME}\"")));
        sub_lbl.add_css_class("dim-label");
        sub_lbl.set_halign(Align::Center);
        sub_lbl.set_margin_bottom(4);

        body.append(&app_title);
        body.append(&sub_lbl);
        body.append(&Separator::new(Orientation::Horizontal));

        if is_first_run {
            body.append(&section_label("How Iskra Works"));
            let intro = Label::new(Some(
                "Iskra plans sermons as single-line ideas, grouped into movements you can \
                 reorder by dragging. Expand any idea for extended notes. Everything autosaves \
                 as you type — there's no manual Save.",
            ));
            intro.set_wrap(true);
            intro.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            intro.set_xalign(0.0);
            intro.set_hexpand(true);
            intro.set_halign(Align::Fill);
            body.append(&intro);

            body.append(&Separator::new(Orientation::Horizontal));
            body.append(&section_label("Getting Started"));
            for item in [
                "Click the title in the header to rename the sermon and pick a planned date",
                "Picking a date resolves the Revised Common Lectionary readings into the sidebar",
                "Type an idea and press + Add idea to add another",
                "Click the triangle beside an idea to expand its notes",
                "Drag the handle on an idea or movement to reorder — or drop into blank space to start a new movement",
                "Use the idea/part tags below each idea to mark structure",
                "Add scripture (s.) and theme (t.) tags in the status bar to make sermons searchable later",
            ] {
                body.append(&bullet_row(item));
            }
        } else {
            body.append(&section_label(&format!("What's New in {VERSION}")));
            for item in [
                "Added: two more lectionaries alongside the RCL — the Roman Catholic Sunday Lectionary and the Narrative Lectionary — plus an RCL Track 2 option, switchable from a picker in the lectionary sidebar",
                "Added: a \"simple\" toggle in the status bar that hides the lectionary picker unless you need it",
                "Added: Escape clears the current idea selection, and so does clicking empty space in the movements column",
                "Fixed: grabbing a drag handle no longer selects rows instead of starting a drag — reordering ideas and movements is much more reliable",
                "Fixed: drag handles are a larger target with a grab cursor, and the drag preview now sits under the pointer so the drop indicator lines up",
            ] {
                body.append(&bullet_row(item));
            }
        }

        body.append(&Separator::new(Orientation::Horizontal));
        body.append(&section_label("Keyboard Shortcuts"));
        for (key, desc) in [
            ("Ctrl+Z", "Undo"),
            ("Ctrl+Shift+Z", "Redo"),
            ("Ctrl+K", "Command palette"),
            ("Ctrl+L", "Open library"),
            ("Ctrl+E", "Export…"),
            ("Ctrl+Shift+P", "Preaching View"),
            ("Ctrl+Shift+H", "History…"),
            ("Ctrl+Shift+G", "Commit & push"),
        ] {
            body.append(&shortcut_row(key, desc));
        }

        scroll.set_child(Some(&clamp));
        outer.append(&scroll);
        outer.append(&Separator::new(Orientation::Horizontal));

        let footer = GtkBox::new(Orientation::Horizontal, 0);
        footer.set_margin_start(16);
        footer.set_margin_end(16);
        footer.set_margin_top(8);
        footer.set_margin_bottom(12);
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        footer.append(&spacer);
        let btn_label = if is_first_run { "Get Started" } else { "Close" };
        let ok_btn = Button::with_label(btn_label);
        ok_btn.add_css_class("suggested-action");
        ok_btn.add_css_class("pill");
        footer.append(&ok_btn);
        outer.append(&footer);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&outer));
        window.set_content(Some(&toolbar_view));

        let win_c = window.clone();
        let cb = on_dismissed.clone();
        ok_btn.connect_clicked(move |_| {
            win_c.close();
            if let Some(f) = cb.borrow().as_ref() {
                f();
            }
        });

        Self { window, on_dismissed }
    }

    /// Called after "Get Started"/"Close" is clicked (after the window closes).
    pub fn set_on_dismissed(&self, f: impl Fn() + 'static) {
        *self.on_dismissed.borrow_mut() = Some(Box::new(f));
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Returns true when the welcome window should be shown (new install or version upgrade).
    pub fn should_show() -> bool {
        let marker = glib::user_data_dir().join("iskra/.welcome_version");
        std::fs::read_to_string(&marker)
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
            != VERSION
    }

    /// Record that the welcome window has been shown for this version.
    pub fn mark_shown() {
        let marker = glib::user_data_dir().join("iskra/.welcome_version");
        if let Some(parent) = marker.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(marker, VERSION);
    }
}

fn section_label(text: &str) -> Label {
    let lbl = Label::new(Some(text));
    lbl.set_xalign(0.0);
    lbl.add_css_class("heading");
    lbl
}

fn bullet_row(text: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.set_margin_start(4);
    row.set_hexpand(true);
    let dot = Label::new(Some("•"));
    dot.set_valign(Align::Start);
    dot.add_css_class("dim-label");
    let lbl = Label::new(Some(text));
    lbl.set_xalign(0.0);
    lbl.set_wrap(true);
    lbl.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    lbl.set_hexpand(true);
    lbl.set_halign(Align::Fill);
    row.append(&dot);
    row.append(&lbl);
    row
}

fn shortcut_row(key: &str, desc: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.set_margin_start(4);
    row.set_hexpand(true);
    let key_lbl = Label::new(Some(key));
    key_lbl.set_width_chars(16);
    key_lbl.set_xalign(0.0);
    key_lbl.add_css_class("monospace");
    let desc_lbl = Label::new(Some(desc));
    desc_lbl.set_xalign(0.0);
    desc_lbl.set_hexpand(true);
    desc_lbl.set_halign(Align::Fill);
    desc_lbl.set_wrap(true);
    desc_lbl.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    desc_lbl.add_css_class("dim-label");
    row.append(&key_lbl);
    row.append(&desc_lbl);
    row
}
