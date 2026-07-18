use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::commands::Cmd;
use crate::config::Config;
use crate::model::Sermon;
use crate::state::AppState;
use crate::storage;
use crate::ui::editor::{ApplyFn, Editor};
use crate::ui::status_bar::StatusBar;
use crate::ui::styles;

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
        library_btn.set_tooltip_text(Some("Open the sermon library (coming in dev4)"));
        library_btn.set_sensitive(false);
        header.pack_start(&library_btn);

        let title_widget = adw::WindowTitle::new(
            state.borrow().sermon.display_title(),
            &planned_date_subtitle(&state.borrow().sermon),
        );
        header.set_title_widget(Some(&title_widget));

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

        // ── Sidebar (empty for now — see Plans/plan.md §4.5) ─────────────
        let sidebar_placeholder = GtkBox::new(Orientation::Vertical, 0);
        let sidebar_header = Label::new(Some("Lectionary"));
        sidebar_header.add_css_class("sidebar-header");
        sidebar_header.set_xalign(0.0);
        sidebar_placeholder.append(&sidebar_header);
        let sidebar_note = Label::new(Some("Readings appear here once a date is planned."));
        sidebar_note.add_css_class("dim-label");
        sidebar_note.add_css_class("caption");
        sidebar_note.set_wrap(true);
        sidebar_note.set_margin_start(12);
        sidebar_note.set_margin_end(12);
        sidebar_placeholder.append(&sidebar_note);

        // ── Editor ────────────────────────────────────────────────────────
        let editor = Editor::new();

        let split_view = adw::OverlaySplitView::new();
        split_view.set_sidebar(Some(&sidebar_placeholder));
        split_view.set_content(Some(editor.widget()));
        split_view.set_show_sidebar(state.borrow().config.sidebar_visible);
        split_view.set_sidebar_width_fraction(0.22);

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
            &title_widget,
            &autosave_pending,
        );

        editor.init_dnd(&state, apply.clone());
        editor.rebuild(&state, apply.clone());

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
            let title_widget = title_widget.clone();
            let autosave_pending = autosave_pending.clone();
            REFRESH.with(|cell| {
                *cell.borrow_mut() = Some(Box::new(move || {
                    let recurse = make_apply(
                        &state,
                        &editor,
                        &undo_btn,
                        &redo_btn,
                        &title_widget,
                        &autosave_pending,
                    );
                    editor.rebuild(&state, recurse);
                    title_widget.set_title(state.borrow().sermon.display_title());
                    title_widget.set_subtitle(&planned_date_subtitle(&state.borrow().sermon));
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
    title_widget: &adw::WindowTitle,
    autosave_pending: &Rc<Cell<Option<glib::SourceId>>>,
) -> ApplyFn {
    let state = state.clone();
    let editor = editor.clone();
    let undo_btn = undo_btn.clone();
    let redo_btn = redo_btn.clone();
    let title_widget = title_widget.clone();
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
                &title_widget,
                &autosave_pending,
            );
            editor.rebuild(&state, recurse);
        }
        title_widget.set_title(state.borrow().sermon.display_title());
        title_widget.set_subtitle(&planned_date_subtitle(&state.borrow().sermon));
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
            }
        }
    });
    pending.set(Some(id));
}

fn planned_date_subtitle(sermon: &Sermon) -> String {
    match sermon.planned_date {
        Some(d) => d.format("%B %-d, %Y").to_string(),
        None => "No date planned".to_string(),
    }
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
