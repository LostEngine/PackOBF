mod files;
mod logs;
mod options;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::{gdk, glib, pango};
use packobf_gui::app::{App, Statistics, Status};

use self::files::{PathEntry, Picker, add_path_button, connect_save};
use self::logs::LogView;
use self::options::Controls;

pub fn run() -> glib::ExitCode {
    let application = adw::Application::builder()
        .application_id("dev.lost.packobf")
        .build();
    application.connect_startup(|_| {
        if let Some(display) = gdk::Display::default() {
            let styles = gtk::CssProvider::new();
            // EntryRow has no property for hiding its decorative edit icon.
            styles.load_from_data(
                "row.pack-path image.edit-icon {
                    opacity: 0;
                    -gtk-icon-size: 1px;
                    min-width: 0;
                    min-height: 0;
                    margin: 0;
                    padding: 0;
                }",
            );
            gtk::style_context_add_provider_for_display(
                &display,
                &styles,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });
    application.connect_activate(|application| {
        if let Some(window) = application.active_window() {
            window.present();
        } else {
            build_window(application).present();
        }
    });
    application.run()
}

fn build_window(application: &adw::Application) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("PackOBF")
        .default_width(720)
        .default_height(800)
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    window.set_content(Some(&toolbar));

    let state = Rc::new(RefCell::new(App::default()));
    let page = adw::PreferencesPage::new();
    toolbar.set_content(Some(&page));

    let file_group = adw::PreferencesGroup::builder()
        .title("Resource pack")
        .description("Select a Minecraft: Java Edition resource pack ZIP.")
        .build();
    let input = PathEntry::new("Resource pack file");
    let cache = PathEntry::new("Cache file (recommended)");
    add_path_button(
        "Select resource pack",
        "document-open-symbolic",
        Picker::Input,
        &window,
        &input,
    );
    add_path_button(
        "Import cache file",
        "document-open-symbolic",
        Picker::ImportCache,
        &window,
        &cache,
    );
    add_path_button(
        "Create cache file",
        "document-new-symbolic",
        Picker::CreateCache,
        &window,
        &cache,
    );
    file_group.add(&input.widget);
    file_group.add(&cache.widget);
    page.add(&file_group);

    let options = Controls::new(&page);
    let status = gtk::Label::builder()
        .label("Idle")
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(pango::EllipsizeMode::End)
        .build();
    let activity = adw::PreferencesGroup::new();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    activity.add(&content);
    page.add(&activity);
    let logs = LogView::new(&content, &state);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_margin_top(8);
    actions.set_margin_bottom(8);
    actions.set_margin_start(16);
    actions.set_margin_end(16);
    let optimize = gtk::Button::builder()
        .label("Optimize")
        .sensitive(false)
        .css_classes(["suggested-action"])
        .build();
    let save = gtk::Button::builder()
        .label("Save file")
        .visible(false)
        .build();
    actions.append(&status);
    actions.append(&optimize);
    actions.append(&save);
    toolbar.set_bottom_bar_style(adw::ToolbarStyle::Raised);
    toolbar.add_bottom_bar(&actions);
    connect_save(&save, &window, &state);

    let invalidate_result = {
        let state = state.clone();
        let status = status.clone();
        let optimize = optimize.downgrade();
        let save = save.clone();
        move || {
            let mut state = state.borrow_mut();
            state.invalidate_result();
            update_status(&status, state.status());
            save.set_visible(false);
            if let Some(optimize) = optimize.upgrade() {
                optimize.add_css_class("suggested-action");
            }
        }
    };
    options.connect_changed(invalidate_result.clone());
    cache.widget.connect_changed({
        let invalidate_result = invalidate_result.clone();
        move |_| invalidate_result()
    });
    input.widget.connect_changed({
        let optimize = optimize.downgrade();
        let state = state.clone();
        move |entry| {
            invalidate_result();
            if let Some(optimize) = optimize.upgrade() {
                optimize.set_sensitive(!state.borrow().is_running() && !entry.text().is_empty());
            }
        }
    });

    optimize.connect_clicked({
        let window = window.downgrade();
        let actions = actions.downgrade();
        move |optimize| {
            let Some(input_path) = input.path() else {
                return;
            };
            let (Some(parent), Some(actions)) = (window.upgrade(), actions.upgrade()) else {
                return;
            };
            let options = options.options();
            if let Err(error) = state.borrow_mut().start(input_path, cache.path(), options) {
                show_message(&parent, "Couldn’t start optimization", &error.to_string());
                return;
            }

            logs.clear();
            update_status(&status, state.borrow().status());
            optimize.set_visible(false);
            optimize.add_css_class("suggested-action");
            save.set_visible(false);
            let spinner = gtk::Spinner::builder()
                .spinning(true)
                .tooltip_text("Processing…")
                .build();
            actions.append(&spinner);

            let window = window.clone();
            let state = state.clone();
            let status = status.clone();
            let logs = logs.clone();
            let optimize = optimize.clone();
            let save = save.clone();
            let entry = input.widget.clone();
            glib::timeout_add_local(Duration::from_millis(50), move || {
                let Some(window) = window.upgrade() else {
                    return glib::ControlFlow::Break;
                };
                let mut state = state.borrow_mut();
                let last_log = state.logs().len();
                let stats = state.poll();
                logs.append(&state.logs()[last_log..]);
                update_status(&status, state.status());

                if state.is_running() {
                    return glib::ControlFlow::Continue;
                }

                actions.remove(&spinner);
                optimize.set_sensitive(!entry.text().is_empty());
                optimize.set_visible(true);
                if state.can_save() {
                    optimize.remove_css_class("suggested-action");
                    save.add_css_class("suggested-action");
                    save.set_visible(true);
                }
                if let Some(stats) = stats {
                    show_statistics(&window, &stats);
                }
                glib::ControlFlow::Break
            });
        }
    });

    window
}

fn update_status(label: &gtk::Label, status: &Status) {
    let text = status.to_string().replace('\0', "␀");
    if label.text() != text {
        label.set_text(&text);
        label.set_tooltip_text(Some(&text));
    }
}

fn show_statistics(window: &adw::ApplicationWindow, stats: &Statistics) {
    let body = format!(
        "Time taken: {:.3}s\nInput size: {}\nOutput size: {}\nSaved: {}",
        stats.elapsed.as_secs_f64(),
        stats.input_size(),
        stats.output_size(),
        stats.saved(),
    );
    show_message(window, "Optimization Complete", &body);
}

fn show_message(window: &adw::ApplicationWindow, heading: &str, body: &str) {
    let dialog = adw::AlertDialog::new(Some(heading), Some(body));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.set_close_response("ok");
    dialog.present(Some(window));
}
