//! Single home for Iskra's static, app-wide CSS, loaded once at
//! `STYLE_PROVIDER_PRIORITY_APPLICATION`. Seeded from zerkalo/src/ui/styles.rs
//! (shared class names kept: `.status-toggle`, `.tag-chip`, `.paned > separator`,
//! `.sidebar-header`) plus Iskra-specific idea/movement/dnd classes.

const GLOBAL_CSS: &str = ".paned > separator { \
        min-width: 5px; \
        min-height: 5px; \
        transition: background-color 150ms ease; \
    } \
    .paned > separator:hover { \
        background-color: alpha(@accent_color, 0.45); \
    } \
    .status-toggle label { \
        opacity: 0.7; \
    } \
    .status-toggle:focus label, \
    .status-toggle:hover label { \
        opacity: 1.0; \
    } \
    .tag-chip { \
        border-radius: 999px; \
        padding: 2px 4px 2px 11px; \
        font-size: 0.78em; \
        font-weight: 600; \
        box-shadow: 0 1px 1px alpha(black, 0.06); \
    } \
    .tag-chip-s { \
        background-color: @accent_bg_color; \
        color: @accent_fg_color; \
    } \
    .tag-chip-t { \
        background: alpha(@success_color, 0.24); \
        color: @success_color; \
    } \
    .tag-chip-remove { \
        opacity: 0.55; \
        padding: 3px; \
        margin-left: 2px; \
        border-radius: 999px; \
        min-width: 0; \
        min-height: 0; \
    } \
    .tag-chip-remove:hover { \
        opacity: 1.0; \
        background: alpha(@window_fg_color, 0.16); \
    } \
    .tag-quick-pick { \
        font-size: 0.8em; \
        padding: 2px 8px; \
        min-height: 0; \
        border-radius: 999px; \
        background: alpha(@window_fg_color, 0.08); \
    } \
    .tag-quick-pick:hover { \
        background: alpha(@accent_color, 0.18); \
    } \
    .sidebar-header { \
        font-size: 0.75em; \
        font-weight: bold; \
        color: alpha(@window_fg_color, 0.55); \
        padding: 8px 12px 2px 12px; \
    } \
    .movement-card { \
        background: alpha(@window_fg_color, 0.035); \
        border: 1px solid alpha(@borders, 1.0); \
        border-radius: 12px; \
        padding: 8px; \
        box-shadow: 0 2px 7px alpha(black, 0.14); \
    } \
    .movement-card-header { \
        padding: 4px 4px 4px 8px; \
        border-radius: 8px; \
        transition: background-color 100ms ease; \
    } \
    .movement-card-header:focus-within { \
        background: alpha(@accent_color, 0.08); \
    } \
    .movement-name-entry { \
        font-weight: 800; \
        font-size: 1.18em; \
    } \
    .idea-number { \
        min-width: 1.7em; \
        min-height: 1.7em; \
        font-weight: 700; \
        font-size: 0.8em; \
        color: @accent_color; \
        opacity: 0.8; \
        background: alpha(@accent_color, 0.1); \
        border-radius: 999px; \
        padding: 0 4px; \
    } \
    .idea-bar { \
        background: @view_bg_color; \
        border: 1px solid alpha(@borders, 0.9); \
        border-radius: 14px; \
        padding: 5px 6px 5px 10px; \
        transition: border-color 100ms ease, background-color 100ms ease, box-shadow 100ms ease; \
    } \
    .idea-bar:hover { \
        border-color: alpha(@accent_color, 0.5); \
        background: alpha(@accent_color, 0.035); \
    } \
    .idea-bar:focus-within { \
        border-color: @accent_color; \
        box-shadow: 0 0 0 1px alpha(@accent_color, 0.35); \
    } \
    .idea-entry, \
    .movement-name-entry { \
        background: transparent; \
        border: none; \
        box-shadow: none; \
        padding: 4px 2px; \
    } \
    .idea-notes { \
        background: alpha(@window_fg_color, 0.04); \
        border: 1px solid alpha(@borders, 0.7); \
        border-radius: 10px; \
    } \
    .idea-notes text { \
        background: transparent; \
    } \
    .empty-movement-placeholder { \
        background: alpha(@window_fg_color, 0.025); \
        border: 1px dashed alpha(@borders, 1.0); \
        border-radius: 12px; \
        padding: 10px 12px; \
    } \
    .ghost-add-btn { \
        border: 1px dashed alpha(@borders, 1.0); \
        border-radius: 14px; \
        padding: 6px 14px; \
        opacity: 0.7; \
        transition: opacity 100ms ease, border-color 100ms ease, background-color 100ms ease; \
    } \
    .ghost-add-btn:hover { \
        opacity: 1.0; \
        border-color: alpha(@accent_color, 0.6); \
        background: alpha(@accent_color, 0.05); \
    } \
    .idea-tag-chip { \
        font-size: 0.72em; \
        font-weight: 600; \
        padding: 2px 9px; \
        border-radius: 999px; \
        min-height: 0; \
        min-width: 0; \
        margin-left: 2px; \
    } \
    .idea-tag-chip-idea { \
        background-color: @accent_bg_color; \
        color: @accent_fg_color; \
    } \
    .idea-tag-chip-idea:hover { \
        box-shadow: inset 0 0 0 1px alpha(@accent_fg_color, 0.35); \
    } \
    .idea-tag-chip-part { \
        background: alpha(@success_color, 0.24); \
        color: @success_color; \
    } \
    .idea-tag-chip-part:hover { \
        background: alpha(@success_color, 0.34); \
    } \
    .idea-tag-chip-empty { \
        background: transparent; \
        color: alpha(@window_fg_color, 0.35); \
        padding: 2px 5px; \
    } \
    .idea-tag-chip-empty:hover { \
        background: alpha(@window_fg_color, 0.1); \
        color: alpha(@window_fg_color, 0.75); \
    } \
    .drop-indicator { \
        min-height: 3px; \
        background: @accent_color; \
        border-radius: 2px; \
        margin: 2px 0; \
    } \
    .drop-indicator-new-movement { \
        min-height: 32px; \
        background: alpha(@accent_color, 0.12); \
        border: 1px dashed alpha(@accent_color, 0.6); \
        border-radius: 8px; \
    } \
    .dragging { \
        opacity: 0.4; \
    } \
    .idea-grabber, \
    .movement-grabber { \
        opacity: 0.5; \
        padding: 5px; \
        border-radius: 6px; \
        min-width: 18px; \
        min-height: 18px; \
        transition: opacity 100ms ease; \
    } \
    .idea-bar:hover .idea-grabber, \
    .movement-card-header:hover .movement-grabber { \
        opacity: 0.85; \
    } \
    .idea-grabber:hover, \
    .movement-grabber:hover { \
        opacity: 1.0; \
        background: alpha(@window_fg_color, 0.08); \
    } \
    .movement-idea-count-badge { \
        background: alpha(@window_fg_color, 0.08); \
        border-radius: 999px; \
        padding: 2px 9px; \
        margin: 0 2px; \
    } \
    .movement-header-icon { \
        opacity: 0.45; \
    } \
    .movement-header-icon:hover, \
    .movement-header-icon:checked { \
        opacity: 1.0; \
    } \
    .idea-delete { \
        opacity: 0.35; \
    } \
    .idea-bar:hover .idea-delete, \
    .idea-delete:hover { \
        opacity: 0.9; \
    } \
    .season-dot { \
        min-width: 9px; \
        min-height: 9px; \
        border-radius: 5px; \
        margin-top: 1px; \
    } \
    .preaching-view-window { \
        background: @window_bg_color; \
        color: @window_fg_color; \
    } \
    .preaching-view-window.warm { \
        background: #FBF6EB; \
        color: #2E2A22; \
    } \
    .preaching-season-bar { \
        min-height: 5px; \
        border-radius: 999px; \
        margin-bottom: 18px; \
    } \
    .preaching-title { \
        font-size: 2.6em; \
        font-weight: 900; \
    } \
    .preaching-date { \
        font-size: 1.2em; \
        opacity: 0.55; \
        margin-bottom: 8px; \
    } \
    .preaching-movement { \
        font-size: 1.5em; \
        font-weight: 800; \
        letter-spacing: 0.02em; \
        color: @accent_color; \
        border-top: 2px solid alpha(@accent_color, 0.25); \
        padding-top: 14px; \
    } \
    .preaching-idea-number { \
        font-size: 1.05em; \
        font-weight: 700; \
        opacity: 0.35; \
        margin-right: 10px; \
    } \
    .preaching-idea { \
        font-size: 2em; \
        font-weight: 500; \
    } \
    .preaching-notes { \
        font-size: 1.15em; \
        font-style: italic; \
        opacity: 0.6; \
        border-left: 3px solid alpha(@window_fg_color, 0.18); \
        padding-left: 12px; \
        margin-left: 2px; \
    } \
    .preaching-progress-dot { \
        min-width: 8px; \
        min-height: 8px; \
        border-radius: 999px; \
        background: alpha(@window_fg_color, 0.2); \
        margin: 0 3px; \
    } \
    .preaching-progress-dot-current { \
        background: @accent_color; \
        min-width: 10px; \
        min-height: 10px; \
    } \
    .preaching-overlay-btn { \
        opacity: 0.35; \
    } \
    .preaching-overlay-btn:hover { \
        opacity: 1.0; \
    } \
    .idea-row-selected .idea-bar { \
        background: alpha(@accent_color, 0.16); \
        border-radius: 6px; \
        border-left: 3px solid @accent_color; \
        padding-left: 7px; \
    } \
    .selection-rect { \
        background: alpha(@accent_color, 0.12); \
        border: 1px solid alpha(@accent_color, 0.6); \
        border-radius: 4px; \
    } \
    .idea-row-tag-dimmed { \
        opacity: 0.35; \
    } \
    .season-dot-6b21a8 { background-color: #6B21A8; } \
    .season-dot-b45309 { background-color: #B45309; } \
    .season-dot-15803d { background-color: #15803D; } \
    .season-dot-b91c1c { background-color: #B91C1C; } \
    .season-dot-111827 { background-color: #111827; }";

/// The RCL colour hexes (`rcl::COLOURS`) are a fixed liturgical palette, not
/// theme colours, so they're pre-registered here as static classes rather
/// than resolved at runtime like `ui::theme`'s named-colour lookups.
pub fn season_dot_class(hex: &str) -> &'static str {
    match hex {
        "#6B21A8" => "season-dot-6b21a8",
        "#B45309" => "season-dot-b45309",
        "#15803D" => "season-dot-15803d",
        "#B91C1C" => "season-dot-b91c1c",
        "#111827" => "season-dot-111827",
        _ => "season-dot-15803d",
    }
}

/// Loads all static, app-wide CSS once. Safe to call multiple times (GTK
/// dedupes identical providers by reference).
pub fn load_global_css() {
    let css = gtk4::CssProvider::new();
    css.load_from_data(GLOBAL_CSS);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
