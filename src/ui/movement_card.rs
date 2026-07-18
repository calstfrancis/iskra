//! A single movement card: header (name, drag grabber, collapse triangle,
//! duplicate, delete) wrapping a `Revealer`-hidden ideas box. The ideas box
//! is identified by the `.movement-ideas-box` CSS class (see
//! `ui::dnd::movement_ideas_box`) so `dnd.rs` can locate it purely from the
//! widget tree without a second parallel bookkeeping structure.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Orientation, Revealer, RevealerTransitionType, ToggleButton};

use crate::model::Movement;
use crate::ui::dnd;

pub struct MovementCardWidgets {
    pub root: GtkBox,
    pub ideas_box: GtkBox,
    pub name_entry: gtk4::Entry,
    pub collapse_btn: ToggleButton,
}

#[allow(clippy::too_many_arguments)]
pub fn build_movement_card(
    movement: &Movement,
    drag_active: &Rc<Cell<bool>>,
    on_rename: impl Fn(String) + 'static,
    on_rename_focus_out: impl Fn() + 'static,
    on_toggle_collapse: impl Fn() + 'static,
    on_delete: impl Fn() + 'static,
    on_duplicate: impl Fn() + 'static,
    on_move: impl Fn(i32) + 'static,
) -> MovementCardWidgets {
    let root = GtkBox::new(Orientation::Vertical, 2);
    root.add_css_class("movement-card");

    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("movement-card-header");

    let name_entry = gtk4::Entry::new();
    name_entry.set_has_frame(false);
    name_entry.add_css_class("movement-name-entry");
    name_entry.set_hexpand(true);
    name_entry.set_text(&movement.name);
    header.append(&name_entry);

    let collapse_btn = ToggleButton::new();
    collapse_btn.set_icon_name(if movement.collapsed {
        "pan-end-symbolic"
    } else {
        "pan-down-symbolic"
    });
    collapse_btn.add_css_class("flat");
    collapse_btn.add_css_class("movement-header-icon");
    collapse_btn.set_active(movement.collapsed);
    collapse_btn.set_tooltip_text(Some(if movement.collapsed {
        "Expand movement"
    } else {
        "Collapse movement"
    }));
    header.append(&collapse_btn);

    let duplicate_btn = Button::from_icon_name("edit-copy-symbolic");
    duplicate_btn.add_css_class("flat");
    duplicate_btn.add_css_class("movement-header-icon");
    duplicate_btn.set_tooltip_text(Some("Duplicate movement"));
    header.append(&duplicate_btn);

    let delete_btn = Button::from_icon_name("user-trash-symbolic");
    delete_btn.add_css_class("flat");
    delete_btn.add_css_class("idea-delete");
    delete_btn.set_tooltip_text(Some("Delete movement (Ctrl+Z to undo)"));
    header.append(&delete_btn);

    let grabber = dnd::drag_grabber("Drag to reorder movement");
    grabber.remove_css_class("idea-grabber");
    grabber.add_css_class("movement-grabber");
    header.append(&grabber);

    root.append(&header);

    let ideas_box = GtkBox::new(Orientation::Vertical, 10);
    ideas_box.add_css_class("movement-ideas-box");
    ideas_box.set_margin_start(12);
    ideas_box.set_margin_end(12);
    ideas_box.set_margin_bottom(8);

    let revealer = Revealer::new();
    revealer.set_transition_type(RevealerTransitionType::SlideDown);
    revealer.set_child(Some(&ideas_box));
    revealer.set_reveal_child(!movement.collapsed);
    root.append(&revealer);

    name_entry.connect_changed(move |e| on_rename(e.text().to_string()));
    {
        let focus_ctl = gtk4::EventControllerFocus::new();
        focus_ctl.connect_leave(move |_| on_rename_focus_out());
        name_entry.add_controller(focus_ctl);
    }
    {
        // Alt+Up/Alt+Down reorders this movement — a keyboard alternative
        // to dragging the grabber, which is fiddlier to land precisely.
        let key_ctl = gtk4::EventControllerKey::new();
        key_ctl.connect_key_pressed(move |_, key, _, modifiers| {
            if !modifiers.contains(gtk4::gdk::ModifierType::ALT_MASK) {
                return glib::Propagation::Proceed;
            }
            match key {
                gtk4::gdk::Key::Up => {
                    on_move(-1);
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Down => {
                    on_move(1);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        name_entry.add_controller(key_ctl);
    }
    // The button's own icon/Revealer state isn't flipped here: toggling is
    // structural (see Cmd::is_structural), so `on_toggle_collapse` applies
    // the command and the ensuing full rebuild constructs a fresh card
    // whose initial state already matches the model — flipping this one's
    // state too would just be redundant work on a widget about to be torn
    // down.
    collapse_btn.connect_toggled(move |_| on_toggle_collapse());
    duplicate_btn.connect_clicked(move |_| on_duplicate());
    delete_btn.connect_clicked(move |_| on_delete());

    let payload = format!("{}{}", dnd::MOVEMENT_PAYLOAD_PREFIX, movement.id);
    dnd::setup_drag_source(&grabber, &root, payload, drag_active);

    MovementCardWidgets {
        root,
        ideas_box,
        name_entry,
        collapse_btn,
    }
}
