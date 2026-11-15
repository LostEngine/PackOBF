use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use packobf_gui::app::{App, LogLevel, LogMessage, log_line};

#[derive(Clone)]
pub(super) struct LogView {
    view: gtk::TextView,
    viewport: gtk::ScrolledWindow,
    tags: [gtk::TextTag; 3],
    visible_levels: Rc<Cell<[bool; 3]>>,
}

impl LogView {
    pub(super) fn new(container: &gtk::Box, state: &Rc<RefCell<App>>) -> Self {
        let panel = gtk::Box::new(gtk::Orientation::Vertical, 6);
        panel.set_vexpand(true);
        panel.set_visible(false);

        let toggle = gtk::ToggleButton::with_label("Show logs");
        toggle.set_halign(gtk::Align::Start);
        toggle.connect_toggled({
            let panel = panel.clone();
            move |toggle| {
                panel.set_visible(toggle.is_active());
                toggle.set_label(if toggle.is_active() {
                    "Hide logs"
                } else {
                    "Show logs"
                });
            }
        });
        container.append(&toggle);
        container.append(&panel);

        let filters = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        panel.append(&filters);

        let view = gtk::TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(gtk::WrapMode::WordChar)
            .top_margin(8)
            .bottom_margin(8)
            .left_margin(8)
            .right_margin(8)
            .build();
        let buffer = view.buffer();
        let tags = ["#3584e4", "#ac6900", "#e01b24"].map(|color| {
            let tag = gtk::TextTag::builder()
                .foreground(color)
                .weight(700)
                .build();
            buffer.tag_table().add(&tag);
            tag
        });

        let viewport = gtk::ScrolledWindow::builder()
            .has_frame(true)
            .min_content_height(120)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&view)
            .vexpand(true)
            .build();
        panel.append(&viewport);

        let logs = Self {
            view,
            viewport,
            tags,
            visible_levels: Rc::new(Cell::new([true; 3])),
        };
        for (index, label) in ["Info", "Warning", "Error"].into_iter().enumerate() {
            let filter = gtk::CheckButton::builder()
                .label(label)
                .active(true)
                .build();
            filter.connect_toggled({
                let logs = logs.clone();
                let state = state.clone();
                move |filter| {
                    let mut visible_levels = logs.visible_levels.get();
                    visible_levels[index] = filter.is_active();
                    logs.visible_levels.set(visible_levels);

                    logs.view.buffer().set_text("");
                    logs.insert(state.borrow().logs());
                    logs.scroll_to_end();
                }
            });
            filters.append(&filter);
        }

        let copy = gtk::Button::with_label("Copy");
        copy.set_tooltip_text(Some("Copy all log messages"));
        copy.connect_clicked({
            let state = state.clone();
            move |button| {
                let text = state.borrow().copyable_logs();
                button.clipboard().set_text(&text.replace('\0', "␀"));
            }
        });
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        filters.append(&spacer);
        filters.append(&copy);

        toggle.connect_toggled({
            let logs = logs.clone();
            move |toggle| {
                if toggle.is_active() && logs.at_end() {
                    logs.scroll_to_end();
                }
            }
        });
        logs
    }

    pub(super) fn clear(&self) {
        self.view.buffer().set_text("");
        self.scroll_to_end();
    }

    pub(super) fn append(&self, batch: &[LogMessage]) {
        if batch.is_empty() {
            return;
        }

        // The adjustment's upper bound will grow when GTK lays out the new text.
        let follow = self.at_end();
        self.insert(batch);
        if follow {
            self.scroll_to_end();
        }
    }

    fn insert(&self, batch: &[LogMessage]) {
        let buffer = self.view.buffer();
        let mut end = buffer.end_iter();
        let visible_levels = self.visible_levels.get();
        for log in batch {
            let index = match log.level {
                LogLevel::Info => 0,
                LogLevel::Warning => 1,
                LogLevel::Error => 2,
            };
            if visible_levels[index] {
                let line = log_line(log).replace('\0', "␀");
                buffer.insert_with_tags(&mut end, &format!("{line}\n"), &[&self.tags[index]]);
            }
        }
    }

    fn at_end(&self) -> bool {
        let adjustment = self.viewport.vadjustment();
        adjustment.value() + adjustment.page_size() + 2.0 >= adjustment.upper()
    }

    fn scroll_to_end(&self) {
        let adjustment = self.viewport.vadjustment();
        // Wait for GTK to update the range for all the newly inserted text.
        gtk::glib::idle_add_local_once(move || {
            adjustment.set_value(adjustment.upper() - adjustment.page_size());
        });
    }
}
