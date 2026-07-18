use std::path::PathBuf;

use crate::commands::UndoStack;
use crate::config::Config;
use crate::library::LibraryIndex;
use crate::model::Sermon;

/// The single in-memory source of truth for the sermon currently open in the
/// editor. Lives behind an `Rc<RefCell<AppState>>` owned by `AppWindow`; every
/// mutation goes through `ui::app_window::apply` (the "single door" — see
/// Plans/plan.md) rather than touching `sermon` directly.
pub struct AppState {
    pub sermon: Sermon,
    pub path: PathBuf,
    pub undo: UndoStack,
    pub dirty: bool,
    pub config: Config,
    /// Set for the duration of a drag gesture so autosave-triggered rebuilds
    /// don't reparent/destroy a widget GTK's drag machinery still holds.
    pub drag_active: bool,
    /// Rescanned on open and after every save — see `library::LibraryIndex`.
    pub library: LibraryIndex,
}

impl AppState {
    pub fn new(sermon: Sermon, path: PathBuf, config: Config) -> Self {
        let library = LibraryIndex::scan(&config.sermons_dir());
        Self {
            sermon,
            path,
            undo: UndoStack::new(),
            dirty: false,
            config,
            drag_active: false,
            library,
        }
    }
}
