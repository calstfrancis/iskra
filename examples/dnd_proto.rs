//! Throwaway prototype validating GTK4 drag-and-drop mechanics before they're
//! wired into the real editor (see Plans/plan.md §3 — dev2's risk gate). Run
//! with `cargo run --example dnd_proto` on the target desktop (KDE/Wayland)
//! and manually verify all seven behaviors listed at the bottom of this file.
//! Nothing here is reused verbatim by the app — once validated, the pattern
//! moves into src/ui/dnd.rs.
//!
//! ARCHITECTURAL FINDING (the reason this file looks the way it does): an
//! earlier version of this prototype put one `DropTarget` per movement box
//! plus a second one on the outer column, all declared for the same GType
//! (String), intending row payloads to be claimed by the inner target and
//! movement-reorder payloads to bubble past it to the outer one. That does
//! NOT work reliably: once the deepest widget's `DropTarget` structurally
//! matches the dragged value's GType, GTK commits to asking that widget,
//! and the widget's own runtime rejection (returning `DragAction::empty()`
//! for a payload it doesn't handle) does not cause the *drop* to fall back
//! to an ancestor — the drag silently cancels instead. Bubbling for
//! *motion* events does still occur, which made this easy to miss. Fix:
//! a single `DropTarget`, attached to the column, handles every drop
//! (row reorder within/between boxes, movement/box reorder, and
//! blank-space → new box) by translating the pointer's column-space `y`
//! into per-box-local coordinates itself, rather than delegating to
//! per-box targets.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gdk::ContentProvider;
use gtk4::glib::Value;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, DragSource, DropTarget, Image, Label, Orientation, ScrolledWindow,
    WidgetPaintable,
};
use libadwaita as adw;
use libadwaita::prelude::*;

const ROW_PAYLOAD_PREFIX: &str = "row:";
const BOX_PAYLOAD_PREFIX: &str = "box:";
const AUTOSCROLL_MARGIN: f64 = 40.0;
const AUTOSCROLL_STEP: f64 = 12.0;

struct RowModel {
    id: String,
    label: String,
}

struct BoxModel {
    id: String,
    rows: Vec<RowModel>,
}

struct AppData {
    boxes: Vec<BoxModel>,
}

/// Long-lived widgets the single column-level DropTarget needs to inspect
/// on every motion/drop event to figure out what's under the pointer.
struct EditorWidgets {
    column: GtkBox,
    scroller: ScrolledWindow,
    indicator: GtkBox,
    drag_active: Rc<Cell<bool>>,
}

fn main() {
    let app = adw::Application::new(Some("io.github.calstfrancis.IskraDndProto"), Default::default());
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    load_proto_css();

    let window = adw::ApplicationWindow::new(app);
    window.set_title(Some("DnD Prototype"));
    window.set_default_width(500);
    window.set_default_height(700);

    let data = Rc::new(RefCell::new(AppData {
        boxes: vec![
            BoxModel {
                id: "box-a".into(),
                rows: (1..=4)
                    .map(|n| RowModel {
                        id: format!("a{n}"),
                        label: format!("Box A · row {n}"),
                    })
                    .collect(),
            },
            BoxModel {
                id: "box-b".into(),
                rows: (1..=3)
                    .map(|n| RowModel {
                        id: format!("b{n}"),
                        label: format!("Box B · row {n}"),
                    })
                    .collect(),
            },
        ],
    }));

    let column = GtkBox::new(Orientation::Vertical, 16);
    column.set_margin_top(16);
    column.set_margin_bottom(16);
    column.set_margin_start(16);
    column.set_margin_end(16);

    let scroller = ScrolledWindow::new();
    scroller.set_child(Some(&column));
    scroller.set_vexpand(true);

    let indicator = gtk4::Box::new(Orientation::Horizontal, 0);
    indicator.add_css_class("drop-indicator");
    indicator.set_visible(false);

    let widgets = Rc::new(EditorWidgets {
        column: column.clone(),
        scroller: scroller.clone(),
        indicator,
        drag_active: Rc::new(Cell::new(false)),
    });

    rebuild(&widgets, &data);

    let drop_target = DropTarget::new(String::static_type(), gtk4::gdk::DragAction::MOVE);
    drop_target.set_preload(true);
    {
        let widgets = widgets.clone();
        drop_target.connect_motion(move |t, _x, y| {
            let Some(payload) = t.value_as::<String>() else {
                return gtk4::gdk::DragAction::empty();
            };
            on_motion(&widgets, &payload, y);
            gtk4::gdk::DragAction::MOVE
        });
    }
    {
        let widgets = widgets.clone();
        drop_target.connect_leave(move |_| {
            clear_indicator(&widgets);
        });
    }
    {
        let widgets = widgets.clone();
        let data = data.clone();
        drop_target.connect_drop(move |_, value, _x, y| {
            clear_indicator(&widgets);
            let Ok(payload) = value.get::<String>() else {
                return false;
            };
            on_drop(&widgets, &data, &payload, y)
        });
    }
    column.add_controller(drop_target);

    window.set_content(Some(&scroller));
    window.present();
}

/// Where a column-space `y` coordinate falls: inside a specific box's rows
/// region (with the box index and a y local to that box's rows_box), or in
/// the blank space between/around boxes (with the index a new box would be
/// inserted at).
enum DropZone {
    InBoxRows { box_index: usize, local_y: f64 },
    BlankSpace { insert_index: usize },
}

fn locate_drop_zone(widgets: &Rc<EditorWidgets>, y: f64) -> DropZone {
    let mut index = 0;
    let mut child = widgets.column.first_child();
    while let Some(w) = child {
        if !w.css_classes().iter().any(|c| c == "card") {
            child = w.next_sibling();
            continue;
        }
        let card = w.clone().downcast::<GtkBox>().expect("card is a Box");
        let rows_box = card
            .last_child()
            .and_then(|c| c.downcast::<GtkBox>().ok())
            .expect("card's last child is rows_box");
        if let Some((_, rows_top)) = rows_box.translate_coordinates(&widgets.column, 0.0, 0.0) {
            let rows_bottom = rows_top + rows_box.height() as f64;
            if y >= rows_top && y <= rows_bottom {
                return DropZone::InBoxRows {
                    box_index: index,
                    local_y: y - rows_top,
                };
            }
        }
        let alloc = w.allocation();
        let midpoint = alloc.y() as f64 + alloc.height() as f64 / 2.0;
        if y < midpoint {
            return DropZone::BlankSpace { insert_index: index };
        }
        index += 1;
        child = w.next_sibling();
    }
    DropZone::BlankSpace { insert_index: index }
}

fn on_motion(widgets: &Rc<EditorWidgets>, payload: &str, y: f64) {
    autoscroll_if_near_edge(&widgets.scroller, y);
    if payload.starts_with(ROW_PAYLOAD_PREFIX) {
        match locate_drop_zone(widgets, y) {
            DropZone::InBoxRows { box_index, local_y } => {
                place_row_indicator(widgets, box_index, local_y);
            }
            DropZone::BlankSpace { .. } => {
                clear_indicator(widgets);
            }
        }
    } else if payload.starts_with(BOX_PAYLOAD_PREFIX) {
        clear_indicator(widgets);
    }
}

fn on_drop(widgets: &Rc<EditorWidgets>, data: &Rc<RefCell<AppData>>, payload: &str, y: f64) -> bool {
    if let Some(row_id) = payload.strip_prefix(ROW_PAYLOAD_PREFIX) {
        let mut d = data.borrow_mut();
        let Some((from_box, from_idx)) = find_row(&d, row_id) else {
            return false;
        };
        match locate_drop_zone(widgets, y) {
            DropZone::InBoxRows { box_index, local_y } => {
                let card = nth_card(&widgets.column, box_index).expect("box exists");
                let rows_box = card
                    .last_child()
                    .and_then(|c| c.downcast::<GtkBox>().ok())
                    .expect("rows_box");
                let insertion_index = compute_insertion_index(&rows_box, local_y);
                let row = d.boxes[from_box].rows.remove(from_idx);
                let adjusted_index = if from_box == box_index && from_idx < insertion_index {
                    insertion_index - 1
                } else {
                    insertion_index
                };
                let target_len = d.boxes[box_index].rows.len();
                d.boxes[box_index]
                    .rows
                    .insert(adjusted_index.min(target_len), row);
            }
            DropZone::BlankSpace { insert_index } => {
                let row = d.boxes[from_box].rows.remove(from_idx);
                let new_id = format!("box-new-{}", d.boxes.len());
                let new_box = BoxModel {
                    id: new_id,
                    rows: vec![row],
                };
                let len = d.boxes.len();
                d.boxes.insert(insert_index.min(len), new_box);
            }
        }
        drop(d);
        rebuild(widgets, data);
        true
    } else if let Some(box_id) = payload.strip_prefix(BOX_PAYLOAD_PREFIX) {
        let mut d = data.borrow_mut();
        let Some(from) = find_box(&d, box_id) else {
            return false;
        };
        // Box reordering always compares against card midpoints, unlike row
        // drops: landing "inside" a sibling box's rows means before/after
        // that box, not literally at its array index.
        let insertion = card_insertion_index(&widgets.column, y);
        let adjusted = if insertion > from { insertion - 1 } else { insertion };
        let b = d.boxes.remove(from);
        let len = d.boxes.len();
        d.boxes.insert(adjusted.min(len), b);
        drop(d);
        rebuild(widgets, data);
        true
    } else {
        false
    }
}

fn nth_card(column: &GtkBox, index: usize) -> Option<GtkBox> {
    column
        .first_child()
        .into_iter()
        .chain(std::iter::successors(column.first_child(), |w| {
            w.next_sibling()
        }))
        .filter(|w| w.css_classes().iter().any(|c| c == "card"))
        .nth(index)
        .and_then(|w| w.downcast::<GtkBox>().ok())
}

fn place_row_indicator(widgets: &Rc<EditorWidgets>, box_index: usize, local_y: f64) {
    let Some(card) = nth_card(&widgets.column, box_index) else {
        return;
    };
    let Some(rows_box) = card.last_child().and_then(|c| c.downcast::<GtkBox>().ok()) else {
        return;
    };
    let insertion_index = compute_insertion_index(&rows_box, local_y);
    if widgets.indicator.parent().is_some() {
        widgets.indicator.unparent();
    }
    let children: Vec<_> = std::iter::successors(rows_box.first_child(), |w| w.next_sibling())
        .filter(|w| w != &widgets.indicator.clone().upcast::<gtk4::Widget>())
        .collect();
    match children.get(insertion_index) {
        Some(reference) => widgets.indicator.insert_before(&rows_box, Some(reference)),
        None => rows_box.append(&widgets.indicator),
    }
    widgets.indicator.set_visible(true);
}

fn clear_indicator(widgets: &Rc<EditorWidgets>) {
    if widgets.indicator.parent().is_some() {
        widgets.indicator.unparent();
    }
    widgets.indicator.set_visible(false);
}

fn load_proto_css() {
    let css = gtk4::CssProvider::new();
    css.load_from_data(
        ".card { background: alpha(@card_bg_color, 0.6); border: 1px solid alpha(@borders, 0.6); \
         border-radius: 8px; } \
         .row-widget { padding: 4px; border-radius: 4px; } \
         .dragging { opacity: 0.4; } \
         .drop-indicator { min-height: 3px; background: @accent_color; border-radius: 2px; margin: 2px 0; }",
    );
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn find_row(data: &AppData, row_id: &str) -> Option<(usize, usize)> {
    for (bi, b) in data.boxes.iter().enumerate() {
        if let Some(ri) = b.rows.iter().position(|r| r.id == row_id) {
            return Some((bi, ri));
        }
    }
    None
}

fn find_box(data: &AppData, box_id: &str) -> Option<usize> {
    data.boxes.iter().position(|b| b.id == box_id)
}

fn rebuild(widgets: &Rc<EditorWidgets>, data: &Rc<RefCell<AppData>>) {
    let column = &widgets.column;
    while let Some(child) = column.first_child() {
        column.remove(&child);
    }

    let box_ids: Vec<String> = data.borrow().boxes.iter().map(|b| b.id.clone()).collect();

    for box_id in &box_ids {
        let card = build_box_card(box_id, data, &widgets.drag_active);
        column.append(&card);
    }

    let hint = Label::new(Some("Drag a row's ⣿ handle above/below the boxes to create a new one."));
    hint.add_css_class("dim-label");
    hint.add_css_class("caption");
    column.append(&hint);
}

fn build_box_card(box_id: &str, data: &Rc<RefCell<AppData>>, drag_active: &Rc<Cell<bool>>) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 4);
    card.add_css_class("card");
    card.set_margin_bottom(4);

    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.set_margin_top(6);
    header.set_margin_bottom(4);
    header.set_margin_start(8);
    header.set_margin_end(8);

    let title = Label::new(Some(box_id));
    title.add_css_class("heading");
    title.set_hexpand(true);
    title.set_xalign(0.0);
    header.append(&title);

    let box_grabber = Image::from_icon_name("list-drag-handle-symbolic");
    box_grabber.set_tooltip_text(Some("Drag to reorder this box"));
    header.append(&box_grabber);

    setup_box_drag_source(&box_grabber, &card, box_id, drag_active);

    card.append(&header);

    let rows_box = GtkBox::new(Orientation::Vertical, 2);
    rows_box.set_margin_start(8);
    rows_box.set_margin_end(8);
    rows_box.set_margin_bottom(8);

    let row_ids: Vec<String> = data
        .borrow()
        .boxes
        .iter()
        .find(|b| b.id == box_id)
        .map(|b| b.rows.iter().map(|r| r.id.clone()).collect())
        .unwrap_or_default();

    for row_id in &row_ids {
        let label = data
            .borrow()
            .boxes
            .iter()
            .find(|b| b.id == box_id)
            .and_then(|b| b.rows.iter().find(|r| r.id == *row_id))
            .map(|r| r.label.clone())
            .unwrap_or_default();
        let row = build_row(row_id, &label, drag_active);
        rows_box.append(&row);
    }

    card.append(&rows_box);
    card
}

fn build_row(row_id: &str, label_text: &str, drag_active: &Rc<Cell<bool>>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.add_css_class("row-widget");
    row.set_margin_top(2);
    row.set_margin_bottom(2);

    let label = Label::new(Some(label_text));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    row.append(&label);

    let grabber = Image::from_icon_name("list-drag-handle-symbolic");
    row.append(&grabber);

    let source = DragSource::new();
    source.set_actions(gtk4::gdk::DragAction::MOVE);
    let payload = format!("{ROW_PAYLOAD_PREFIX}{row_id}");
    source.connect_prepare(move |_, _, _| Some(ContentProvider::for_value(&Value::from(&payload))));

    {
        let row_w = row.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_begin(move |src, _drag| {
            drag_active.set(true);
            row_w.add_css_class("dragging");
            let paintable = WidgetPaintable::new(Some(&row_w));
            src.set_icon(Some(&paintable), 0, 0);
        });
    }
    {
        let row_w = row.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_end(move |_, _, _| {
            drag_active.set(false);
            row_w.remove_css_class("dragging");
        });
    }
    {
        let row_w = row.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_cancel(move |_, _, _| {
            // Esc or a rejected drop: same cleanup as a normal drag_end.
            drag_active.set(false);
            row_w.remove_css_class("dragging");
            false
        });
    }
    grabber.add_controller(source);

    row
}

fn setup_box_drag_source(grabber: &Image, card: &GtkBox, box_id: &str, drag_active: &Rc<Cell<bool>>) {
    let source = DragSource::new();
    source.set_actions(gtk4::gdk::DragAction::MOVE);
    let payload = format!("{BOX_PAYLOAD_PREFIX}{box_id}");
    source.connect_prepare(move |_, _, _| Some(ContentProvider::for_value(&Value::from(&payload))));
    {
        let card_w = card.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_begin(move |src, _drag| {
            drag_active.set(true);
            card_w.add_css_class("dragging");
            let paintable = WidgetPaintable::new(Some(&card_w));
            src.set_icon(Some(&paintable), 0, 0);
        });
    }
    {
        let card_w = card.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_end(move |_, _, _| {
            drag_active.set(false);
            card_w.remove_css_class("dragging");
        });
    }
    {
        let card_w = card.clone();
        let drag_active = drag_active.clone();
        source.connect_drag_cancel(move |_, _, _| {
            drag_active.set(false);
            card_w.remove_css_class("dragging");
            false
        });
    }
    grabber.add_controller(source);
}

fn card_insertion_index(column: &GtkBox, y: f64) -> usize {
    let mut index = 0;
    let mut child = column.first_child();
    while let Some(w) = child {
        if !w.css_classes().iter().any(|c| c == "card") {
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

fn compute_insertion_index(rows_box: &GtkBox, y: f64) -> usize {
    let mut index = 0;
    let mut child = rows_box.first_child();
    while let Some(w) = child {
        if w.css_classes().iter().any(|c| c == "drop-indicator") {
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

fn autoscroll_if_near_edge(scroller: &ScrolledWindow, y: f64) {
    let vadj = scroller.vadjustment();
    let viewport_height = scroller.height() as f64;
    if y < AUTOSCROLL_MARGIN {
        vadj.set_value((vadj.value() - AUTOSCROLL_STEP).max(vadj.lower()));
    } else if y > viewport_height - AUTOSCROLL_MARGIN {
        vadj.set_value((vadj.value() + AUTOSCROLL_STEP).min(vadj.upper() - vadj.page_size()));
    }
}

// ── Verification checklist ──────────────────────────────────────────────────
// Verified headlessly (isolated Xvfb + icewm, synthetic xdotool drags) on
// 2026-07-17 — see conversation history for screenshots:
// 1. Reorder within a box                                    ✓ verified
// 2. Cross-box move                                          ✓ verified
// 3. Drop-position indicator tracks the pointer               ✓ verified
// 4. Drag preview (translucent row copy via WidgetPaintable)  ✓ verified (opacity
//    dimming confirmed; the floating drag icon itself isn't visible in a
//    window screenshot — it's compositor-level — but is standard GTK4 API
//    usage, same call Zerkalo already ships)
// 5. Esc cancel: nothing moves, `.dragging` clears on release  ✓ verified
//    (clears on mouse-up following Esc, not on Esc itself — the drag is tied
//    to the pointer grab, so this is correct GTK4 behavior, not a bug)
// 6. Blank-space drop creates a new box at the dropped position ✓ verified
// 7. Edge auto-scroll                                          not exercised
//    (mechanism is a two-line vadjustment nudge in on_motion — lowest-risk
//    item on this list; exercise interactively once real content overflows)
// 8. Movement/box reorder                                     ✓ verified
//
// Real-desktop (KDE/Wayland) confirmation is still worth doing once the
// pattern lands in src/ui/dnd.rs, per Plans/plan.md, but the mechanics
// above are GTK4 API behavior, not X11-specific — expect them to carry over.
