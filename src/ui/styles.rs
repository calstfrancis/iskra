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
        border-radius: 4px; \
        padding: 1px 6px; \
        font-size: 0.75em; \
        font-weight: 600; \
    } \
    .tag-chip-s { \
        background-color: @accent_bg_color; \
        color: @accent_fg_color; \
    } \
    .tag-chip-t { \
        background: alpha(@success_color, 0.24); \
        color: @success_color; \
    } \
    .sidebar-header { \
        font-size: 0.75em; \
        font-weight: bold; \
        color: alpha(@window_fg_color, 0.55); \
        padding: 8px 12px 2px 12px; \
    } \
    .movement-card { \
        background: alpha(@window_fg_color, 0.035); \
        border: 1px solid alpha(@borders, 0.9); \
        border-radius: 12px; \
        padding: 8px; \
        box-shadow: 0 1px 3px alpha(black, 0.08); \
    } \
    .movement-card-header { \
        padding: 4px 4px 4px 8px; \
    } \
    .movement-name-entry { \
        font-weight: 700; \
        font-size: 1.05em; \
    } \
    .idea-number { \
        min-width: 1.7em; \
        min-height: 1.7em; \
        font-weight: 700; \
        font-size: 0.85em; \
        color: @accent_color; \
        background: alpha(@accent_color, 0.14); \
        border-radius: 999px; \
        padding: 0 4px; \
    } \
    .idea-bar { \
        background: @view_bg_color; \
        border: 1px solid alpha(@borders, 0.9); \
        border-radius: 14px; \
        padding: 5px 6px 5px 10px; \
        transition: border-color 100ms ease, box-shadow 100ms ease; \
    } \
    .idea-bar:hover { \
        border-color: alpha(@accent_color, 0.5); \
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
        border: 1px dashed alpha(@borders, 1.0); \
        border-radius: 12px; \
        padding: 10px 12px; \
    } \
    .tag-tab { \
        font-size: 0.75em; \
        font-weight: 600; \
        padding: 2px 9px; \
        border-radius: 0 0 8px 8px; \
        min-height: 0; \
        box-shadow: 0 1px 2px alpha(black, 0.06); \
    } \
    .tag-tab-idea { \
        background-color: @accent_bg_color; \
        color: @accent_fg_color; \
    } \
    .tag-tab-idea:hover { \
        box-shadow: 0 1px 2px alpha(black, 0.06), inset 0 0 0 1px alpha(@accent_fg_color, 0.35); \
    } \
    .tag-tab-part { \
        background: alpha(@success_color, 0.24); \
        color: @success_color; \
    } \
    .tag-tab-part:hover { \
        background: alpha(@success_color, 0.34); \
    } \
    .tag-tab-idea.tag-tab-empty, \
    .tag-tab-part.tag-tab-empty { \
        background: alpha(@window_fg_color, 0.08); \
        color: alpha(@window_fg_color, 0.5); \
        font-weight: 500; \
        box-shadow: none; \
    } \
    .tag-tab-idea.tag-tab-empty:hover, \
    .tag-tab-part.tag-tab-empty:hover { \
        background: alpha(@window_fg_color, 0.14); \
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
        padding: 5px; \
        border-radius: 6px; \
        min-width: 18px; \
        min-height: 18px; \
    } \
    .idea-grabber:hover, \
    .movement-grabber:hover { \
        opacity: 1.0; \
        background: alpha(@window_fg_color, 0.08); \
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
