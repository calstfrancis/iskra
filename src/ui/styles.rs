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
        background: alpha(@window_fg_color, 0.12); \
        border-radius: 4px; \
        padding: 1px 6px; \
        font-size: 0.75em; \
    } \
    .sidebar-header { \
        font-size: 0.75em; \
        font-weight: bold; \
        color: alpha(@window_fg_color, 0.55); \
        padding: 8px 12px 2px 12px; \
    } \
    .movement-card { \
        background: alpha(@card_bg_color, 0.6); \
        border: 1px solid alpha(@borders, 0.6); \
        border-radius: 8px; \
        padding: 6px; \
    } \
    .movement-card-header { \
        padding: 4px 4px 4px 8px; \
    } \
    .idea-number { \
        min-width: 2.2em; \
        font-weight: 600; \
        color: alpha(@window_fg_color, 0.55); \
    } \
    .idea-entry, \
    .movement-name-entry { \
        background: transparent; \
        border: none; \
        box-shadow: none; \
        padding: 4px 2px; \
    } \
    .idea-entry:focus, \
    .movement-name-entry:focus { \
        background: alpha(@accent_color, 0.06); \
        border-radius: 4px; \
    } \
    .tag-tab { \
        font-size: 0.75em; \
        padding: 1px 8px; \
        border-radius: 0 0 6px 6px; \
        background: alpha(@window_fg_color, 0.08); \
        min-height: 0; \
    } \
    .tag-tab:hover { \
        background: alpha(@accent_color, 0.15); \
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
        opacity: 0.4; \
    } \
    .idea-grabber:hover, \
    .movement-grabber:hover { \
        opacity: 0.9; \
    } \
    .season-dot { \
        min-width: 9px; \
        min-height: 9px; \
        border-radius: 5px; \
        margin-top: 1px; \
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
