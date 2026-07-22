//! Changelog viewer, opened from the status bar's version button. Renders
//! `CHANGELOG.md` live (`include_str!`, parsed at runtime) rather than
//! maintaining a separate hand-written "what's new" list, so it can never
//! drift from the actual file. Adapted from Zerkalo's `show_changelog`.

use glib::IsA;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;

const CHANGELOG: &str = include_str!("../../CHANGELOG.md");

/// The current version's entry, flattened to `"Section: bullet"` strings for
/// the welcome window's What's New list. Falls back to the topmost entry when
/// no heading matches — dev builds update one entry in place (per CLAUDE.md),
/// so `0.4.0-dev3`'s version can outrun a heading still reading `0.4.0-dev2`.
pub fn whats_new_bullets(version: &str) -> Vec<String> {
    let mut best: Vec<String> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut section = String::new();
    let mut in_entry = false;
    let mut matched = false;

    for line in CHANGELOG.lines() {
        let trimmed = line.trim();
        if let Some(inner) = trimmed.strip_prefix("## [") {
            if in_entry {
                if matched {
                    return current;
                }
                if best.is_empty() {
                    best = std::mem::take(&mut current);
                }
            }
            current = Vec::new();
            section.clear();
            in_entry = true;
            matched = inner.split_once(']').map(|(v, _)| v) == Some(version);
        } else if let Some(text) = trimmed.strip_prefix("### ") {
            section = text.to_string();
        } else if let Some(content) = trimmed.strip_prefix("- ") {
            if in_entry {
                current.push(if section.is_empty() {
                    content.to_string()
                } else {
                    format!("{section}: {content}")
                });
            }
        }
    }

    if matched {
        return current;
    }
    if best.is_empty() {
        best = current;
    }
    best
}

pub fn show_changelog(parent: &impl IsA<gtk4::Window>) {
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    let win = adw::Window::new();
    win.set_title(Some("Changelog — Iskra"));
    win.set_default_width(680);
    win.set_default_height(640);
    win.set_transient_for(Some(parent));
    win.set_modal(false);

    let header = adw::HeaderBar::new();
    let title_widget = adw::WindowTitle::new("Changelog", &format!("You're on v{CURRENT_VERSION}"));
    header.set_title_widget(Some(&title_widget));

    let body = GtkBox::new(Orientation::Vertical, 4);
    body.set_margin_start(24);
    body.set_margin_end(24);
    body.set_margin_top(16);
    body.set_margin_bottom(24);

    let mut first_heading = true;
    for line in CHANGELOG.lines() {
        let trimmed = line.trim();
        if let Some(inner) = trimmed.strip_prefix("## [") {
            let (version, rest) = match inner.split_once(']') {
                Some((v, r)) => (v, r.trim()),
                None => (inner.trim_end_matches(']'), ""),
            };
            let title = rest.strip_prefix("— ").unwrap_or(rest);

            let heading_row = GtkBox::new(Orientation::Horizontal, 8);
            heading_row.set_margin_top(if first_heading { 0 } else { 22 });
            first_heading = false;

            let ver_lbl = Label::new(Some(version));
            ver_lbl.add_css_class("monospace");
            ver_lbl.add_css_class("dim-label");
            ver_lbl.add_css_class("caption-heading");
            ver_lbl.set_xalign(0.0);
            heading_row.append(&ver_lbl);

            if version == CURRENT_VERSION {
                let badge = Label::new(Some("· Current"));
                badge.add_css_class("caption-heading");
                badge.add_css_class("accent");
                heading_row.append(&badge);
            }
            body.append(&heading_row);

            if !title.is_empty() {
                let title_lbl = Label::new(Some(title));
                title_lbl.add_css_class("title-3");
                title_lbl.set_xalign(0.0);
                title_lbl.set_wrap(true);
                title_lbl.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
                title_lbl.set_margin_bottom(2);
                body.append(&title_lbl);
            }
        } else if let Some(text) = trimmed.strip_prefix("### ") {
            let lbl = Label::new(Some(text));
            lbl.add_css_class("heading");
            lbl.set_xalign(0.0);
            lbl.set_margin_top(8);
            lbl.set_margin_start(4);
            lbl.set_margin_bottom(2);
            lbl.set_wrap(true);
            body.append(&lbl);
        } else if let Some(content) = trimmed.strip_prefix("- ") {
            body.append(&changelog_bullet(content));
        }
    }

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
    let clamp = adw::Clamp::new();
    clamp.set_maximum_size(660);
    clamp.set_child(Some(&body));
    scroll.set_child(Some(&clamp));

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&scroll));
    win.set_content(Some(&toolbar));
    win.present();
}

fn changelog_bullet(text: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.set_margin_start(8);
    let dot = Label::new(Some("•"));
    dot.set_valign(gtk4::Align::Start);
    dot.add_css_class("dim-label");
    dot.set_margin_top(1);

    let markup = md_inline_to_pango(text);
    let lbl = Label::new(None);
    lbl.set_markup(&markup);
    lbl.set_xalign(0.0);
    lbl.set_wrap(true);
    lbl.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    lbl.set_hexpand(true);
    lbl.set_halign(gtk4::Align::Fill);

    row.append(&dot);
    row.append(&lbl);
    row
}

pub fn md_inline_to_pango(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                out.push_str("<b>");
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some('*') if chars.peek() == Some(&'*') => {
                            chars.next();
                            break;
                        }
                        Some(ch) => inner.push(ch),
                        None => break,
                    }
                }
                out.push_str(&glib::markup_escape_text(&inner));
                out.push_str("</b>");
            }
            '`' => {
                out.push_str("<tt>");
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some('`') => break,
                        Some(ch) => inner.push(ch),
                        None => break,
                    }
                }
                out.push_str(&glib::markup_escape_text(&inner));
                out.push_str("</tt>");
            }
            '[' => {
                let mut link_text = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(ch) => link_text.push(ch),
                        None => break,
                    }
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    loop {
                        match chars.next() {
                            Some(')') | None => break,
                            _ => {}
                        }
                    }
                }
                out.push_str(&glib::markup_escape_text(&link_text));
            }
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whats_new_prefixes_bullets_with_their_section() {
        let bullets = whats_new_bullets(env!("CARGO_PKG_VERSION"));
        assert!(!bullets.is_empty());
        assert!(bullets.iter().all(|b| b.contains(": ")));
    }

    #[test]
    fn whats_new_falls_back_to_topmost_entry_for_an_unlisted_version() {
        let top = whats_new_bullets("0.0.0-nonexistent");
        let first_heading = CHANGELOG
            .lines()
            .find_map(|l| l.trim().strip_prefix("## ["))
            .and_then(|i| i.split_once(']'))
            .map(|(v, _)| v.to_string())
            .expect("CHANGELOG has at least one version heading");
        assert_eq!(top, whats_new_bullets(&first_heading));
    }

    #[test]
    fn whats_new_stops_at_the_next_version_heading() {
        let mut headings = CHANGELOG
            .lines()
            .filter_map(|l| l.trim().strip_prefix("## ["))
            .filter_map(|i| i.split_once(']'))
            .map(|(v, _)| v.to_string());
        let first = headings.next().expect("at least one heading");
        let second = headings.next().expect("at least two headings");
        let a = whats_new_bullets(&first);
        let b = whats_new_bullets(&second);
        assert!(!a.is_empty() && !b.is_empty());
        assert!(a.iter().all(|x| !b.contains(x)) || a != b);
        assert_ne!(a, b);
    }
}
