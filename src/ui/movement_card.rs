//! A single movement card: header (name, drag grabber, collapse triangle)
//! wrapping a `Revealer`-hidden ideas box. The ideas box is identified by
//! the `.movement-ideas-box` CSS class (see `ui::dnd::movement_ideas_box`)
//! so `dnd.rs` can locate it purely from the widget tree without a second
//! parallel bookkeeping structure.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Revealer, RevealerTransitionType, ToggleButton};

use crate::model::Movement;
use crate::ui::dnd;

pub struct MovementCardWidgets {
    pub root: GtkBox,
    pub ideas_box: GtkBox,
    pub name_entry: gtk4::Entry,
    pub collapse_btn: ToggleButton,
}

pub fn build_movement_card(
    movement: &Movement,
    drag_active: &Rc<Cell<bool>>,
    on_rename: impl Fn(String) + 'static,
    on_rename_focus_out: impl Fn() + 'static,
    on_toggle_collapse: impl Fn() + 'static,
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
    collapse_btn.set_tooltip_text(Some("Collapse movement"));
    header.append(&collapse_btn);

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
    // The button's own icon/Revealer state isn't flipped here: toggling is
    // structural (see Cmd::is_structural), so `on_toggle_collapse` applies
    // the command and the ensuing full rebuild constructs a fresh card
    // whose initial state already matches the model — flipping this one's
    // state too would just be redundant work on a widget about to be torn
    // down.
    collapse_btn.connect_toggled(move |_| on_toggle_collapse());

    let payload = format!("{}{}", dnd::MOVEMENT_PAYLOAD_PREFIX, movement.id);
    dnd::setup_drag_source(&grabber, &root, payload, drag_active);

    MovementCardWidgets {
        root,
        ideas_box,
        name_entry,
        collapse_btn,
    }
}
