use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use packobf_gui::app::App;

use super::show_message;

pub(super) struct PathEntry {
    pub(super) widget: adw::EntryRow,
    selected: Rc<RefCell<Option<PathBuf>>>,
}

impl PathEntry {
    pub(super) fn new(title: &str) -> Self {
        let widget = adw::EntryRow::builder()
            .title(title)
            .css_classes(["pack-path"])
            .build();
        let selected = Rc::new(RefCell::new(None));
        widget.connect_changed({
            let selected = selected.clone();
            move |_| *selected.borrow_mut() = None
        });
        Self { widget, selected }
    }

    pub(super) fn path(&self) -> Option<PathBuf> {
        let text = self.widget.text();
        if text.is_empty() {
            return None;
        }
        // The chooser can return paths that an editable GTK string cannot represent.
        self.selected
            .borrow()
            .clone()
            .or_else(|| Some(PathBuf::from(text.as_str())))
    }

    fn set_path(&self, path: PathBuf) {
        self.widget.set_text(&path.to_string_lossy());
        *self.selected.borrow_mut() = Some(path);
    }
}

#[derive(Clone, Copy)]
pub(super) enum Picker {
    Input,
    ImportCache,
    CreateCache,
    Output,
}

impl Picker {
    fn title(self) -> &'static str {
        match self {
            Self::Input => "Select Resource Pack",
            Self::ImportCache => "Import Cache File",
            Self::CreateCache => "Create Cache File",
            Self::Output => "Save Optimized Resource Pack",
        }
    }

    fn open(
        self,
        window: &adw::ApplicationWindow,
        on_chosen: impl FnOnce(&adw::ApplicationWindow, PathBuf) + 'static,
    ) {
        let title = self.title();
        let (filter_name, pattern, initial_name) = match self {
            Self::Input => ("ZIP files", "*.zip", None),
            Self::ImportCache => ("BIN files", "*.bin", None),
            Self::CreateCache => ("BIN files", "*.bin", Some("cache.bin")),
            Self::Output => ("ZIP files", "*.zip", Some("pack.zip")),
        };

        let filter = gtk::FileFilter::new();
        filter.set_name(Some(filter_name));
        filter.add_pattern(pattern);
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title(title)
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();

        let parent = window.downgrade();
        let on_result = move |result: Result<gio::File, glib::Error>| {
            let Some(window) = parent.upgrade() else {
                return;
            };
            match result {
                Ok(file) => match file.path() {
                    Some(path) => on_chosen(&window, path),
                    None => show_message(&window, title, "Choose a local file."),
                },
                Err(error)
                    if error.matches(gtk::DialogError::Dismissed)
                        || error.matches(gtk::DialogError::Cancelled) => {}
                Err(error) => show_message(&window, title, error.message()),
            }
        };

        if let Some(name) = initial_name {
            dialog.set_initial_name(Some(name));
            dialog.save(Some(window), gio::Cancellable::NONE, on_result);
        } else {
            dialog.open(Some(window), gio::Cancellable::NONE, on_result);
        }
    }
}

pub(super) fn add_path_button(
    label: &str,
    icon: &str,
    picker: Picker,
    window: &adw::ApplicationWindow,
    entry: &PathEntry,
) {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(label)
        .valign(gtk::Align::Center)
        .build();
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button.connect_clicked({
        let window = window.downgrade();
        let widget = entry.widget.downgrade();
        let selected = entry.selected.clone();
        move |_| {
            if let Some(window) = window.upgrade() {
                let widget = widget.clone();
                let selected = selected.clone();
                picker.open(&window, move |_, path| {
                    if let Some(widget) = widget.upgrade() {
                        PathEntry { widget, selected }.set_path(path);
                    }
                });
            }
        }
    });
    entry.widget.add_suffix(&button);
}

pub(super) fn connect_save(
    button: &gtk::Button,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<App>>,
) {
    let window = window.downgrade();
    let state = state.clone();
    button.connect_clicked(move |_| {
        if let Some(window) = window.upgrade() {
            let state = state.clone();
            Picker::Output.open(&window, move |window, chosen| {
                let path = if chosen.extension().is_none() {
                    chosen.with_extension("zip")
                } else {
                    chosen.clone()
                };

                // The chooser only confirmed overwriting the name it returned.
                if path != chosen && path.symlink_metadata().is_ok() {
                    let dialog = adw::AlertDialog::new(
                        Some("Replace existing file?"),
                        Some(&path.to_string_lossy()),
                    );
                    dialog.add_responses(&[("cancel", "Cancel"), ("replace", "Replace")]);
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");
                    dialog.set_response_appearance("replace", adw::ResponseAppearance::Destructive);
                    let parent = window.downgrade();
                    dialog.connect_response(Some("replace"), move |_, _| {
                        if let Some(window) = parent.upgrade() {
                            save(&window, &state.borrow(), &path);
                        }
                    });
                    dialog.present(Some(window));
                } else {
                    save(window, &state.borrow(), &path);
                }
            });
        }
    });
}

fn save(window: &adw::ApplicationWindow, state: &App, path: &Path) {
    if let Err(error) = state.save_output(path) {
        show_message(window, "Couldn’t save resource pack", &error.to_string());
    }
}
