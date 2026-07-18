//! Backup setup dialog: git identity + GitHub repository linking. Ported
//! from Zerkalo's `setup_wizard.rs` `git_identity_group`/`github_repo_group`
//! — Iskra only needs those two sections (no fonts/tools/multi-remote-backup
//! sections, which are Zerkalo-specific), so this is a small standalone
//! dialog rather than a full wizard. `git2` handles local repo
//! discovery/init/identity; actual sync (add/commit/push) goes through
//! `git_sync.rs`, which shells out to the `git` CLI.

use std::path::Path;

use gtk4::prelude::*;
use gtk4::{
    AlertDialog, Align, Box as GtkBox, Button, Image, Label, LinkButton, Orientation,
    ScrolledWindow, Switch,
};
use libadwaita as adw;
use libadwaita::prelude::*;

use super::github_signin;

pub struct BackupSetup {
    window: adw::Window,
}

impl BackupSetup {
    pub fn new(parent: &impl IsA<gtk4::Window>, work_dir: &Path) -> Self {
        let window = adw::Window::builder()
            .title("Set Up Backup")
            .transient_for(parent)
            .modal(true)
            .default_width(560)
            .default_height(520)
            .build();

        let header = adw::HeaderBar::new();

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);

        let body = GtkBox::new(Orientation::Vertical, 20);
        body.set_margin_start(16);
        body.set_margin_end(16);
        body.set_margin_top(16);
        body.set_margin_bottom(16);

        let intro = Label::new(Some(
            "Back up your sermons to GitHub so they're safe and available on any machine.",
        ));
        intro.set_wrap(true);
        intro.set_xalign(0.0);
        intro.add_css_class("dim-label");
        body.append(&intro);

        let (git_group, _git_complete) = git_identity_group();
        body.append(&git_group);

        let (repo_group, _repo_complete) = github_repo_group(&window, work_dir);
        body.append(&repo_group);

        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(560);
        clamp.set_child(Some(&body));
        scroll.set_child(Some(&clamp));

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&scroll));
        window.set_content(Some(&toolbar_view));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn window(&self) -> &adw::Window {
        &self.window
    }
}

fn group_header_suffix(icon_name: &str) -> Image {
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("dim-label");
    icon
}

fn git_identity_group() -> (adw::PreferencesGroup, bool) {
    let group = adw::PreferencesGroup::new();
    group.set_title("Git Identity");
    group.set_description(Some(
        "Git records your name and email on every save. Set these once, globally.",
    ));
    group.set_header_suffix(Some(&group_header_suffix("avatar-default-symbolic")));

    let (current_name, current_email) = git_identity();
    let complete = !current_name.is_empty() && !current_email.is_empty();

    let name_row = adw::EntryRow::new();
    name_row.set_title("Name");
    name_row.set_text(&current_name);

    let email_row = adw::EntryRow::new();
    email_row.set_title("Email");
    email_row.set_text(&current_email);

    let status_lbl = Label::new(None);
    status_lbl.set_xalign(0.0);
    status_lbl.set_margin_top(4);
    status_lbl.set_wrap(true);
    if complete {
        status_lbl.set_label("✓ Git identity is set.");
        status_lbl.add_css_class("success");
    } else {
        status_lbl.set_label("Enter your name and email, then click Apply.");
        status_lbl.add_css_class("dim-label");
    }

    let apply_btn = Button::with_label("Apply");
    apply_btn.set_halign(Align::End);
    apply_btn.add_css_class("suggested-action");

    {
        let name_c = name_row.clone();
        let email_c = email_row.clone();
        let lbl_c = status_lbl.clone();
        apply_btn.connect_clicked(move |_| {
            let name = name_c.text().to_string();
            let email = email_c.text().to_string();
            match set_git_identity(&name, &email) {
                Ok(()) => {
                    lbl_c.set_label("✓ Git identity saved.");
                    lbl_c.remove_css_class("error");
                    lbl_c.add_css_class("success");
                }
                Err(e) => {
                    lbl_c.set_label(&format!("Error: {e}"));
                    lbl_c.remove_css_class("success");
                    lbl_c.add_css_class("error");
                }
            }
        });
    }

    group.add(&name_row);
    group.add(&email_row);

    let suffix_box = GtkBox::new(Orientation::Vertical, 6);
    suffix_box.set_margin_top(8);
    suffix_box.set_margin_bottom(4);
    suffix_box.append(&status_lbl);
    suffix_box.append(&apply_btn);

    let wrapper = adw::ActionRow::new();
    wrapper.set_activatable(false);
    wrapper.add_suffix(&suffix_box);
    group.add(&wrapper);

    (group, complete)
}

fn github_repo_group(parent: &adw::Window, work_dir: &Path) -> (adw::PreferencesGroup, bool) {
    let work_dir = work_dir.to_path_buf();

    let group = adw::PreferencesGroup::new();
    group.set_title("GitHub Repository");
    group.set_description(Some(
        "Back up your sermons by connecting this folder to a GitHub repository.",
    ));
    group.set_header_suffix(Some(&group_header_suffix("network-server-symbolic")));

    let is_repo = git2::Repository::discover(&work_dir).is_ok();
    let remote_url = get_git_remote(&work_dir);
    let complete = remote_url.is_some();

    let repo_row = adw::ActionRow::new();
    repo_row.set_title("Local repository");
    if is_repo {
        repo_row.set_subtitle("✓ Git repository found in the sermons folder");
    } else {
        repo_row.set_subtitle("No git repository — click to initialise one");
        let init_btn = Button::with_label("git init");
        init_btn.set_valign(Align::Center);
        init_btn.add_css_class("suggested-action");
        let work_dir_c = work_dir.clone();
        let row_c = repo_row.clone();
        init_btn.connect_clicked(move |btn| match git2::Repository::init(&work_dir_c) {
            Ok(_) => {
                row_c.set_subtitle("✓ Git repository initialised");
                btn.set_sensitive(false);
            }
            Err(e) => {
                row_c.set_subtitle(&format!("Error: {e}"));
            }
        });
        repo_row.add_suffix(&init_btn);
    }
    group.add(&repo_row);

    let account_row = adw::ActionRow::new();
    account_row.set_title("GitHub Account");
    let has_token = crate::secret_store::load_github_token().is_some();
    account_row.set_subtitle(if has_token { "Connected" } else { "Not connected" });

    let signup_link = LinkButton::with_label(
        "https://github.com/signup",
        "Don't have an account? Create one (free) ↗",
    );
    signup_link.add_css_class("flat");
    signup_link.add_css_class("caption");

    let signin_btn = Button::with_label(if has_token { "Reconnect" } else { "Sign in with GitHub" });
    signin_btn.set_valign(Align::Center);
    signin_btn.add_css_class("suggested-action");

    let create_row = adw::EntryRow::new();
    create_row.set_title("New repository name");
    create_row.set_text("iskra-sermons");

    let private_switch = Switch::new();
    private_switch.set_active(true);
    private_switch.set_valign(Align::Center);
    let private_label = Label::new(Some("Private"));

    let create_status_lbl = Label::new(None);
    create_status_lbl.set_xalign(0.0);
    create_status_lbl.set_margin_top(4);
    create_status_lbl.set_wrap(true);
    create_status_lbl.add_css_class("dim-label");
    create_status_lbl.set_label(if has_token {
        "Creates a repository on your GitHub account and links it here."
    } else {
        "Sign in with GitHub above, then create a repository here."
    });

    let create_btn = Button::with_label("Create & Link");
    create_btn.set_halign(Align::End);
    create_btn.add_css_class("suggested-action");

    let remote_entry = adw::EntryRow::new();
    remote_entry.set_title("Remote URL (GitHub)");
    if let Some(ref url) = remote_url {
        remote_entry.set_text(url);
    }

    let status_lbl = Label::new(None);
    status_lbl.set_xalign(0.0);
    status_lbl.set_margin_top(4);
    status_lbl.set_wrap(true);
    match &remote_url {
        Some(url) => {
            status_lbl.set_label(&format!("✓ Remote: {url}"));
            status_lbl.add_css_class("success");
        }
        None => {
            status_lbl.set_label("Paste the URL of a repository you already created on GitHub.");
            status_lbl.add_css_class("dim-label");
        }
    }

    let apply_btn = Button::with_label("Apply");
    apply_btn.set_halign(Align::End);
    apply_btn.add_css_class("suggested-action");

    {
        let parent = parent.clone();
        let row_c = account_row.clone();
        let signin_btn_c = signin_btn.clone();
        let create_row_c = create_row.clone();
        let create_status_c = create_status_lbl.clone();
        signin_btn.connect_clicked(move |_| {
            let row_c2 = row_c.clone();
            let signin_btn_c2 = signin_btn_c.clone();
            let create_row_c2 = create_row_c.clone();
            let create_status_c2 = create_status_c.clone();
            github_signin::present(&parent, move |username| {
                row_c2.set_subtitle(&format!("Connected as {username}"));
                signin_btn_c2.set_label("Reconnect");
                create_status_c2.set_label("Connected! Pick a name below and click Create & Link to finish.");
                create_status_c2.remove_css_class("dim-label");
                create_status_c2.add_css_class("success");
                create_row_c2.grab_focus();
            });
        });
    }

    {
        let name_c = create_row.clone();
        let private_c = private_switch.clone();
        let status_c = create_status_lbl.clone();
        let wdir = work_dir.clone();
        let remote_entry_c = remote_entry.clone();
        let remote_status_c = status_lbl.clone();
        let win_for_create = parent.clone();
        create_btn.connect_clicked(move |btn| {
            let Some(token) = crate::secret_store::load_github_token() else {
                status_c.set_label("Sign in with GitHub first.");
                status_c.remove_css_class("success");
                status_c.add_css_class("error");
                return;
            };
            let name = name_c.text().trim().to_string();
            if name.is_empty() {
                status_c.set_label("Enter a repository name.");
                return;
            }
            let private = private_c.is_active();

            let go = {
                let btn = btn.clone();
                let status_c = status_c.clone();
                let wdir = wdir.clone();
                let remote_entry_c = remote_entry_c.clone();
                let remote_status_c = remote_status_c.clone();
                move || {
                    btn.set_sensitive(false);
                    status_c.remove_css_class("error");
                    status_c.set_label("Creating repository…");

                    let (tx, rx) = std::sync::mpsc::sync_channel(1);
                    std::thread::spawn(move || {
                        let _ = tx.send(crate::github_auth::create_repo(&token, &name, private));
                    });

                    let btn = btn.clone();
                    let wdir = wdir.clone();
                    let status_c = status_c.clone();
                    let remote_entry_c = remote_entry_c.clone();
                    let remote_status_c = remote_status_c.clone();
                    glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                        match rx.try_recv() {
                            Ok(result) => {
                                match result {
                                    Ok(clone_url) => match set_git_remote(&wdir, &clone_url) {
                                        Ok(()) => {
                                            status_c.set_label(&format!("✓ Created and linked: {clone_url}"));
                                            status_c.add_css_class("success");
                                            remote_entry_c.set_text(&clone_url);
                                            remote_status_c.set_label(&format!("✓ Remote: {clone_url}"));
                                            remote_status_c.remove_css_class("dim-label");
                                            remote_status_c.add_css_class("success");
                                        }
                                        Err(e) => {
                                            status_c.set_label(&format!(
                                                "Repository created, but linking failed: {e}"
                                            ));
                                            status_c.add_css_class("error");
                                        }
                                    },
                                    Err(e) => {
                                        status_c.set_label(&format!("Error: {e}"));
                                        status_c.add_css_class("error");
                                    }
                                }
                                btn.set_sensitive(true);
                                glib::ControlFlow::Break
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                status_c.set_label("Error: repository creation task ended unexpectedly.");
                                status_c.add_css_class("error");
                                btn.set_sensitive(true);
                                glib::ControlFlow::Break
                            }
                        }
                    });
                }
            };

            if has_git_remote(&wdir) {
                let existing = get_git_remote(&wdir).unwrap_or_default();
                let confirm = AlertDialog::builder()
                    .modal(true)
                    .message("Replace the existing remote?")
                    .detail(format!(
                        "Iskra's sermons folder is already linked to:\n{existing}\n\nCreating a new repository will replace this link. The old repository on GitHub is not deleted."
                    ))
                    .buttons(["Cancel", "Create New Repository"])
                    .cancel_button(0)
                    .default_button(0)
                    .build();
                confirm.choose(Some(&win_for_create), None::<&gtk4::gio::Cancellable>, move |result| {
                    if let Ok(1) = result {
                        go();
                    }
                });
                return;
            }
            go();
        });
    }

    {
        let entry_c = remote_entry.clone();
        let lbl_c = status_lbl.clone();
        let wdir = work_dir.clone();
        apply_btn.connect_clicked(move |_| {
            let url = entry_c.text().to_string();
            if url.is_empty() {
                lbl_c.set_label("Please enter a repository URL.");
                return;
            }
            match set_git_remote(&wdir, &url) {
                Ok(()) => {
                    lbl_c.set_label(&format!("✓ Remote set: {url}"));
                    lbl_c.remove_css_class("error");
                    lbl_c.add_css_class("success");
                }
                Err(e) => {
                    lbl_c.set_label(&format!("Error: {e}"));
                    lbl_c.remove_css_class("success");
                    lbl_c.add_css_class("error");
                }
            }
        });
    }

    let account_suffix = GtkBox::new(Orientation::Vertical, 4);
    account_suffix.set_halign(Align::End);
    account_suffix.append(&signin_btn);
    account_suffix.append(&signup_link);
    account_row.add_suffix(&account_suffix);
    group.add(&account_row);

    group.add(&create_row);

    let private_box = GtkBox::new(Orientation::Horizontal, 6);
    private_box.set_halign(Align::End);
    private_box.append(&private_label);
    private_box.append(&private_switch);

    let create_suffix = GtkBox::new(Orientation::Vertical, 6);
    create_suffix.set_margin_top(8);
    create_suffix.set_margin_bottom(4);
    create_suffix.append(&create_status_lbl);
    let create_btn_row = GtkBox::new(Orientation::Horizontal, 8);
    create_btn_row.set_halign(Align::End);
    create_btn_row.append(&private_box);
    create_btn_row.append(&create_btn);
    create_suffix.append(&create_btn_row);

    let create_wrapper = adw::ActionRow::new();
    create_wrapper.set_activatable(false);
    create_wrapper.add_suffix(&create_suffix);
    group.add(&create_wrapper);

    let fallback_expander = adw::ExpanderRow::new();
    fallback_expander.set_title("Already have a repository?");
    fallback_expander.set_subtitle("Paste its URL instead of creating a new one");
    fallback_expander.add_row(&remote_entry);

    let fallback_suffix_box = GtkBox::new(Orientation::Vertical, 6);
    fallback_suffix_box.set_margin_top(8);
    fallback_suffix_box.set_margin_bottom(4);
    fallback_suffix_box.set_margin_start(12);
    fallback_suffix_box.set_margin_end(12);
    fallback_suffix_box.append(&status_lbl);
    let fallback_btn_row = GtkBox::new(Orientation::Horizontal, 8);
    fallback_btn_row.set_halign(Align::End);
    fallback_btn_row.append(&apply_btn);
    fallback_suffix_box.append(&fallback_btn_row);
    let fallback_wrapper = adw::ActionRow::new();
    fallback_wrapper.set_activatable(false);
    fallback_wrapper.add_suffix(&fallback_suffix_box);
    fallback_expander.add_row(&fallback_wrapper);

    group.add(&fallback_expander);

    (group, complete)
}

fn git_identity() -> (String, String) {
    let cfg = git2::Config::open_default().ok();
    let name = cfg
        .as_ref()
        .and_then(|c| c.get_string("user.name").ok())
        .unwrap_or_default();
    let email = cfg
        .as_ref()
        .and_then(|c| c.get_string("user.email").ok())
        .unwrap_or_default();
    (name, email)
}

fn set_git_identity(name: &str, email: &str) -> Result<(), String> {
    let mut cfg = git2::Config::open_default().map_err(|e| e.message().to_string())?;
    cfg.set_str("user.name", name).map_err(|e| e.message().to_string())?;
    cfg.set_str("user.email", email).map_err(|e| e.message().to_string())?;
    Ok(())
}

fn has_git_remote(work_dir: &Path) -> bool {
    get_git_remote(work_dir).is_some()
}

fn get_git_remote(work_dir: &Path) -> Option<String> {
    let repo = git2::Repository::discover(work_dir).ok()?;
    let remotes = repo.remotes().ok()?;
    let name = remotes.get(0)?;
    let remote = repo.find_remote(name).ok()?;
    remote.url().map(|s| s.to_string())
}

fn set_git_remote(work_dir: &Path, url: &str) -> Result<(), String> {
    let repo = git2::Repository::discover(work_dir).map_err(|e| e.message().to_string())?;
    let _ = repo.remote_delete("origin");
    repo.remote("origin", url).map_err(|e| e.message().to_string())?;
    Ok(())
}
