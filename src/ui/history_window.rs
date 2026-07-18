//! Read-only browser over the open sermon's git backup history (same repo
//! `git_sync.rs` commits to during "Commit & Push"), with a "Restore" action
//! per version. Distinct from undo/redo (in-session, unlimited, lost on
//! close) — this recovers whatever was last committed, potentially days old,
//! and only exists once backup sync has run at least once.

use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SelectionMode};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::git_sync::{self, FileHistoryEntry};

pub struct HistoryWindow {
    window: adw::Window,
}

impl HistoryWindow {
    pub fn new(
        parent: &impl IsA<gtk4::Window>,
        repo_path: PathBuf,
        file_path: PathBuf,
        on_restore: impl Fn(String) + 'static,
    ) -> Self {
        let window = adw::Window::builder()
            .title("Sermon History")
            .transient_for(parent)
            .default_width(460)
            .default_height(540)
            .build();

        let header = adw::HeaderBar::new();

        let history = git_sync::file_history(&repo_path, &file_path);
        let on_restore: Rc<dyn Fn(String)> = Rc::new(on_restore);

        let content: gtk4::Widget = if history.is_empty() {
            let empty_page = adw::StatusPage::new();
            empty_page.set_icon_name(Some("document-open-recent-symbolic"));
            empty_page.set_title("No history yet");
            empty_page.set_description(Some(
                "Versions appear here once this sermon has been committed via Commit & Push.",
            ));
            empty_page.set_vexpand(true);
            empty_page.upcast()
        } else {
            let list = ListBox::new();
            list.set_selection_mode(SelectionMode::None);
            list.add_css_class("boxed-list");

            for entry in &history {
                let row = build_history_row(entry);
                let repo_path = repo_path.clone();
                let file_path = file_path.clone();
                let commit = entry.commit.clone();
                let window = window.clone();
                let on_restore = on_restore.clone();
                row.restore_btn.connect_clicked(move |_| {
                    confirm_restore(&window, repo_path.clone(), file_path.clone(), commit.clone(), on_restore.clone());
                });
                list.append(&row.row);
            }

            let scroll = ScrolledWindow::new();
            scroll.set_vexpand(true);
            scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
            let margin = GtkBox::new(Orientation::Vertical, 0);
            margin.set_margin_top(8);
            margin.set_margin_bottom(8);
            margin.set_margin_start(8);
            margin.set_margin_end(8);
            margin.append(&list);
            scroll.set_child(Some(&margin));
            scroll.upcast()
        };

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&content));
        window.set_content(Some(&toolbar));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

struct HistoryRow {
    row: ListBoxRow,
    restore_btn: Button,
}

fn build_history_row(entry: &FileHistoryEntry) -> HistoryRow {
    let row = ListBoxRow::new();

    let hbox = GtkBox::new(Orientation::Horizontal, 8);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(8);
    hbox.set_margin_end(8);

    let text_col = GtkBox::new(Orientation::Vertical, 2);
    text_col.set_hexpand(true);

    let date_lbl = Label::new(Some(&entry.date.format("%B %-d, %Y · %-I:%M %p").to_string()));
    date_lbl.add_css_class("heading");
    date_lbl.set_xalign(0.0);
    text_col.append(&date_lbl);

    let message = if entry.message.is_empty() {
        "(no commit message)".to_string()
    } else {
        entry.message.clone()
    };
    let message_lbl = Label::new(Some(&message));
    message_lbl.add_css_class("dim-label");
    message_lbl.add_css_class("caption");
    message_lbl.set_xalign(0.0);
    text_col.append(&message_lbl);

    hbox.append(&text_col);

    let restore_btn = Button::with_label("Restore");
    restore_btn.add_css_class("flat");
    restore_btn.set_valign(Align::Center);
    restore_btn.set_tooltip_text(Some("Replace the open sermon with this version"));
    hbox.append(&restore_btn);

    row.set_child(Some(&hbox));
    HistoryRow { row, restore_btn }
}

fn confirm_restore(
    parent: &adw::Window,
    repo_path: PathBuf,
    file_path: PathBuf,
    commit: String,
    on_restore: Rc<dyn Fn(String)>,
) {
    let alert = gtk4::AlertDialog::builder()
        .modal(true)
        .message("Restore this version?")
        .detail("The sermon's current content will be replaced. This can be undone with Ctrl+Z right after restoring.")
        .buttons(["Cancel", "Restore"])
        .cancel_button(0)
        .default_button(0)
        .build();
    let parent_ref = parent.clone();
    let parent = parent.clone();
    alert.choose(Some(&parent_ref), None::<&gtk4::gio::Cancellable>, move |result| {
        if let Ok(1) = result {
            if let Some(content) = git_sync::show_file_at_commit(&repo_path, &file_path, &commit) {
                on_restore(content);
                parent.close();
            }
        }
    });
}
