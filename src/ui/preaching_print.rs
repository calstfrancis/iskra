//! Printing for Preaching View — a Cairo/Pango render path entirely separate
//! from the on-screen fullscreen display (`preaching_view.rs`): a printed
//! page wants plain black-on-white, page-aware breaks, and a running
//! header/page number, none of which the screen view needs or has. Uses
//! GTK's native `PrintOperation` (system print dialog, printer selection,
//! and — depending on the print backend — its own preview) rather than a
//! custom preview widget, and injects an "Iskra" tab into that same native
//! dialog (`custom_tab_label`/`create_custom_widget`) for the notes/tags/
//! font-size options, instead of a separate pre-print dialog.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{pango, Box as GtkBox, CheckButton, Label, Orientation, SpinButton};

use crate::model::Sermon;
use crate::state::AppState;

const PAGE_MARGIN_PT: f64 = 54.0;
const HEADER_HEIGHT_PT: f64 = 22.0;
const FOOTER_HEIGHT_PT: f64 = 20.0;
/// If placing a movement heading would leave less than this much room
/// afterward on the page, push it to the next page instead — otherwise a
/// heading can land as the very last line on a page with none of its own
/// ideas following it until the next sheet, which reads badly at the pulpit.
const MIN_ORPHAN_SPACE_PT: f64 = 60.0;

#[derive(Clone)]
struct PrintOptions {
    include_notes: bool,
    include_tags: bool,
    font_pt: f64,
}

#[derive(Clone)]
enum Block {
    Title(String),
    Date(String),
    Movement(String),
    IdeaTags(String),
    IdeaText(String),
    IdeaNotes(String),
    Bibliography,
    BibliographyEntry(String),
}

struct Measured {
    block: Block,
    top_margin: f64,
    height: f64,
}

pub fn print_sermon(parent: &impl IsA<gtk4::Window>, state: &Rc<RefCell<AppState>>) {
    let sermon = state.borrow().sermon.clone();
    let title = sermon.display_title().to_string();

    let op = gtk4::PrintOperation::new();
    op.set_job_name(&title);
    op.set_unit(gtk4::Unit::Points);
    op.set_custom_tab_label(Some("Iskra"));

    let options = Rc::new(RefCell::new(PrintOptions {
        include_notes: state.borrow().config.print_include_notes,
        include_tags: state.borrow().config.print_include_tags,
        font_pt: state.borrow().config.print_font_pt,
    }));

    let notes_check = CheckButton::with_label("Include notes");
    notes_check.set_active(options.borrow().include_notes);
    let tags_check = CheckButton::with_label("Include idea/part tags");
    tags_check.set_active(options.borrow().include_tags);
    let font_spin = SpinButton::with_range(9.0, 24.0, 0.5);
    font_spin.set_value(options.borrow().font_pt);

    {
        let notes_check = notes_check.clone();
        let tags_check = tags_check.clone();
        let font_spin = font_spin.clone();
        op.connect_create_custom_widget(move |_| {
            let col = GtkBox::new(Orientation::Vertical, 10);
            col.set_margin_top(12);
            col.set_margin_bottom(12);
            col.set_margin_start(12);
            col.set_margin_end(12);
            col.append(&notes_check);
            col.append(&tags_check);
            let font_row = GtkBox::new(Orientation::Horizontal, 8);
            font_row.append(&Label::new(Some("Font size (pt)")));
            font_row.append(&font_spin);
            col.append(&font_row);
            Some(col.upcast::<glib::Object>())
        });
    }
    {
        let options = options.clone();
        let notes_check = notes_check.clone();
        let tags_check = tags_check.clone();
        let font_spin = font_spin.clone();
        op.connect_custom_widget_apply(move |_, _widget| {
            let mut opts = options.borrow_mut();
            opts.include_notes = notes_check.is_active();
            opts.include_tags = tags_check.is_active();
            opts.font_pt = font_spin.value();
        });
    }

    let pages: Rc<RefCell<Vec<Vec<Measured>>>> = Rc::new(RefCell::new(Vec::new()));

    {
        let sermon = sermon.clone();
        let options = options.clone();
        let pages = pages.clone();
        op.connect_begin_print(move |op, context| {
            let built = paginate(&sermon, context, &options.borrow());
            op.set_n_pages(built.len().max(1) as i32);
            *pages.borrow_mut() = built;
        });
    }
    {
        let pages = pages.clone();
        let title = title.clone();
        let options = options.clone();
        op.connect_draw_page(move |_, context, page_nr| {
            draw_page(context, &pages.borrow(), page_nr as usize, &title, options.borrow().font_pt);
        });
    }
    {
        let state = state.clone();
        let options = options.clone();
        op.connect_done(move |_, result| {
            if result != gtk4::PrintOperationResult::Apply {
                return;
            }
            let opts = options.borrow();
            let mut st = state.borrow_mut();
            st.config.print_include_notes = opts.include_notes;
            st.config.print_include_tags = opts.include_tags;
            st.config.print_font_pt = opts.font_pt;
            let _ = st.config.save();
        });
    }

    if let Err(e) = op.run(gtk4::PrintOperationAction::PrintDialog, Some(parent)) {
        tracing::warn!("print operation failed: {e}");
    }
}

fn paginate(sermon: &Sermon, context: &gtk4::PrintContext, options: &PrintOptions) -> Vec<Vec<Measured>> {
    let available_width = context.width() - 2.0 * PAGE_MARGIN_PT;
    let available_height = context.height() - 2.0 * PAGE_MARGIN_PT - HEADER_HEIGHT_PT - FOOTER_HEIGHT_PT;

    let mut blocks = vec![Block::Title(sermon.display_title().to_string())];
    if let Some(date) = sermon.planned_date {
        blocks.push(Block::Date(date.format("%B %-d, %Y").to_string()));
    }
    for movement in &sermon.movements {
        blocks.push(Block::Movement(movement.name.clone()));
        for idea in &movement.ideas {
            if idea.text.is_empty() && idea.notes.is_empty() {
                continue;
            }
            if options.include_tags && (!idea.idea_tag.is_empty() || !idea.part_tag.is_empty()) {
                let tag_line = [&idea.idea_tag, &idea.part_tag]
                    .into_iter()
                    .filter(|t| !t.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" · ");
                blocks.push(Block::IdeaTags(tag_line));
            }
            if !idea.text.is_empty() {
                blocks.push(Block::IdeaText(crate::bible::render_display(&idea.text)));
            }
            if options.include_notes && !idea.notes.is_empty() {
                blocks.push(Block::IdeaNotes(idea.notes.clone()));
            }
        }
    }

    if !sermon.s_tags.is_empty() {
        blocks.push(Block::Bibliography);
        let mut tags = sermon.s_tags.clone();
        tags.sort_by_key(|t| crate::bible::sort_key(t));
        for tag in tags {
            blocks.push(Block::BibliographyEntry(tag));
        }
    }

    let measured: Vec<Measured> = blocks
        .into_iter()
        .map(|block| measure(context, &block, available_width, options.font_pt))
        .collect();

    let mut pages = Vec::new();
    let mut current: Vec<Measured> = Vec::new();
    let mut y = 0.0;
    for m in measured {
        let needed = m.top_margin + m.height;
        if y + needed > available_height && !current.is_empty() {
            pages.push(std::mem::take(&mut current));
            y = 0.0;
        }
        if matches!(m.block, Block::Movement(_) | Block::Bibliography)
            && !current.is_empty()
            && available_height - (y + needed) < MIN_ORPHAN_SPACE_PT
        {
            pages.push(std::mem::take(&mut current));
            y = 0.0;
        }
        y += needed;
        current.push(m);
    }
    if !current.is_empty() {
        pages.push(current);
    }
    if pages.is_empty() {
        pages.push(Vec::new());
    }
    pages
}

/// Text, font size/weight/style, and top margin for a block — the single
/// source both `measure` (during `begin_print`, for pagination) and
/// `draw_page` read from. They must never compute sizes independently: an
/// earlier draft had `draw_page` using its own hardcoded sizes while
/// `measure` scaled off `font_pt`, so the page-break math and the actual
/// drawn text silently disagreed (and the font-size option did nothing for
/// anything but idea text).
fn block_style(block: &Block, font_pt: f64) -> (&str, f64, bool, bool, f64) {
    match block {
        Block::Title(t) => (t.as_str(), font_pt * 2.0, true, false, 0.0),
        Block::Date(t) => (t.as_str(), font_pt * 1.1, false, false, 4.0),
        Block::Movement(t) => (t.as_str(), font_pt * 1.3, true, false, 26.0),
        Block::IdeaTags(t) => (t.as_str(), font_pt * 0.75, false, true, 12.0),
        Block::IdeaText(t) => (t.as_str(), font_pt, false, false, 10.0),
        Block::IdeaNotes(t) => (t.as_str(), font_pt * 0.85, false, true, 3.0),
        Block::Bibliography => ("Bibliography", font_pt * 1.3, true, false, 26.0),
        Block::BibliographyEntry(t) => (t.as_str(), font_pt * 0.95, false, false, 6.0),
    }
}

fn layout_for(context: &gtk4::PrintContext, text: &str, size_pt: f64, bold: bool, italic: bool, width: f64) -> pango::Layout {
    let layout = context.create_pango_layout();
    let mut desc = pango::FontDescription::new();
    desc.set_family("Serif");
    desc.set_size((size_pt * pango::SCALE as f64).round() as i32);
    if bold {
        desc.set_weight(pango::Weight::Bold);
    }
    if italic {
        desc.set_style(pango::Style::Italic);
    }
    layout.set_font_description(Some(&desc));
    layout.set_width((width * pango::SCALE as f64).round() as i32);
    layout.set_wrap(pango::WrapMode::WordChar);
    layout.set_text(text);
    layout
}

fn measure(context: &gtk4::PrintContext, block: &Block, width: f64, font_pt: f64) -> Measured {
    let (text, size_pt, bold, italic, top_margin) = block_style(block, font_pt);
    let layout = layout_for(context, text, size_pt, bold, italic, width);
    Measured {
        block: block.clone(),
        top_margin,
        height: layout.pixel_size().1 as f64,
    }
}

fn draw_page(context: &gtk4::PrintContext, pages: &[Vec<Measured>], page_nr: usize, title: &str, font_pt: f64) {
    let cr = context.cairo_context();
    let width = context.width();
    let height = context.height();
    cr.set_source_rgb(0.0, 0.0, 0.0);

    // Running header: sermon title, left-aligned, small caps-ish caption size.
    let header_layout = context.create_pango_layout();
    let mut header_desc = pango::FontDescription::new();
    header_desc.set_family("Sans");
    header_desc.set_size((9.0 * pango::SCALE as f64) as i32);
    header_layout.set_font_description(Some(&header_desc));
    header_layout.set_text(title);
    cr.move_to(PAGE_MARGIN_PT, PAGE_MARGIN_PT * 0.5);
    pangocairo::show_layout(&cr, &header_layout);

    // Footer: page number, centered.
    let footer_text = format!("Page {} of {}", page_nr + 1, pages.len());
    let footer_layout = context.create_pango_layout();
    let mut footer_desc = pango::FontDescription::new();
    footer_desc.set_family("Sans");
    footer_desc.set_size((9.0 * pango::SCALE as f64) as i32);
    footer_layout.set_font_description(Some(&footer_desc));
    footer_layout.set_text(&footer_text);
    let footer_width = footer_layout.pixel_size().0 as f64;
    cr.move_to((width - footer_width) / 2.0, height - PAGE_MARGIN_PT * 0.5 - 10.0);
    pangocairo::show_layout(&cr, &footer_layout);

    let Some(blocks) = pages.get(page_nr) else {
        return;
    };
    let available_width = width - 2.0 * PAGE_MARGIN_PT;
    let mut y = PAGE_MARGIN_PT + HEADER_HEIGHT_PT;
    for m in blocks {
        y += m.top_margin;
        let (text, size_pt, bold, italic, _) = block_style(&m.block, font_pt);
        let layout = layout_for(context, text, size_pt, bold, italic, available_width);
        cr.move_to(PAGE_MARGIN_PT, y);
        pangocairo::show_layout(&cr, &layout);
        y += m.height;
    }
}
