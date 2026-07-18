//! Drag-and-drop geometry and payload plumbing for the movements/ideas
//! editor. Validated in isolation first (see `examples/dnd_proto.rs`) before
//! landing here — read that file's module doc for the architectural finding
//! this design is built around: **one `DropTarget` on the movements column**,
//! not one per movement. GTK4 does not reliably bubble a rejected *drop*
//! (only rejected *motion*) from an inner target of the same GType to an
//! ancestor, so a single target that resolves what's under the pointer
//! itself is the robust shape, not a per-widget delegation chain.
//!
//! This module only computes geometry and manages the indicator widget; it
//! never touches `AppState` or emits `Cmd`s — callers (`editor.rs`) own the
//! model access and command dispatch, keeping this module reusable and unit
//! testable independent of GTK initialization where the math is concerned.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::gdk::ContentProvider;
use gtk4::glib::Value;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, DragSource, Image, Orientation, ScrolledWindow, WidgetPaintable};

pub const IDEA_PAYLOAD_PREFIX: &str = "idea:";
pub const MOVEMENT_PAYLOAD_PREFIX: &str = "movement:";
const AUTOSCROLL_MARGIN: f64 = 40.0;
const AUTOSCROLL_STEP: f64 = 12.0;

/// Wraps an idea/movement drag payload in its own `GType` rather than a plain
/// `String`. `GtkText` (which backs every `Entry`) registers its own built-in
/// drop target for plain-string/text content, and — since that target lives
/// on the Entry itself, deeper in the tree than our column-level
/// `DropTarget` — it wins whenever a payload's GType matches `String`,
/// silently inserting the dragged idea's raw payload text ("idea:<uuid>")
/// into whatever Entry the drop landed on instead of reordering. A distinct
/// boxed type means GtkText's text target never matches it, so the drop
/// always reaches our own handling.
#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "IskraDragPayload")]
pub struct DragPayload(pub String);

/// Where a movements-column-space `y` coordinate falls: inside a specific
/// movement's ideas region (with the movement's index and a y local to that
/// movement's ideas box), or in the blank space between/around movement
/// cards (with the index a new movement would be inserted at).
pub enum DropZone {
    InMovementIdeas { movement_index: usize, local_y: f64 },
    BlankSpace { insert_index: usize },
}

/// Locates the drop zone for `y` (in `movements_column`'s own coordinate
/// space) by walking its `.movement-card` children and translating each
/// card's full allocation (header *and* ideas box) into that space. Landing
/// anywhere on a movement's card — including its header/name row, not just
/// its ideas list — counts as a drop onto that movement: earlier logic only
/// treated the ideas-box region as "in movement" and fell through to
/// blank-space handling for the header band, which (since the header sits
/// above the ideas box, and blank-space compared `y` against the *card's*
/// midpoint) misread most header drops as "insert a new movement before this
/// one" instead of "add to this movement" — the bug where dragging an idea
/// onto a movement's title created a new movement.
pub fn locate_drop_zone(movements_column: &GtkBox, y: f64) -> DropZone {
    let mut index = 0;
    let mut child = movements_column.first_child();
    while let Some(w) = child {
        if !w.css_classes().iter().any(|c| c == "movement-card") {
            child = w.next_sibling();
            continue;
        }
        let card = w.clone().downcast::<GtkBox>().expect("movement-card is a Box");
        let alloc = w.allocation();
        if let Some((_, card_top)) = w.translate_coordinates(movements_column, 0.0, 0.0) {
            let card_bottom = card_top + alloc.height() as f64;
            if y >= card_top && y <= card_bottom {
                let local_y = movement_ideas_box(&card)
                    .filter(|ideas_box| ideas_box.height() > 0)
                    .and_then(|ideas_box| ideas_box.translate_coordinates(movements_column, 0.0, 0.0))
                    .map(|(_, top)| (y - top).max(0.0))
                    // Collapsed (zero-height ideas box) or unresolvable:
                    // always append, matching "collapsed movements still
                    // accept idea drops" from the mockup.
                    .unwrap_or(f64::MAX);
                return DropZone::InMovementIdeas {
                    movement_index: index,
                    local_y,
                };
            }
        }
        let midpoint = alloc.y() as f64 + alloc.height() as f64 / 2.0;
        if y < midpoint {
            return DropZone::BlankSpace { insert_index: index };
        }
        index += 1;
        child = w.next_sibling();
    }
    DropZone::BlankSpace { insert_index: index }
}

/// Insertion index among movement cards by comparing `y` against each
/// card's midpoint — the right measure for *reordering movements*, where
/// landing inside a sibling's ideas region means "before/after that
/// sibling," not "at that sibling's own index" (the latter is a no-op
/// whenever dragging movement 0 onto a point inside movement 1: remove-then-
/// reinsert at the same index round-trips to where it started).
pub fn movement_insertion_index(movements_column: &GtkBox, y: f64) -> usize {
    let mut index = 0;
    let mut child = movements_column.first_child();
    while let Some(w) = child {
        if !w.css_classes().iter().any(|c| c == "movement-card") {
            child = w.next_sibling();
            continue;
        }
        let alloc = w.allocation();
        let midpoint = alloc.y() as f64 + alloc.height() as f64 / 2.0;
        if y < midpoint {
            return index;
        }
        index += 1;
        child = w.next_sibling();
    }
    index
}

/// Insertion index among idea rows within a single movement's ideas box, by
/// comparing local `y` against each row's midpoint.
pub fn idea_insertion_index(ideas_box: &GtkBox, y: f64) -> usize {
    let mut index = 0;
    let mut child = ideas_box.first_child();
    while let Some(w) = child {
        if w
            .css_classes()
            .iter()
            .any(|c| c == "drop-indicator" || c == "empty-movement-placeholder")
        {
            child = w.next_sibling();
            continue;
        }
        let alloc = w.allocation();
        let midpoint = alloc.y() as f64 + alloc.height() as f64 / 2.0;
        if y < midpoint {
            return index;
        }
        index += 1;
        child = w.next_sibling();
    }
    index
}

pub fn nth_movement_card(movements_column: &GtkBox, index: usize) -> Option<GtkBox> {
    std::iter::successors(movements_column.first_child(), |w| w.next_sibling())
        .filter(|w| w.css_classes().iter().any(|c| c == "movement-card"))
        .nth(index)
        .and_then(|w| w.downcast::<GtkBox>().ok())
}

/// A movement card's ideas box is identified by CSS class, not position,
/// since a card's header row also lives in the same container.
pub fn movement_ideas_box(card: &GtkBox) -> Option<GtkBox> {
    std::iter::successors(card.first_child(), |w| w.next_sibling())
        .find(|w| w.css_classes().iter().any(|c| c == "movement-ideas-box"))
        .and_then(|w| w.downcast::<GtkBox>().ok())
}

pub fn place_idea_indicator(indicator: &GtkBox, movements_column: &GtkBox, movement_index: usize, local_y: f64) {
    let Some(card) = nth_movement_card(movements_column, movement_index) else {
        return;
    };
    let Some(ideas_box) = movement_ideas_box(&card) else {
        return;
    };
    let insertion_index = idea_insertion_index(&ideas_box, local_y);
    if indicator.parent().is_some() {
        indicator.unparent();
    }
    let children: Vec<_> = std::iter::successors(ideas_box.first_child(), |w| w.next_sibling())
        .filter(|w| w != &indicator.clone().upcast::<gtk4::Widget>())
        .filter(|w| !w.css_classes().iter().any(|c| c == "empty-movement-placeholder"))
        .collect();
    match children.get(insertion_index) {
        Some(reference) => indicator.insert_before(&ideas_box, Some(reference)),
        None => ideas_box.append(indicator),
    }
    indicator.set_visible(true);
}

pub fn clear_indicator(indicator: &GtkBox) {
    if indicator.parent().is_some() {
        indicator.unparent();
    }
    indicator.set_visible(false);
}

pub fn new_drop_indicator() -> GtkBox {
    let indicator = GtkBox::new(Orientation::Horizontal, 0);
    indicator.add_css_class("drop-indicator");
    indicator.set_visible(false);
    indicator
}

pub fn autoscroll_if_near_edge(scroller: &ScrolledWindow, y: f64) {
    let vadj = scroller.vadjustment();
    let viewport_height = scroller.height() as f64;
    if y < AUTOSCROLL_MARGIN {
        vadj.set_value((vadj.value() - AUTOSCROLL_STEP).max(vadj.lower()));
    } else if y > viewport_height - AUTOSCROLL_MARGIN {
        vadj.set_value((vadj.value() + AUTOSCROLL_STEP).min(vadj.upper() - vadj.page_size()));
    }
}

/// Wires a `DragSource` onto `grabber` carrying `payload` (already prefixed
/// with [`IDEA_PAYLOAD_PREFIX`] or [`MOVEMENT_PAYLOAD_PREFIX`]), showing
/// `preview_widget` as the drag icon and toggling `.dragging` on it for the
/// gesture's duration. `drag_active` is set for the duration of the drag so
/// callers can defer any rebuild that would destroy the widget GTK's drag
/// machinery still holds a reference to — a real GTK4 crash class otherwise.
pub fn setup_drag_source(
    grabber: &impl IsA<gtk4::Widget>,
    preview_widget: &impl IsA<gtk4::Widget>,
    payload: String,
    drag_active: &Rc<Cell<bool>>,
) {
    let source = DragSource::new();
    source.set_actions(gtk4::gdk::DragAction::MOVE);
    source.connect_prepare(move |_, _, _| {
        Some(ContentProvider::for_value(&Value::from(&DragPayload(
            payload.clone(),
        ))))
    });

    let preview = preview_widget.clone().upcast::<gtk4::Widget>();
    {
        let preview = preview.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_begin(move |src, _drag| {
            drag_active.set(true);
            preview.add_css_class("dragging");
            let paintable = WidgetPaintable::new(Some(&preview));
            src.set_icon(Some(&paintable), 0, 0);
        });
    }
    {
        let preview = preview.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_end(move |_, _, _| {
            drag_active.set(false);
            preview.remove_css_class("dragging");
        });
    }
    {
        let drag_active = drag_active.clone();
        source.connect_drag_cancel(move |_, _, _| {
            // Esc or a rejected drop: same cleanup as a normal drag_end. The
            // class actually clears on button-release, not on Esc itself —
            // the drag is tied to the pointer grab, confirmed correct GTK4
            // behavior in the prototype, not a bug to work around.
            drag_active.set(false);
            preview.remove_css_class("dragging");
            false
        });
    }
    grabber.as_ref().add_controller(source);
}

pub fn drag_grabber(tooltip: &str) -> Image {
    let grabber = Image::from_icon_name("list-drag-handle-symbolic");
    grabber.add_css_class("idea-grabber");
    grabber.set_tooltip_text(Some(tooltip));
    grabber
}
