//! Minimal sermon picker for "Copy movement to another sermon" (movement
//! card header button, see `movement_card.rs`). Deliberately smaller than
//! the full `LibraryWindow` (no tag sidebar, no delete, no new-sermon menu)
//! — its only job is picking a destination file to append a duplicated
//! movement onto. Writes straight to the destination file via `storage`,
//! bypassing the `Cmd`/undo system entirely: that system only ever governs
//! the one currently-open sermon, and a background sermon being written to
//! here has no open editor session to keep in sync.

use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SearchEntry, SelectionMode};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::model::Movement;
use crate::storage;

pub fn present(
    parent: &impl IsA<gtk4::Window>,
    sermons_dir: PathBuf,
    exclude: PathBuf,
    movement: Movement,
    on_copied: impl Fn(String) + 'static,
) {
    let window = adw::Window::builder()
        .title(format!("Copy \"{}\" to…", movement.name))
        .transient_for(parent)
        .default_width(420)
        .default_height(480)
        .build();

    let header = adw::HeaderBar::new();
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search sermons…"));
    header.set_title_widget(Some(&search_entry));

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("boxed-list");

    let empty_page = adw::StatusPage::new();
    empty_page.set_icon_name(Some("folder-symbolic"));
    empty_page.set_title("No other sermons");
    empty_page.set_vexpand(true);
    empty_page.set_visible(false);

    let scroll = ScrolledWindow::new();
    scroll.set_vexpand(true);
    let margin = GtkBox::new(Orientation::Vertical, 0);
    margin.set_margin_top(8);
    margin.set_margin_bottom(8);
    margin.set_margin_start(8);
    margin.set_margin_end(8);
    margin.append(&list);
    scroll.set_child(Some(&margin));

    let content = GtkBox::new(Orientation::Vertical, 0);
    content.append(&scroll);
    content.append(&empty_page);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&content));
    window.set_content(Some(&toolbar));

    let all: Vec<(PathBuf, String)> = storage::scan_sermons(&sermons_dir)
        .into_iter()
        .filter(|(p, _)| p != &exclude)
        .map(|(p, s)| (p, s.display_title().to_string()))
        .collect();

    // Rows currently shown, in list order — `connect_row_activated` below
    // reads this by index rather than baking a per-row click handler into
    // the rebuild loop, so filtering never accumulates duplicate handlers
    // on `list` itself (same idiom as `library_window.rs::rebuild_sermon_list`).
    let shown: Rc<std::cell::RefCell<Vec<(PathBuf, String)>>> = Rc::new(std::cell::RefCell::new(Vec::new()));

    let rebuild = {
        let list = list.clone();
        let empty_page = empty_page.clone();
        let all = all.clone();
        let shown = shown.clone();
        move |query: &str| {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            let query = query.to_lowercase();
            let matches: Vec<(PathBuf, String)> = all
                .iter()
                .filter(|(_, title)| query.is_empty() || title.to_lowercase().contains(&query))
                .cloned()
                .collect();
            empty_page.set_visible(matches.is_empty());
            list.set_visible(!matches.is_empty());
            for (_, title) in &matches {
                let row = ListBoxRow::new();
                let label = Label::new(Some(title));
                label.set_xalign(0.0);
                label.set_margin_top(8);
                label.set_margin_bottom(8);
                label.set_margin_start(10);
                label.set_margin_end(10);
                row.set_child(Some(&label));
                list.append(&row);
            }
            *shown.borrow_mut() = matches;
        }
    };
    rebuild("");

    {
        let rebuild = rebuild.clone();
        search_entry.connect_search_changed(move |e| rebuild(&e.text()));
    }
    {
        let shown = shown.clone();
        let window = window.clone();
        let movement = movement.clone();
        let on_copied = Rc::new(on_copied);
        list.connect_row_activated(move |_, row| {
            let index = row.index();
            if index < 0 {
                return;
            }
            let Some((path, title)) = shown.borrow().get(index as usize).cloned() else {
                return;
            };
            if let Ok(mut dest) = storage::load_sermon(&path) {
                dest.movements.push(movement.duplicate());
                if storage::save_touched(&path, &mut dest).is_ok() {
                    on_copied(title);
                }
            }
            window.close();
        });
    }

    window.present();
    search_entry.grab_focus();
}
