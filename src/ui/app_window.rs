use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, MenuButton, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::commands::{Cmd, SermonTagKind};
use crate::config::Config;
use crate::git_sync;
use crate::library::LibraryIndex;
use crate::model::Sermon;
use crate::state::AppState;
use crate::storage;
use crate::ui::backup_setup::BackupSetup;
use crate::ui::changelog_window::show_changelog;
use crate::ui::command_palette::{default_commands, outline_items, CommandPalette};
use crate::ui::editor::{ApplyFn, Editor};
use crate::ui::export_dialog::ExportDialog;
use crate::ui::history_window::HistoryWindow;
use crate::ui::lectionary_panel::LectionaryPanel;
use crate::ui::library_window::LibraryWindow;
use crate::ui::preaching_view::PreachingView;
use crate::ui::status_bar::StatusBar;
use crate::ui::styles;
use crate::ui::title_date_popover::TitleDatePopover;
use crate::ui::welcome_window::WelcomeWindow;

pub struct AppWindow {
    window: adw::ApplicationWindow,
}

impl AppWindow {
    pub fn new(app: &adw::Application, config: Config) -> Self {
        styles::load_global_css();

        let sermons_dir = config.sermons_dir();
        std::fs::create_dir_all(&sermons_dir).ok();

        let (sermon, path) = load_or_create_sermon(&config);

        let window = adw::ApplicationWindow::new(app);
        window.set_title(Some("Iskra"));
        window.set_default_width(config.window_width);
        window.set_default_height(config.window_height);
        if config.window_maximized {
            window.maximize();
        }

        let state = Rc::new(RefCell::new(AppState::new(sermon, path, config)));

        // ── Header bar ───────────────────────────────────────────────────
        let header = adw::HeaderBar::new();

        let library_btn = Button::with_label("Library");
        library_btn.add_css_class("flat");
        library_btn.set_tooltip_text(Some("Open the sermon library (Ctrl+L)"));
        header.pack_start(&library_btn);

        let title_date = TitleDatePopover::new(&state.borrow().sermon);
        header.set_title_widget(Some(&title_date.button));

        let menu_btn = MenuButton::new();
        menu_btn.set_icon_name("open-menu-symbolic");
        menu_btn.add_css_class("flat");
        menu_btn.set_tooltip_text(Some("Menu"));
        header.pack_end(&menu_btn);

        let menu_box = GtkBox::new(Orientation::Vertical, 2);
        menu_box.set_margin_top(6);
        menu_box.set_margin_bottom(6);
        menu_box.set_margin_start(6);
        menu_box.set_margin_end(6);
        let export_item = make_menu_item("Export…", "Ctrl+E");
        menu_box.append(&export_item);
        let preaching_view_item = make_menu_item("Preaching View", "Ctrl+Shift+P");
        menu_box.append(&preaching_view_item);
        let history_item = make_menu_item("History…", "Ctrl+Shift+H");
        menu_box.append(&history_item);
        let show_folder_item = make_menu_item("Show Sermons Folder", "");
        show_folder_item.set_tooltip_text(Some(&state.borrow().config.sermons_dir().display().to_string()));
        menu_box.append(&show_folder_item);
        let backup_setup_item = make_menu_item("Set Up Backup…", "");
        menu_box.append(&backup_setup_item);
        let menu_popover = gtk4::Popover::new();
        menu_popover.set_child(Some(&menu_box));
        menu_btn.set_popover(Some(&menu_popover));

        let undo_btn = Button::from_icon_name("edit-undo-symbolic");
        undo_btn.add_css_class("flat");
        undo_btn.set_tooltip_text(Some("Undo (Ctrl+Z)"));
        undo_btn.set_sensitive(false);
        header.pack_end(&undo_btn);

        let redo_btn = Button::from_icon_name("edit-redo-symbolic");
        redo_btn.add_css_class("flat");
        redo_btn.set_tooltip_text(Some("Redo (Ctrl+Shift+Z)"));
        redo_btn.set_sensitive(false);
        header.pack_end(&redo_btn);

        let sync_btn = Button::from_icon_name("vcs-push-symbolic");
        sync_btn.add_css_class("flat");
        sync_btn.set_tooltip_text(Some("Commit & Push to GitHub (Ctrl+Shift+G)"));
        header.pack_end(&sync_btn);

        // ── Sidebar: lectionary panel ─────────────────────────────────────
        let lectionary_panel = LectionaryPanel::new();
        lectionary_panel.refresh(&state.borrow().sermon);

        // ── Editor ────────────────────────────────────────────────────────
        let editor = Editor::new();

        let split_view = adw::OverlaySplitView::new();
        split_view.set_sidebar(Some(&lectionary_panel.root));
        split_view.set_content(Some(editor.widget()));
        split_view.set_show_sidebar(state.borrow().config.sidebar_visible);
        split_view.set_sidebar_width_fraction(state.borrow().config.sidebar_width_fraction);

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&split_view));

        let status_bar = StatusBar::new();

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.add_bottom_bar(&status_bar.root);
        toolbar_view.set_content(Some(&toast_overlay));

        window.set_content(Some(&toolbar_view));

        // ── The single door: every mutation flows through here ───────────
        let autosave_pending: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
        let apply: ApplyFn = make_apply(
            &state,
            &editor,
            &undo_btn,
            &redo_btn,
            &title_date,
            &lectionary_panel,
            &status_bar,
            &toast_overlay,
            &autosave_pending,
        );

        editor.init_dnd(&state, apply.clone());
        editor.rebuild(&state, apply.clone());
        title_date.init(&state, apply.clone());
        status_bar.init(apply.clone());
        status_bar.refresh(&state.borrow().sermon);

        {
            let state = state.clone();
            undo_btn.connect_clicked(move |_| {
                if perform_undo(&state) {
                    force_full_refresh(&state);
                }
            });
        }
        {
            let state = state.clone();
            redo_btn.connect_clicked(move |_| {
                if perform_redo(&state) {
                    force_full_refresh(&state);
                }
            });
        }

        // Rebuild widget tree after a direct undo/redo call (which bypasses
        // `apply` since there's no new `Cmd` to push — see buttons above).
        {
            let state = state.clone();
            let editor = editor.clone();
            let undo_btn = undo_btn.clone();
            let redo_btn = redo_btn.clone();
            let title_date = title_date.clone();
            let lectionary_panel = lectionary_panel.clone();
            let status_bar = status_bar.clone();
            let toast_overlay = toast_overlay.clone();
            let autosave_pending = autosave_pending.clone();
            REFRESH.with(|cell| {
                *cell.borrow_mut() = Some(Box::new(move || {
                    let recurse = make_apply(
                        &state,
                        &editor,
                        &undo_btn,
                        &redo_btn,
                        &title_date,
                        &lectionary_panel,
                        &status_bar,
                        &toast_overlay,
                        &autosave_pending,
                    );
                    editor.rebuild(&state, recurse);
                    title_date.refresh(&state.borrow().sermon);
                    lectionary_panel.refresh(&state.borrow().sermon);
                    status_bar.refresh(&state.borrow().sermon);
                    status_bar.set_dirty();
                    undo_btn.set_sensitive(state.borrow().undo.can_undo());
                    redo_btn.set_sensitive(state.borrow().undo.can_redo());
                    arm_autosave(&state, &toast_overlay, &status_bar, &autosave_pending);
                }));
            });
        }

        // ── Keybindings: Ctrl+Z / Ctrl+Shift+Z ───────────────────────────
        // Capture phase, not the default bubble: a focused Entry's own key
        // handling otherwise consumes Ctrl+Z before it would ever bubble up
        // to this controller, silently swallowing the shortcut whenever the
        // user is mid-edit — which is most of the time in this app.
        let key_ctl = gtk4::EventControllerKey::new();
        key_ctl.set_propagation_phase(gtk4::PropagationPhase::Capture);
        {
            let state = state.clone();
            key_ctl.connect_key_pressed(move |_, key, _, modifiers| {
                use gtk4::gdk::ModifierType;
                let ctrl = modifiers.contains(ModifierType::CONTROL_MASK);
                let shift = modifiers.contains(ModifierType::SHIFT_MASK);
                if ctrl && key == gtk4::gdk::Key::z && !shift {
                    if perform_undo(&state) {
                        force_full_refresh(&state);
                    }
                    return glib::Propagation::Stop;
                }
                if ctrl && shift && key == gtk4::gdk::Key::Z {
                    if perform_redo(&state) {
                        force_full_refresh(&state);
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
        }
        window.add_controller(key_ctl);

        // ── Version button → changelog ────────────────────────────────────
        {
            let window = window.clone();
            status_bar.version_btn.connect_clicked(move |_| {
                show_changelog(&window);
            });
        }

        // ── Hamburger menu: Export… ───────────────────────────────────────
        {
            let window = window.clone();
            let state = state.clone();
            let menu_popover = menu_popover.clone();
            export_item.connect_clicked(move |_| {
                menu_popover.popdown();
                ExportDialog::new(&window, state.borrow().sermon.clone()).present();
            });
        }
        {
            let win_for_closure = window.clone();
            let state = state.clone();
            let ctl = gtk4::EventControllerKey::new();
            ctl.set_propagation_phase(gtk4::PropagationPhase::Capture);
            ctl.connect_key_pressed(move |_, key, _, modifiers| {
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && key == gtk4::gdk::Key::e
                {
                    ExportDialog::new(&win_for_closure, state.borrow().sermon.clone()).present();
                    return glib::Propagation::Stop;
                }
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
                    && key == gtk4::gdk::Key::P
                {
                    PreachingView::new(&win_for_closure, &state.borrow().sermon).present();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            window.add_controller(ctl);
        }

        // ── Hamburger menu: Preaching View ────────────────────────────────
        {
            let state = state.clone();
            let menu_popover = menu_popover.clone();
            let window = window.clone();
            preaching_view_item.connect_clicked(move |_| {
                menu_popover.popdown();
                PreachingView::new(&window, &state.borrow().sermon).present();
            });
        }

        // ── Hamburger menu: History… ───────────────────────────────────────
        {
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            let menu_popover = menu_popover.clone();
            let window = window.clone();
            history_item.connect_clicked(move |_| {
                menu_popover.popdown();
                open_history_window(&window, &state, &toast_overlay);
            });
        }
        {
            let win_for_closure = window.clone();
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            let ctl = gtk4::EventControllerKey::new();
            ctl.set_propagation_phase(gtk4::PropagationPhase::Capture);
            ctl.connect_key_pressed(move |_, key, _, modifiers| {
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
                    && key == gtk4::gdk::Key::H
                {
                    open_history_window(&win_for_closure, &state, &toast_overlay);
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            window.add_controller(ctl);
        }

        // ── Hamburger menu: Show Sermons Folder ───────────────────────────
        {
            let state = state.clone();
            let menu_popover = menu_popover.clone();
            let toast_overlay = toast_overlay.clone();
            show_folder_item.connect_clicked(move |_| {
                menu_popover.popdown();
                let dir = state.borrow().config.sermons_dir();
                let uri = format!("file://{}", dir.display());
                if gtk4::gio::AppInfo::launch_default_for_uri(
                    &uri,
                    None::<&gtk4::gio::AppLaunchContext>,
                )
                .is_err()
                {
                    show_toast(&toast_overlay, &format!("Sermons are saved in {}", dir.display()));
                }
            });
        }

        // ── Hamburger menu: Set Up Backup… ────────────────────────────────
        {
            let window = window.clone();
            let state = state.clone();
            let menu_popover = menu_popover.clone();
            backup_setup_item.connect_clicked(move |_| {
                menu_popover.popdown();
                let work_dir = state.borrow().config.work_dir.clone();
                BackupSetup::new(&window, &work_dir).present();
            });
        }

        // ── Sync button ──────────────────────────────────────────────────
        {
            let win_for_closure = window.clone();
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            let btn_for_closure = sync_btn.clone();
            sync_btn.connect_clicked(move |_| {
                let work_dir = state.borrow().config.work_dir.clone();
                if !git_sync::has_remote(&work_dir) {
                    let setup = BackupSetup::new(&win_for_closure, &work_dir);
                    let window2 = win_for_closure.clone();
                    let toast_overlay2 = toast_overlay.clone();
                    let sync_btn2 = btn_for_closure.clone();
                    let work_dir2 = work_dir.clone();
                    setup.window().connect_destroy(move |_| {
                        if git_sync::has_remote(&work_dir2) {
                            do_sync(work_dir2.clone(), window2.clone(), toast_overlay2.clone(), sync_btn2.clone());
                        }
                    });
                    setup.present();
                    return;
                }
                do_sync(work_dir, win_for_closure.clone(), toast_overlay.clone(), btn_for_closure.clone());
            });
        }
        {
            let win_for_closure = window.clone();
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            let sync_btn = sync_btn.clone();
            let ctl = gtk4::EventControllerKey::new();
            ctl.set_propagation_phase(gtk4::PropagationPhase::Capture);
            ctl.connect_key_pressed(move |_, key, _, modifiers| {
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
                    && key == gtk4::gdk::Key::G
                {
                    let work_dir = state.borrow().config.work_dir.clone();
                    if git_sync::has_remote(&work_dir) {
                        do_sync(work_dir, win_for_closure.clone(), toast_overlay.clone(), sync_btn.clone());
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            window.add_controller(ctl);
        }

        // ── Welcome / What's New ─────────────────────────────────────────
        if WelcomeWindow::should_show() {
            let is_first_run = WelcomeWindow::is_first_run();
            let welcome = WelcomeWindow::new(&window, is_first_run);
            welcome.set_on_dismissed(WelcomeWindow::mark_shown);
            welcome.present();
        }

        // ── Library ──────────────────────────────────────────────────────
        let library_window = LibraryWindow::new(&window, state.borrow().config.sermons_dir());
        library_window.set_current_open(Some(state.borrow().path.clone()));
        {
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            let lw = library_window.clone();
            library_window.set_on_open(move |path| {
                switch_to_sermon(&state, &toast_overlay, path);
                lw.set_current_open(Some(state.borrow().path.clone()));
                force_full_refresh(&state);
                lw.window().close();
            });
        }
        let create_new_sermon: Rc<dyn Fn(Option<String>)> = {
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            let lw = library_window.clone();
            Rc::new(move |template_id: Option<String>| {
                let sermon = crate::sermon_templates::build_sermon(template_id.as_deref());
                let path = storage::new_sermon_path(&state.borrow().config.sermons_dir(), &sermon);
                match storage::save_sermon(&path, &sermon) {
                    Ok(()) => {
                        switch_to_sermon(&state, &toast_overlay, path);
                        lw.set_current_open(Some(state.borrow().path.clone()));
                        force_full_refresh(&state);
                        lw.window().close();
                    }
                    Err(e) => {
                        tracing::warn!("failed to create new sermon at {}: {e}", path.display());
                        show_toast(&toast_overlay, "Couldn't create a new sermon");
                    }
                }
            })
        };
        {
            let create_new_sermon = create_new_sermon.clone();
            library_window.set_on_new(move |template_id| create_new_sermon(template_id));
        }
        {
            let state = state.clone();
            let toast_overlay = toast_overlay.clone();
            library_window.set_on_delete(move |path| {
                if path == state.borrow().path {
                    // The row for the currently-open sermon has no delete
                    // button (see `library_window::build_sermon_row`), so
                    // this only guards against a stale callback firing late.
                    return;
                }
                if let Err(e) = std::fs::remove_file(&path) {
                    tracing::warn!("failed to delete sermon {}: {e}", path.display());
                    show_toast(&toast_overlay, "Couldn't delete that sermon");
                }
                let sermons_dir = state.borrow().config.sermons_dir();
                state.borrow_mut().library = LibraryIndex::scan(&sermons_dir);
            });
        }
        {
            let library_window = library_window.clone();
            library_btn.connect_clicked(move |_| library_window.present());
        }
        {
            let library_window = library_window.clone();
            let ctl = gtk4::EventControllerKey::new();
            ctl.set_propagation_phase(gtk4::PropagationPhase::Capture);
            ctl.connect_key_pressed(move |_, key, _, modifiers| {
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && key == gtk4::gdk::Key::l
                {
                    library_window.present();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            window.add_controller(ctl);
        }

        // ── Command palette (Ctrl+K) ──────────────────────────────────────
        let palette = CommandPalette::new(&window);
        {
            let state = state.clone();
            let apply = apply.clone();
            let editor = editor.clone();
            let library_window = library_window.clone();
            let window = window.clone();
            let split_view = split_view.clone();
            let create_new_sermon = create_new_sermon.clone();
            let toast_overlay = toast_overlay.clone();
            palette.set_on_activate(move |id| match id {
                "new_sermon" => create_new_sermon(None),
                "open_library" => library_window.present(),
                "export" => {
                    ExportDialog::new(&window, state.borrow().sermon.clone()).present();
                }
                "preaching_view" => {
                    PreachingView::new(&window, &state.borrow().sermon).present();
                }
                "history" => open_history_window(&window, &state, &toast_overlay),
                "undo" => {
                    if perform_undo(&state) {
                        force_full_refresh(&state);
                    }
                }
                "redo" => {
                    if perform_redo(&state) {
                        force_full_refresh(&state);
                    }
                }
                "toggle_sidebar" => split_view.set_show_sidebar(!split_view.shows_sidebar()),
                "add_movement" => {
                    let at = state.borrow().sermon.movements.len();
                    apply(Cmd::InsertMovement {
                        at,
                        movement: crate::model::Movement::new(at),
                    });
                }
                "changelog" => show_changelog(&window),
                other => {
                    editor.focus_by_name(other);
                }
            });
        }
        {
            let state = state.clone();
            let palette = palette.clone();
            let ctl = gtk4::EventControllerKey::new();
            ctl.set_propagation_phase(gtk4::PropagationPhase::Capture);
            ctl.connect_key_pressed(move |_, key, _, modifiers| {
                if modifiers.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                    && key == gtk4::gdk::Key::k
                {
                    let mut items = default_commands();
                    items.extend(outline_items(&state.borrow().sermon));
                    palette.set_items(items);
                    palette.show();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            window.add_controller(ctl);
        }

        // ── Geometry + sidebar persistence (debounced 400 ms; ready-gated so
        // GTK's initial layout pass — which fires these same notify signals
        // once on realize — doesn't get saved as if the user had resized).
        let geometry_ready = Rc::new(Cell::new(false));
        {
            let ready = geometry_ready.clone();
            window.connect_realize(move |_| {
                let ready = ready.clone();
                glib::idle_add_local_once(move || ready.set(true));
            });
        }
        let geometry_save_pending: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
        {
            let state = state.clone();
            let ready = geometry_ready.clone();
            let pending = geometry_save_pending.clone();
            window.connect_default_width_notify(move |w| {
                if !ready.get() || w.is_maximized() {
                    return;
                }
                state.borrow_mut().config.window_width = w.default_width();
                arm_config_save(&state, &pending);
            });
        }
        {
            let state = state.clone();
            let ready = geometry_ready.clone();
            let pending = geometry_save_pending.clone();
            window.connect_default_height_notify(move |w| {
                if !ready.get() || w.is_maximized() {
                    return;
                }
                state.borrow_mut().config.window_height = w.default_height();
                arm_config_save(&state, &pending);
            });
        }
        {
            let state = state.clone();
            let ready = geometry_ready.clone();
            window.connect_maximized_notify(move |w| {
                if !ready.get() {
                    return;
                }
                state.borrow_mut().config.window_maximized = w.is_maximized();
                let _ = state.borrow().config.save();
            });
        }
        {
            let state = state.clone();
            let ready = geometry_ready.clone();
            let pending = geometry_save_pending.clone();
            split_view.connect_sidebar_width_fraction_notify(move |sv| {
                if !ready.get() {
                    return;
                }
                state.borrow_mut().config.sidebar_width_fraction = sv.sidebar_width_fraction();
                arm_config_save(&state, &pending);
            });
        }
        {
            let state = state.clone();
            let ready = geometry_ready.clone();
            split_view.connect_show_sidebar_notify(move |sv| {
                if !ready.get() {
                    return;
                }
                state.borrow_mut().config.sidebar_visible = sv.shows_sidebar();
                let _ = state.borrow().config.save();
            });
        }

        // ── Save on close ─────────────────────────────────────────────────
        {
            let state = state.clone();
            window.connect_close_request(move |_| {
                let st = state.borrow();
                if st.dirty {
                    let mut sermon = st.sermon.clone();
                    let _ = storage::save_touched(&st.path, &mut sermon);
                }
                let mut cfg = st.config.clone();
                cfg.sidebar_visible = split_view.shows_sidebar();
                let _ = cfg.save();
                glib::Propagation::Proceed
            });
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

thread_local! {
    static REFRESH: RefCell<Option<Box<dyn Fn()>>> = const { RefCell::new(None) };
}

/// Undo/redo mutate the sermon directly (there's no new `Cmd` to route
/// through `apply`), so they call back into the same rebuild/refresh logic
/// registered by `AppWindow::new` via this thread-local instead of
/// duplicating it.
fn force_full_refresh(_state: &Rc<RefCell<AppState>>) {
    REFRESH.with(|cell| {
        if let Some(f) = cell.borrow().as_ref() {
            f();
        }
    });
}

/// Undo/redo mutate `sermon` directly rather than through `apply()`, so they
/// need their own `dirty` marking — without this, undoing or redoing a
/// change was never picked up by autosave (a real bug: `dirty` was only ever
/// set by `apply()`'s `Cmd` path, so an undo could silently go unsaved).
fn perform_undo(state: &Rc<RefCell<AppState>>) -> bool {
    let mut st = state.borrow_mut();
    let AppState { undo, sermon, .. } = &mut *st;
    let changed = undo.undo(sermon);
    if changed {
        st.dirty = true;
    }
    changed
}

fn open_history_window(
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    toast_overlay: &adw::ToastOverlay,
) {
    let (repo_path, file_path) = {
        let st = state.borrow();
        (st.config.work_dir.clone(), st.path.clone())
    };
    let state = state.clone();
    let toast_overlay = toast_overlay.clone();
    let history = HistoryWindow::new(window, repo_path, file_path, move |content| {
        if restore_sermon_version(&state, &toast_overlay, content) {
            force_full_refresh(&state);
        }
    });
    history.present();
}

/// Replaces the open sermon in-memory with a historical version restored
/// from git (see `HistoryWindow`) — mirrors undo/redo's direct-mutation
/// pattern (bypasses `apply()`, since there's no single `Cmd` for "load an
/// entirely different sermon") so the change flows through the same
/// `force_full_refresh` + autosave path afterward.
fn restore_sermon_version(
    state: &Rc<RefCell<AppState>>,
    toast_overlay: &adw::ToastOverlay,
    content: String,
) -> bool {
    let sermon: Sermon = match toml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("failed to parse historical sermon version: {e}");
            show_toast(toast_overlay, "Couldn't restore that version — the file may be corrupt");
            return false;
        }
    };
    let mut st = state.borrow_mut();
    st.sermon = sermon;
    st.undo = crate::commands::UndoStack::new();
    st.dirty = true;
    true
}

fn perform_redo(state: &Rc<RefCell<AppState>>) -> bool {
    let mut st = state.borrow_mut();
    let AppState { undo, sermon, .. } = &mut *st;
    let changed = undo.redo(sermon);
    if changed {
        st.dirty = true;
    }
    changed
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn make_apply(
    state: &Rc<RefCell<AppState>>,
    editor: &Rc<Editor>,
    undo_btn: &Button,
    redo_btn: &Button,
    title_date: &Rc<TitleDatePopover>,
    lectionary_panel: &Rc<LectionaryPanel>,
    status_bar: &Rc<StatusBar>,
    toast_overlay: &adw::ToastOverlay,
    autosave_pending: &Rc<Cell<Option<glib::SourceId>>>,
) -> ApplyFn {
    let state = state.clone();
    let editor = editor.clone();
    let undo_btn = undo_btn.clone();
    let redo_btn = redo_btn.clone();
    let title_date = title_date.clone();
    let lectionary_panel = lectionary_panel.clone();
    let status_bar = status_bar.clone();
    let toast_overlay = toast_overlay.clone();
    let autosave_pending = autosave_pending.clone();
    Rc::new(move |cmd: Cmd| {
        let structural = cmd.is_structural();
        note_past_sermon_reuse(&cmd, &state, &toast_overlay);
        {
            let mut st = state.borrow_mut();
            let AppState { undo, sermon, .. } = &mut *st;
            undo.push_applying(sermon, cmd);
            st.dirty = true;
        }
        status_bar.set_dirty();
        if structural {
            let recurse = make_apply(
                &state,
                &editor,
                &undo_btn,
                &redo_btn,
                &title_date,
                &lectionary_panel,
                &status_bar,
                &toast_overlay,
                &autosave_pending,
            );
            editor.rebuild(&state, recurse);
        }
        title_date.refresh(&state.borrow().sermon);
        lectionary_panel.refresh(&state.borrow().sermon);
        status_bar.refresh(&state.borrow().sermon);
        undo_btn.set_sensitive(state.borrow().undo.can_undo());
        redo_btn.set_sensitive(state.borrow().undo.can_redo());
        arm_autosave(&state, &toast_overlay, &status_bar, &autosave_pending);
    })
}

fn show_toast(overlay: &adw::ToastOverlay, message: &str) {
    overlay.add_toast(adw::Toast::new(message));
}

/// When a scripture (s.) tag is added, checks whether any *other* sermon in
/// the library already used it, and surfaces a toast naming it — so
/// repeating a passage is a deliberate choice, not an accident nobody
/// noticed. Only fires on genuinely new tags (not on removal), and only
/// scripture tags (`s.`), not theme (`t.`) tags — a repeated theme is far
/// more common and expected than a repeated passage.
fn note_past_sermon_reuse(cmd: &Cmd, state: &Rc<RefCell<AppState>>, toast_overlay: &adw::ToastOverlay) {
    let Cmd::SetSermonTags { kind: SermonTagKind::S, old, new } = cmd else {
        return;
    };
    let added = new.iter().find(|t| !old.contains(t));
    let Some(tag) = added else {
        return;
    };

    let st = state.borrow();
    let current_id = &st.sermon.id;
    let found = st
        .library
        .sermons
        .iter()
        .find(|(_, s)| &s.id != current_id && s.s_tags.iter().any(|t| t == tag));

    if let Some((_, sermon)) = found {
        let date = sermon
            .planned_date
            .map(|d| d.format("%B %-d, %Y").to_string())
            .unwrap_or_else(|| "an earlier sermon".to_string());
        let title = sermon.display_title().to_string();
        let tag = tag.clone();
        drop(st);
        show_toast(toast_overlay, &format!("You've also preached on \"{tag}\" — {title} ({date})"));
    }
}

fn arm_autosave(
    state: &Rc<RefCell<AppState>>,
    toast_overlay: &adw::ToastOverlay,
    status_bar: &Rc<StatusBar>,
    pending: &Rc<Cell<Option<glib::SourceId>>>,
) {
    if let Some(id) = pending.take() {
        id.remove();
    }
    let debounce_ms = state.borrow().config.autosave_debounce_ms;
    let state = state.clone();
    let toast_overlay = toast_overlay.clone();
    let status_bar = status_bar.clone();
    let pending_for_cb = pending.clone();
    let id = glib::timeout_add_local_once(Duration::from_millis(debounce_ms), move || {
        pending_for_cb.set(None);
        let mut st = state.borrow_mut();
        if st.dirty {
            let path = st.path.clone();
            match storage::save_touched(&path, &mut st.sermon) {
                Ok(()) => {
                    st.dirty = false;
                    st.library = crate::library::LibraryIndex::scan(&st.config.sermons_dir());
                    status_bar.set_saved();
                }
                Err(e) => {
                    tracing::warn!("autosave failed for {}: {e}", path.display());
                    show_toast(&toast_overlay, "Couldn't save — check disk space or permissions");
                }
            }
        }
    });
    pending.set(Some(id));
}

/// A hamburger-menu row: label flush-left, a dim keyboard-shortcut caption
/// flush-right — see the root CLAUDE.md's "hand-built popover" UI standard.
fn make_menu_item(label: &str, shortcut: &str) -> Button {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    let label_lbl = Label::new(Some(label));
    label_lbl.set_xalign(0.0);
    label_lbl.set_hexpand(true);
    let shortcut_lbl = Label::new(Some(shortcut));
    shortcut_lbl.add_css_class("dim-label");
    shortcut_lbl.add_css_class("caption");
    row.append(&label_lbl);
    row.append(&shortcut_lbl);

    let btn = Button::new();
    btn.add_css_class("flat");
    btn.set_child(Some(&row));
    btn
}

fn arm_config_save(state: &Rc<RefCell<AppState>>, pending: &Rc<Cell<Option<glib::SourceId>>>) {
    if let Some(id) = pending.take() {
        id.remove();
    }
    let state = state.clone();
    let id = glib::timeout_add_local_once(Duration::from_millis(400), move || {
        let _ = state.borrow().config.save();
    });
    pending.set(Some(id));
}

fn load_or_create_sermon(config: &Config) -> (Sermon, std::path::PathBuf) {
    if let Some(last) = &config.last_sermon {
        if let Ok(sermon) = storage::load_sermon(last) {
            return (sermon, last.clone());
        }
    }
    let existing = storage::scan_sermons(&config.sermons_dir());
    if let Some((path, sermon)) = existing.into_iter().next() {
        return (sermon, path);
    }
    let sermon = Sermon::new();
    let path = storage::new_sermon_path(&config.sermons_dir(), &sermon);
    (sermon, path)
}

/// Saves the currently open sermon if dirty, then loads `path` as the new
/// open sermon — undo history resets, since it belongs to the old sermon.
/// Caller is responsible for the widget refresh (`force_full_refresh`).
fn switch_to_sermon(state: &Rc<RefCell<AppState>>, toast_overlay: &adw::ToastOverlay, path: std::path::PathBuf) {
    let mut st = state.borrow_mut();
    if st.dirty {
        let mut sermon = st.sermon.clone();
        if let Err(e) = storage::save_touched(&st.path, &mut sermon) {
            tracing::warn!("save before switching sermons failed for {}: {e}", st.path.display());
            show_toast(toast_overlay, "Couldn't save the current sermon before switching");
        }
    }
    let sermon = match storage::load_sermon(&path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("failed to open sermon {}: {e}", path.display());
            show_toast(toast_overlay, "Couldn't open that sermon — the file may be corrupt");
            return;
        }
    };
    st.sermon = sermon;
    st.path = path.clone();
    st.undo = crate::commands::UndoStack::new();
    st.dirty = false;
    st.config.last_sermon = Some(path.clone());
    st.config.push_recent(path);
    let _ = st.config.save();
}

fn do_sync(
    root: std::path::PathBuf,
    window: adw::ApplicationWindow,
    overlay: adw::ToastOverlay,
    btn: Button,
) {
    use std::sync::mpsc::TryRecvError;

    btn.set_sensitive(false);

    let token = crate::secret_store::load_github_token();
    let root_for_thread = root.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel::<git_sync::SyncResult>(1);
    std::thread::spawn(move || {
        tx.send(git_sync::sync(&root_for_thread, token.as_deref())).ok();
    });

    let rx = Rc::new(rx);
    glib::timeout_add_local(Duration::from_millis(100), move || match rx.try_recv() {
        Ok(result) => {
            btn.set_sensitive(true);
            show_sync_result(&window, &overlay, result);
            glib::ControlFlow::Break
        }
        Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(TryRecvError::Disconnected) => {
            btn.set_sensitive(true);
            glib::ControlFlow::Break
        }
    });
}

fn show_sync_result(window: &adw::ApplicationWindow, overlay: &adw::ToastOverlay, result: git_sync::SyncResult) {
    if let Some(err) = result.error {
        show_alert(window, "Sync Failed", &err);
        return;
    }
    if !result.push_errors.is_empty() {
        let detail = result.push_errors.join("\n");
        if result.auth_failed {
            show_alert(
                window,
                "GitHub authentication failed",
                "Your stored GitHub token was rejected. Open the hamburger menu → Set Up Backup… and sign in again.",
            );
            return;
        }
        // Matches only git's own "CONFLICT (...)" marker — not a substring
        // of our own wrapper text, which previously made every pull failure
        // (auth, network, missing branch, ...) get misreported as a merge
        // conflict.
        let is_conflict = detail.contains("CONFLICT");
        if result.pushed {
            let summary = result.commit_message.lines().next().unwrap_or("Synced").to_string();
            show_toast(overlay, &format!("Synced — {summary}"));
            show_alert(window, "Some remotes failed", &detail);
        } else if is_conflict {
            show_alert(
                window,
                "Merge conflict — sync paused",
                "A sermon changed on GitHub in a way that conflicts with a change made here. \
                 Nothing was lost — your local copy is unchanged, and autosave keeps working \
                 normally. Iskra will keep retrying on every sync until the conflict is \
                 resolved on GitHub.com (or by someone comfortable with git from the command \
                 line, in the folder under Menu → Show Sermons Folder).",
            );
        } else {
            show_alert(window, "Sync Failed", &detail);
        }
        return;
    }
    if result.pushed {
        let summary = result.commit_message.lines().next().unwrap_or("Synced").to_string();
        show_toast(overlay, &format!("Synced — {summary}"));
    } else if result.committed {
        show_toast(overlay, "Committed locally — no remote push");
    } else {
        show_toast(overlay, "Nothing to sync");
    }
}

fn show_alert(window: &adw::ApplicationWindow, title: &str, body: &str) {
    let dlg = adw::MessageDialog::new(Some(window), Some(title), Some(body));
    dlg.add_response("ok", "OK");
    dlg.present();
}
