//! Shared helpers for deriving colors from the active libadwaita theme,
//! for the handful of spots (Pango markup, TextTag properties) that can't
//! reference GTK CSS named colors (`@accent_color`, etc.) directly.
//!
//! Copied from zerkalo/src/ui/theme.rs — app-agnostic.

use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use libadwaita as adw;

pub fn is_dark() -> bool {
    adw::StyleManager::default().is_dark()
}

pub fn rgba_to_hex(c: &RGBA) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.red() * 255.0).round() as u8,
        (c.green() * 255.0).round() as u8,
        (c.blue() * 255.0).round() as u8
    )
}

/// Resolves a GTK named color (e.g. "error_color", "accent_color") on the given
/// widget's style context to a solid hex string, falling back if unresolved.
pub fn lookup_color_hex(widget: &impl IsA<gtk4::Widget>, name: &str, fallback: &str) -> String {
    widget
        .as_ref()
        .style_context()
        .lookup_color(name)
        .map(|c| rgba_to_hex(&c))
        .unwrap_or_else(|| fallback.to_string())
}

/// Blends window_fg_color into window_bg_color to approximate the "dim-label"
/// muted foreground as a solid hex, since Pango markup can't apply CSS alpha.
#[allow(dead_code)]
pub fn muted_fg_hex(widget: &impl IsA<gtk4::Widget>) -> String {
    let ctx = widget.as_ref().style_context();
    match (
        ctx.lookup_color("window_fg_color"),
        ctx.lookup_color("window_bg_color"),
    ) {
        (Some(fg), Some(bg)) => {
            let a = 0.6f32;
            let blend = |f: f32, b: f32| f * a + b * (1.0 - a);
            format!(
                "#{:02x}{:02x}{:02x}",
                (blend(fg.red(), bg.red()) * 255.0).round() as u8,
                (blend(fg.green(), bg.green()) * 255.0).round() as u8,
                (blend(fg.blue(), bg.blue()) * 255.0).round() as u8
            )
        }
        _ => "#888888".to_string(),
    }
}
