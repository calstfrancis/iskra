//! Sermon library: search + a clickable tag sidebar (grouped scripture/theme,
//! per Plans/iskra-kickoff-prompt.md §5) + a sermon list, modeled loosely on
//! Zerkalo's/Rubric's library windows but sized for Iskra's flat, single-kind
//! collection (no folders/projects). New/open/delete; the currently open
//! sermon can't be deleted from here.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Label, ListBox, ListBoxRow, MenuButton, Orientation, Popover,
    ScrolledWindow, SearchEntry, Separator, SelectionMode,
};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::commands::SermonTagKind;
use crate::library::{LibraryFilter, LibraryIndex};
use crate::model::Sermon;
use crate::sermon_templates::TEMPLATES;

pub struct LibraryWindow {
    window: adw::Window,
    search_entry: SearchEntry,
    tag_sidebar: GtkBox,
    sermon_list: ListBox,
    empty_page: adw::StatusPage,
    sermons_dir: PathBuf,
    query: RefCell<String>,
    tag_filter: RefCell<Option<LibraryFilter>>,
    current_open: RefCell<Option<PathBuf>>,
    on_open: RefCell<Option<Box<dyn Fn(PathBuf)>>>,
    on_new: RefCell<Option<Box<dyn Fn(Option<String>)>>>,
    on_delete: RefCell<Option<Box<dyn Fn(PathBuf)>>>,
}

impl LibraryWindow {
    pub fn new(parent: &impl IsA<gtk4::Window>, sermons_dir: PathBuf) -> Rc<Self> {
        let window = adw::Window::builder()
            .title("Sermon Library")
            .transient_for(parent)
            .default_width(760)
            .default_height(560)
            .build();

        let header = adw::HeaderBar::new();
        let new_btn = MenuButton::new();
        new_btn.set_icon_name("list-add-symbolic");
        new_btn.set_tooltip_text(Some("New sermon"));
        header.pack_start(&new_btn);

        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search title, notes, tags, date…"));
        header.set_title_widget(Some(&search_entry));

        let tag_sidebar = GtkBox::new(Orientation::Vertical, 2);
        tag_sidebar.set_margin_top(8);
        tag_sidebar.set_margin_bottom(8);
        tag_sidebar.set_margin_start(6);
        tag_sidebar.set_margin_end(6);
        let sidebar_scroll = ScrolledWindow::new();
        sidebar_scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
        sidebar_scroll.set_child(Some(&tag_sidebar));
        sidebar_scroll.set_width_request(200);

        let sermon_list = ListBox::new();
        sermon_list.set_selection_mode(SelectionMode::None);
        sermon_list.add_css_class("boxed-list");

        let empty_page = adw::StatusPage::new();
        empty_page.set_icon_name(Some("folder-symbolic"));
        empty_page.set_title("No sermons found");
        empty_page.set_description(Some("Try a different search, or start a new sermon."));
        empty_page.set_vexpand(true);
        empty_page.set_visible(false);

        let list_col = GtkBox::new(Orientation::Vertical, 0);
        let list_scroll = ScrolledWindow::new();
        list_scroll.set_vexpand(true);
        list_scroll.set_hscrollbar_policy(gtk4::PolicyType::Never);
        let list_margin = GtkBox::new(Orientation::Vertical, 0);
        list_margin.set_margin_top(8);
        list_margin.set_margin_bottom(8);
        list_margin.set_margin_start(8);
        list_margin.set_margin_end(8);
        list_margin.append(&sermon_list);
        list_scroll.set_child(Some(&list_margin));
        list_col.append(&list_scroll);
        list_col.append(&empty_page);

        let split = GtkBox::new(Orientation::Horizontal, 0);
        split.append(&sidebar_scroll);
        split.append(&Separator::new(Orientation::Vertical));
        split.append(&list_col);
        split.set_hexpand(true);
        split.set_vexpand(true);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&split));
        window.set_content(Some(&toolbar));

        let this = Rc::new(Self {
            window,
            search_entry: search_entry.clone(),
            tag_sidebar,
            sermon_list,
            empty_page,
            sermons_dir,
            query: RefCell::new(String::new()),
            tag_filter: RefCell::new(None),
            current_open: RefCell::new(None),
            on_open: RefCell::new(None),
            on_new: RefCell::new(None),
            on_delete: RefCell::new(None),
        });

        {
            let this = this.clone();
            search_entry.connect_search_changed(move |e| {
                *this.query.borrow_mut() = e.text().to_string();
                this.rebuild_sermon_list();
            });
        }
        {
            let popover_box = GtkBox::new(Orientation::Vertical, 0);
            popover_box.set_margin_top(4);
            popover_box.set_margin_bottom(4);
            popover_box.set_margin_start(4);
            popover_box.set_margin_end(4);
            let popover = Popover::new();
            popover.set_child(Some(&popover_box));

            let blank_btn = new_sermon_menu_row("Blank Sermon", None);
            popover_box.append(&blank_btn);
            popover_box.append(&Separator::new(Orientation::Horizontal));
            for template in TEMPLATES {
                let row = new_sermon_menu_row(template.name, Some(template.description));
                popover_box.append(&row);
                let this = this.clone();
                let popover = popover.clone();
                let template_id = template.id.to_string();
                row.connect_clicked(move |_| {
                    popover.popdown();
                    if let Some(f) = this.on_new.borrow().as_ref() {
                        f(Some(template_id.clone()));
                    }
                });
            }
            {
                let this = this.clone();
                let popover = popover.clone();
                blank_btn.connect_clicked(move |_| {
                    popover.popdown();
                    if let Some(f) = this.on_new.borrow().as_ref() {
                        f(None);
                    }
                });
            }

            new_btn.set_popover(Some(&popover));
        }

        this
    }

    pub fn set_on_open(&self, f: impl Fn(PathBuf) + 'static) {
        *self.on_open.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_on_new(&self, f: impl Fn(Option<String>) + 'static) {
        *self.on_new.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_on_delete(&self, f: impl Fn(PathBuf) + 'static) {
        *self.on_delete.borrow_mut() = Some(Box::new(f));
    }

    pub fn set_current_open(&self, path: Option<PathBuf>) {
        *self.current_open.borrow_mut() = path;
    }

    pub fn present(self: &Rc<Self>) {
        self.refresh();
        self.window.present();
        self.search_entry.grab_focus();
    }

    pub fn window(&self) -> &adw::Window {
        &self.window
    }

    /// Rescans the library from disk and rebuilds both the tag sidebar and
    /// the sermon list. Call whenever the window is (re)opened or a sermon
    /// is created/deleted elsewhere.
    pub fn refresh(self: &Rc<Self>) {
        self.rebuild_tag_sidebar();
        self.rebuild_sermon_list();
    }

    fn rebuild_tag_sidebar(self: &Rc<Self>) {
        while let Some(child) = self.tag_sidebar.first_child() {
            self.tag_sidebar.remove(&child);
        }

        let index = LibraryIndex::scan(&self.sermons_dir);

        let all_btn = tag_row("All sermons", None);
        {
            let this = self.clone();
            all_btn.connect_clicked(move |_| {
                *this.tag_filter.borrow_mut() = None;
                this.rebuild_sermon_list();
            });
        }
        self.tag_sidebar.append(&all_btn);
        self.tag_sidebar.append(&Separator::new(Orientation::Horizontal));

        self.tag_sidebar.append(&group_header("Scripture"));
        for (tag, count) in index.s_tag_census() {
            let btn = tag_row(&tag, Some(count));
            let this = self.clone();
            let tag_clone = tag.clone();
            btn.connect_clicked(move |_| {
                *this.tag_filter.borrow_mut() = Some(LibraryFilter::Tag(SermonTagKind::S, tag_clone.clone()));
                this.rebuild_sermon_list();
            });
            self.tag_sidebar.append(&btn);
        }

        self.tag_sidebar.append(&group_header("Theme"));
        for (tag, count) in index.t_tag_census() {
            let btn = tag_row(&tag, Some(count));
            let this = self.clone();
            let tag_clone = tag.clone();
            btn.connect_clicked(move |_| {
                *this.tag_filter.borrow_mut() = Some(LibraryFilter::Tag(SermonTagKind::T, tag_clone.clone()));
                this.rebuild_sermon_list();
            });
            self.tag_sidebar.append(&btn);
        }

        let series_census = index.series_census();
        if !series_census.is_empty() {
            self.tag_sidebar.append(&group_header("Series"));
            for (series, count) in series_census {
                let btn = tag_row(&series, Some(count));
                let this = self.clone();
                let series_clone = series.clone();
                btn.connect_clicked(move |_| {
                    *this.tag_filter.borrow_mut() = Some(LibraryFilter::Series(series_clone.clone()));
                    this.rebuild_sermon_list();
                });
                self.tag_sidebar.append(&btn);
            }
        }
    }

    fn rebuild_sermon_list(self: &Rc<Self>) {
        while let Some(child) = self.sermon_list.first_child() {
            self.sermon_list.remove(&child);
        }

        let index = LibraryIndex::scan(&self.sermons_dir);
        let query = self.query.borrow().clone();
        let tag_filter = self.tag_filter.borrow().clone();
        let results = index.filter(&query, tag_filter.as_ref());
        let current_open = self.current_open.borrow().clone();

        self.empty_page.set_visible(results.is_empty());
        self.sermon_list.set_visible(!results.is_empty());

        for (path, sermon) in results {
            let is_current = current_open.as_deref() == Some(path.as_path());
            let row = build_sermon_row(sermon, is_current, {
                let this = self.clone();
                let path = path.clone();
                let title = sermon.display_title().to_string();
                move || confirm_delete(&this, path.clone(), &title)
            });
            self.sermon_list.append(&row);
        }

        {
            let this = self.clone();
            self.sermon_list.connect_row_activated(move |list, row| {
                let index = row.index();
                let index_u = if index < 0 { return } else { index as usize };
                let entries = LibraryIndex::scan(&this.sermons_dir);
                let query = this.query.borrow().clone();
                let tag_filter = this.tag_filter.borrow().clone();
                let results = entries.filter(&query, tag_filter.as_ref());
                if let Some((path, _)) = results.get(index_u) {
                    if let Some(f) = this.on_open.borrow().as_ref() {
                        f(path.clone());
                    }
                }
                let _ = list;
            });
        }
    }
}

fn new_sermon_menu_row(name: &str, description: Option<&str>) -> Button {
    let col = GtkBox::new(Orientation::Vertical, 1);
    col.set_margin_top(4);
    col.set_margin_bottom(4);
    col.set_margin_start(8);
    col.set_margin_end(8);

    let name_lbl = Label::new(Some(name));
    name_lbl.set_xalign(0.0);
    col.append(&name_lbl);

    if let Some(desc) = description {
        let desc_lbl = Label::new(Some(desc));
        desc_lbl.add_css_class("dim-label");
        desc_lbl.add_css_class("caption");
        desc_lbl.set_xalign(0.0);
        desc_lbl.set_wrap(true);
        col.append(&desc_lbl);
    }

    let btn = Button::new();
    btn.set_child(Some(&col));
    btn.add_css_class("flat");
    btn.set_halign(Align::Fill);
    btn
}

fn group_header(text: &str) -> Label {
    let lbl = Label::new(Some(text));
    lbl.add_css_class("sidebar-header");
    lbl.set_xalign(0.0);
    lbl
}

fn tag_row(label: &str, count: Option<usize>) -> Button {
    let text = match count {
        Some(n) => format!("{label}  ·  {n}"),
        None => label.to_string(),
    };
    let btn = Button::with_label(&text);
    btn.add_css_class("flat");
    btn.set_halign(Align::Fill);
    if let Some(lbl) = btn.child().and_downcast_ref::<Label>() {
        lbl.set_xalign(0.0);
    }
    btn
}

fn confirm_delete(window: &Rc<LibraryWindow>, path: PathBuf, title: &str) {
    let alert = gtk4::AlertDialog::builder()
        .modal(true)
        .message("Delete this sermon?")
        .detail(&format!("'{title}' will be permanently deleted. This can't be undone."))
        .buttons(["Cancel", "Delete"])
        .cancel_button(0)
        .default_button(0)
        .build();
    let parent = window.window().clone();
    let window = window.clone();
    alert.choose(
        Some(&parent),
        None::<&gtk4::gio::Cancellable>,
        move |result| {
            if let Ok(1) = result {
                if let Some(f) = window.on_delete.borrow().as_ref() {
                    f(path.clone());
                }
                window.refresh();
            }
        },
    );
}

fn build_sermon_row(
    sermon: &Sermon,
    is_current: bool,
    on_delete: impl Fn() + 'static,
) -> ListBoxRow {
    let row = ListBoxRow::new();

    let hbox = GtkBox::new(Orientation::Horizontal, 8);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(8);
    hbox.set_margin_end(8);

    let text_col = GtkBox::new(Orientation::Vertical, 2);
    text_col.set_hexpand(true);

    let title = Label::new(Some(sermon.display_title()));
    title.set_xalign(0.0);
    title.add_css_class("heading");
    if is_current {
        title.set_markup(&format!("{} <span alpha='60%'>· open</span>", glib::markup_escape_text(sermon.display_title())));
    }
    text_col.append(&title);

    let mut subtitle_parts = Vec::new();
    if let Some(d) = sermon.planned_date {
        subtitle_parts.push(d.format("%B %-d, %Y").to_string());
    }
    if let Some(link) = &sermon.lectionary {
        subtitle_parts.push(link.week.clone());
    }
    let tag_text: Vec<String> = sermon
        .s_tags
        .iter()
        .chain(sermon.t_tags.iter())
        .cloned()
        .collect();
    if !tag_text.is_empty() {
        subtitle_parts.push(tag_text.join(", "));
    }
    let subtitle = Label::new(Some(&subtitle_parts.join("  ·  ")));
    subtitle.add_css_class("dim-label");
    subtitle.add_css_class("caption");
    subtitle.set_xalign(0.0);
    text_col.append(&subtitle);

    hbox.append(&text_col);

    if !is_current {
        let delete_btn = Button::from_icon_name("user-trash-symbolic");
        delete_btn.add_css_class("flat");
        delete_btn.set_valign(Align::Center);
        delete_btn.set_tooltip_text(Some("Delete sermon"));
        delete_btn.connect_clicked(move |_| on_delete());
        hbox.append(&delete_btn);
    }

    row.set_child(Some(&hbox));
    row
}
