//! A single movement card: header (name, drag grabber, collapse triangle,
//! duplicate, delete) wrapping a `Revealer`-hidden ideas box. The ideas box
//! is identified by the `.movement-ideas-box` CSS class (see
//! `ui::dnd::movement_ideas_box`) so `dnd.rs` can locate it purely from the
//! widget tree without a second parallel bookkeeping structure.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Revealer, RevealerTransitionType, ToggleButton};

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
    is_first: bool,
    drag_active: &Rc<Cell<bool>>,
    on_rename: impl Fn(String) + 'static,
    on_rename_focus_out: impl Fn() + 'static,
    on_toggle_collapse: impl Fn() + 'static,
    on_delete: impl Fn() + 'static,
    on_duplicate: impl Fn() + 'static,
    on_merge_up: impl Fn() + 'static,
    on_move: impl Fn(i32) + 'static,
    on_split: impl Fn(usize) + 'static,
    on_marquee_select: impl Fn(Vec<String>, bool) + 'static,
    selected: Rc<RefCell<HashSet<String>>>,
    on_delete_selected: impl Fn() + 'static,
    on_copy_to_sermon: impl Fn() + 'static,
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

    // Collapsing hides the ideas box entirely — without some hint of what's
    // inside, a collapsed movement with real content looks the same as an
    // empty one. Only shown while collapsed (toggling always triggers a
    // full rebuild — see `Cmd::is_structural` — so this never needs to
    // change live within one card's lifetime).
    if movement.collapsed {
        let count = movement.ideas.len();
        let badge = Label::new(Some(&format!(
            "{count} idea{}",
            if count == 1 { "" } else { "s" }
        )));
        badge.add_css_class("dim-label");
        badge.add_css_class("caption");
        badge.add_css_class("movement-idea-count-badge");
        header.append(&badge);
    }

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

    let merge_up_btn = Button::from_icon_name("go-up-symbolic");
    merge_up_btn.add_css_class("flat");
    merge_up_btn.add_css_class("movement-header-icon");
    merge_up_btn.set_tooltip_text(Some("Merge with movement above"));
    merge_up_btn.set_sensitive(!is_first);
    header.append(&merge_up_btn);

    let copy_to_btn = Button::from_icon_name("send-to-symbolic");
    copy_to_btn.add_css_class("flat");
    copy_to_btn.add_css_class("movement-header-icon");
    copy_to_btn.set_tooltip_text(Some("Copy movement to another sermon…"));
    header.append(&copy_to_btn);

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

    {
        // Right-click on blank space in the ideas box (idea rows/buttons/
        // entries claim button-3 for their own handling — an `Entry`'s
        // built-in context menu, in particular — so this only ever fires
        // over the gaps between rows, never on a pill itself).
        let idea_count = movement.ideas.len();
        let ideas_box_for_gesture = ideas_box.clone();
        let on_split: Rc<dyn Fn(usize)> = Rc::new(on_split);
        let on_delete_selected: Rc<dyn Fn()> = Rc::new(on_delete_selected);
        let click = gtk4::GestureClick::new();
        click.set_button(gtk4::gdk::BUTTON_SECONDARY);
        click.connect_pressed(move |gesture, _n_press, x, y| {
            let selected_count = selected.borrow().len();
            if idea_count == 0 && selected_count == 0 {
                return;
            }
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            let split_at = dnd::idea_insertion_index(&ideas_box_for_gesture, y);

            let popover = gtk4::Popover::new();
            popover.set_parent(&ideas_box_for_gesture);
            popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.set_autohide(true);

            let menu = GtkBox::new(Orientation::Vertical, 0);
            if idea_count > 0 {
                let split_btn = Button::with_label("Split movement here");
                split_btn.add_css_class("flat");
                {
                    let popover_for_close = popover.clone();
                    let on_split = on_split.clone();
                    split_btn.connect_clicked(move |_| {
                        on_split(split_at);
                        popover_for_close.popdown();
                    });
                }
                menu.append(&split_btn);
            }
            if selected_count > 0 {
                let delete_btn = Button::with_label(&format!("Delete {selected_count} idea(s)"));
                delete_btn.add_css_class("flat");
                {
                    let popover_for_close = popover.clone();
                    let on_delete_selected = on_delete_selected.clone();
                    delete_btn.connect_clicked(move |_| {
                        on_delete_selected();
                        popover_for_close.popdown();
                    });
                }
                menu.append(&delete_btn);
            }
            popover.set_child(Some(&menu));
            {
                let popover_for_closed = popover.clone();
                popover.connect_closed(move |_| popover_for_closed.unparent());
            }
            popover.popup();
        });
        ideas_box.add_controller(click);
    }

    {
        // Rubber-band select: drag on blank space in the ideas box. The
        // press point is resolved with `pick()` and the gesture denied
        // outright unless it landed on the ideas box itself — the earlier
        // version relied on "child widgets claim their own clicks first,"
        // which is true of Entry/buttons/chips but *not* of the plain
        // `Image` drag handle or the `Label` idea number, neither of which
        // claims anything. A press on the handle therefore also started a
        // marquee, and since this gesture claimed at 4px while a
        // `DragSource` can't claim until GTK's larger system drag threshold,
        // the marquee reliably won and cancelled the drag — grabbing the
        // handle selected rows instead of moving one. Threshold also raised
        // above the system drag threshold so the two can never race again.
        let ideas_box_for_drag = ideas_box.clone();
        let start = Rc::new(Cell::new((0.0f64, 0.0f64)));
        let marquee_armed = Rc::new(Cell::new(false));
        let drag = gtk4::GestureDrag::new();
        drag.set_button(gtk4::gdk::BUTTON_PRIMARY);
        {
            let start = start.clone();
            let marquee_armed = marquee_armed.clone();
            let ideas_box_for_pick = ideas_box_for_drag.clone();
            drag.connect_drag_begin(move |gesture, x, y| {
                let on_blank = match ideas_box_for_pick.pick(x, y, gtk4::PickFlags::DEFAULT) {
                    Some(w) => {
                        w == ideas_box_for_pick.clone().upcast::<gtk4::Widget>()
                            || w.css_classes().iter().any(|c| c == "empty-movement-placeholder")
                    }
                    None => true,
                };
                marquee_armed.set(on_blank);
                if !on_blank {
                    gesture.set_state(gtk4::EventSequenceState::Denied);
                    return;
                }
                start.set((x, y));
            });
        }
        {
            let start = start.clone();
            let marquee_armed = marquee_armed.clone();
            let ideas_box_for_drag = ideas_box_for_drag.clone();
            drag.connect_drag_update(move |gesture, dx, dy| {
                if !marquee_armed.get() {
                    return;
                }
                if dx * dx + dy * dy < 100.0 {
                    return;
                }
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                let (sx, sy) = start.get();
                let (x0, x1) = (sx.min(sx + dx), sx.max(sx + dx));
                let (y0, y1) = (sy.min(sy + dy), sy.max(sy + dy));
                let mut child = ideas_box_for_drag.first_child();
                while let Some(w) = child {
                    if w.css_classes().iter().any(|c| c == "idea-row") {
                        let alloc = w.allocation();
                        let intersects = (alloc.x() as f64) < x1
                            && (alloc.x() as f64 + alloc.width() as f64) > x0
                            && (alloc.y() as f64) < y1
                            && (alloc.y() as f64 + alloc.height() as f64) > y0;
                        if intersects {
                            w.add_css_class("idea-row-selected");
                        } else {
                            w.remove_css_class("idea-row-selected");
                        }
                    }
                    child = w.next_sibling();
                }
            });
        }
        {
            let ideas_box_for_drag = ideas_box_for_drag.clone();
            let marquee_armed = marquee_armed.clone();
            drag.connect_drag_end(move |gesture, _dx, _dy| {
                if !marquee_armed.get() {
                    return;
                }
                marquee_armed.set(false);
                let mods = gesture.current_event_state();
                let ctrl = mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
                let mut ids = Vec::new();
                let mut child = ideas_box_for_drag.first_child();
                while let Some(w) = child {
                    if w.css_classes().iter().any(|c| c == "idea-row-selected") {
                        if let Some(id) = w.widget_name().strip_prefix("idea:") {
                            ids.push(id.to_string());
                        }
                    }
                    child = w.next_sibling();
                }
                on_marquee_select(ids, ctrl);
            });
        }
        ideas_box.add_controller(drag);
    }

    name_entry.connect_changed(move |e| on_rename(e.text().to_string()));
    {
        let focus_ctl = gtk4::EventControllerFocus::new();
        focus_ctl.connect_leave(move |_| on_rename_focus_out());
        name_entry.add_controller(focus_ctl);
    }
    {
        // Alt+Up/Alt+Down reorders this movement — a keyboard alternative
        // to dragging the grabber, which is fiddlier to land precisely.
        // Alt+Shift+Up/Down jumps straight to the top/bottom (see
        // `idea_row.rs`'s identical sentinel convention for `on_move`).
        let key_ctl = gtk4::EventControllerKey::new();
        key_ctl.connect_key_pressed(move |_, key, _, modifiers| {
            if !modifiers.contains(gtk4::gdk::ModifierType::ALT_MASK) {
                return glib::Propagation::Proceed;
            }
            let shift = modifiers.contains(gtk4::gdk::ModifierType::SHIFT_MASK);
            match key {
                gtk4::gdk::Key::Up => {
                    on_move(if shift { i32::MIN } else { -1 });
                    glib::Propagation::Stop
                }
                gtk4::gdk::Key::Down => {
                    on_move(if shift { i32::MAX } else { 1 });
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
    merge_up_btn.connect_clicked(move |_| on_merge_up());
    copy_to_btn.connect_clicked(move |_| on_copy_to_sermon());

    let payload = format!("{}{}", dnd::MOVEMENT_PAYLOAD_PREFIX, movement.id);
    dnd::setup_drag_source(&grabber, &root, move || payload.clone(), drag_active);

    MovementCardWidgets {
        root,
        ideas_box,
        name_entry,
        collapse_btn,
    }
}
