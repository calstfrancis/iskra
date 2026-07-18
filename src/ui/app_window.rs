use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::Button;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::commands::Cmd;
use crate::config::Config;
use crate::library::LibraryIndex;
use crate::model::Sermon;
use crate::state::AppState;
use crate::storage;
use crate::ui::changelog_window::show_changelog;
use crate::ui::editor::{ApplyFn, Editor};
use crate::ui::lectionary_panel::LectionaryPanel;
use crate::ui::library_window::LibraryWindow;
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

        let menu_btn = Button::from_icon_name("open-menu-symbolic");
        menu_btn.add_css_class("flat");
        menu_btn.set_tooltip_text(Some("Menu"));
        header.pack_end(&menu_btn);

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
                let changed = {
                    let mut st = state.borrow_mut();
                    let AppState { undo, sermon, .. } = &mut *st;
                    undo.undo(sermon)
                };
                if changed {
                    force_full_refresh(&state);
                }
            });
        }
        {
            let state = state.clone();
            redo_btn.connect_clicked(move |_| {
                let changed = {
                    let mut st = state.borrow_mut();
                    let AppState { undo, sermon, .. } = &mut *st;
                    undo.redo(sermon)
                };
                if changed {
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
                        &autosave_pending,
                    );
                    editor.rebuild(&state, recurse);
                    title_date.refresh(&state.borrow().sermon);
                    lectionary_panel.refresh(&state.borrow().sermon);
                    status_bar.refresh(&state.borrow().sermon);
                    undo_btn.set_sensitive(state.borrow().undo.can_undo());
                    redo_btn.set_sensitive(state.borrow().undo.can_redo());
                    arm_autosave(&state, &autosave_pending);
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
                    let changed = {
                        let mut st = state.borrow_mut();
                        let AppState { undo, sermon, .. } = &mut *st;
                        undo.undo(sermon)
                    };
                    if changed {
                        force_full_refresh(&state);
                    }
                    return glib::Propagation::Stop;
                }
                if ctrl && shift && key == gtk4::gdk::Key::Z {
                    let changed = {
                        let mut st = state.borrow_mut();
                        let AppState { undo, sermon, .. } = &mut *st;
                        undo.redo(sermon)
                    };
                    if changed {
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
            let lw = library_window.clone();
            library_window.set_on_open(move |path| {
                switch_to_sermon(&state, path);
                lw.set_current_open(Some(state.borrow().path.clone()));
                force_full_refresh(&state);
                lw.window().close();
            });
        }
        {
            let state = state.clone();
            let lw = library_window.clone();
            library_window.set_on_new(move || {
                let sermon = Sermon::new();
                let path = storage::new_sermon_path(&state.borrow().config.sermons_dir(), &sermon);
                if storage::save_sermon(&path, &sermon).is_ok() {
                    switch_to_sermon(&state, path);
                    lw.set_current_open(Some(state.borrow().path.clone()));
                    force_full_refresh(&state);
                    lw.window().close();
                }
            });
        }
        {
            let state = state.clone();
            library_window.set_on_delete(move |path| {
                if path == state.borrow().path {
                    // The row for the currently-open sermon has no delete
                    // button (see `library_window::build_sermon_row`), so
                    // this only guards against a stale callback firing late.
                    return;
                }
                let _ = std::fs::remove_file(&path);
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

#[allow(clippy::too_many_arguments)]
fn make_apply(
    state: &Rc<RefCell<AppState>>,
    editor: &Rc<Editor>,
    undo_btn: &Button,
    redo_btn: &Button,
    title_date: &Rc<TitleDatePopover>,
    lectionary_panel: &Rc<LectionaryPanel>,
    status_bar: &Rc<StatusBar>,
    autosave_pending: &Rc<Cell<Option<glib::SourceId>>>,
) -> ApplyFn {
    let state = state.clone();
    let editor = editor.clone();
    let undo_btn = undo_btn.clone();
    let redo_btn = redo_btn.clone();
    let title_date = title_date.clone();
    let lectionary_panel = lectionary_panel.clone();
    let status_bar = status_bar.clone();
    let autosave_pending = autosave_pending.clone();
    Rc::new(move |cmd: Cmd| {
        let structural = cmd.is_structural();
        {
            let mut st = state.borrow_mut();
            let AppState { undo, sermon, .. } = &mut *st;
            undo.push_applying(sermon, cmd);
            st.dirty = true;
        }
        if structural {
            let recurse = make_apply(
                &state,
                &editor,
                &undo_btn,
                &redo_btn,
                &title_date,
                &lectionary_panel,
                &status_bar,
                &autosave_pending,
            );
            editor.rebuild(&state, recurse);
        }
        title_date.refresh(&state.borrow().sermon);
        lectionary_panel.refresh(&state.borrow().sermon);
        status_bar.refresh(&state.borrow().sermon);
        undo_btn.set_sensitive(state.borrow().undo.can_undo());
        redo_btn.set_sensitive(state.borrow().undo.can_redo());
        arm_autosave(&state, &autosave_pending);
    })
}

fn arm_autosave(state: &Rc<RefCell<AppState>>, pending: &Rc<Cell<Option<glib::SourceId>>>) {
    if let Some(id) = pending.take() {
        id.remove();
    }
    let debounce_ms = state.borrow().config.autosave_debounce_ms;
    let state = state.clone();
    let pending_for_cb = pending.clone();
    let id = glib::timeout_add_local_once(Duration::from_millis(debounce_ms), move || {
        pending_for_cb.set(None);
        let mut st = state.borrow_mut();
        if st.dirty {
            let path = st.path.clone();
            if storage::save_touched(&path, &mut st.sermon).is_ok() {
                st.dirty = false;
                st.library = crate::library::LibraryIndex::scan(&st.config.sermons_dir());
            }
        }
    });
    pending.set(Some(id));
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
fn switch_to_sermon(state: &Rc<RefCell<AppState>>, path: std::path::PathBuf) {
    let mut st = state.borrow_mut();
    if st.dirty {
        let mut sermon = st.sermon.clone();
        let _ = storage::save_touched(&st.path, &mut sermon);
    }
    let Ok(sermon) = storage::load_sermon(&path) else {
        return;
    };
    st.sermon = sermon;
    st.path = path.clone();
    st.undo = crate::commands::UndoStack::new();
    st.dirty = false;
    st.config.last_sermon = Some(path.clone());
    st.config.push_recent(path);
    let _ = st.config.save();
}
