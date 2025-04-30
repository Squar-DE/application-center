use adw::prelude::*;
use adw::Application;
use gtk::{Box as GtkBox, Orientation, Button, Entry, glib::clone};
use std::process::Command;
use pango;
fn main() {
    let app = Application::builder()
        .application_id("com.SquarDE.ApplicationManager")
        .build();

    app.connect_activate(build_ui);
    app.run(); 
}

fn build_ui(app: &Application) {
    let win = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .build();

    let header = adw::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("Application Center")))
        .show_end_title_buttons(true)
        .build();
    let vbox = GtkBox::new(Orientation::Vertical, 0);
    vbox.append(&header);
    let search_box = GtkBox::new(Orientation::Horizontal, 6);
    search_box.set_halign(gtk::Align::Center);
    let search_entry = Entry::new();
    search_box.append(&search_entry);
    search_box.set_margin_top(10);
    vbox.append(&search_box);
    // After creating the separator
    let seperator = gtk::Separator::new(Orientation::Horizontal);
    seperator.set_margin_top(10);
    vbox.append(&seperator);
    
    // Create a horizontal box to hold both app list and info center
    let content_box = GtkBox::new(Orientation::Horizontal, 12);
    vbox.append(&content_box);
    
    // Set up app list as before
    let app_list = gtk::FlowBox::new();
    app_list.set_valign(gtk::Align::Start);
    app_list.set_max_children_per_line(3); // 3 buttons per row
    app_list.set_selection_mode(gtk::SelectionMode::None);
    
    let app_list_scroll = gtk::ScrolledWindow::builder()
        .child(&app_list)
        .vexpand(true)
        .hexpand(true) // Let app list take available space
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .build();
    
    // Add the app list to the horizontal content box (left side)
    content_box.append(&app_list_scroll);
    // Create a revealer for the separator
    let separator_revealer = gtk::Revealer::new();
    separator_revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
    separator_revealer.set_transition_duration(300);

    // Add a vertical separator
    let vertical_separator = gtk::Separator::new(Orientation::Vertical);
    vertical_separator.set_margin_top(10);
    vertical_separator.set_margin_bottom(10);
    separator_revealer.set_child(Some(&vertical_separator)); 
    content_box.append(&separator_revealer);
    content_box.append(&vertical_separator);
    let info_revealer = gtk::Revealer::new();
    info_revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft); // Slide from right
    info_revealer.set_transition_duration(300); // 300ms animation
    
    // Set up info center as before
    let info_center = GtkBox::new(Orientation::Vertical, 12);
    info_center.set_margin_top(12);
    info_center.set_margin_start(12);
    info_center.set_margin_end(12);
    info_center.set_width_request(200); // Give it a minimum width
    info_center.set_vexpand(true);
    
    // Add info_center to the revealer
    info_revealer.set_child(Some(&info_center));
    
    // Add the revealer to the horizontal content box (right side)
    content_box.append(&info_revealer);
    search_entry.connect_activate(clone!(@weak search_entry => move |_| {
        let query = search_entry.text();
        if !query.is_empty() {
            search_pacman(&query, &app_list, &info_revealer, &info_center, &separator_revealer);
        }
    }));
    win.set_content(Some(&vbox));
    win.show();
}



fn search_pacman(query: &str, app_list: &gtk::FlowBox, info_revealer: &gtk::Revealer, info_center: &gtk::Box, separator_revealer: &gtk::Revealer) {
    let output = Command::new("pacman")
        .args(["-Ss", query])
        .output()
        .expect("Failed to execute pacman");

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.lines().collect();

    // clear previous search results
    while let Some(child) = app_list.last_child() {
        app_list.remove(&child);
    }

    let mut results = vec![];

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        if line.trim().is_empty() || !line.contains('/') {
            i += 1;
            continue;
        }

        if let Some(name_version) = line.split_whitespace().next() {
            let mut parts = name_version.split('/');
            let _repo = parts.next();
            let app_name = parts.next().unwrap_or(name_version);

            let version = line.split_whitespace().nth(1).unwrap_or("");

            let description = if i + 1 < lines.len() {
                lines[i + 1].trim()
            } else {
                ""
            };

            // calculate score
            let query_lower = query.to_lowercase();
            let app_lower = app_name.to_lowercase();
            let desc_lower = description.to_lowercase();

            let score = if app_lower == query_lower {
                100
            } else if app_lower.starts_with(&query_lower) {
                80
            } else if app_lower.contains(&query_lower) {
                50
            } else if desc_lower.contains(&query_lower) {
                10
            } else {
                0
            };

            results.push((score, app_name.to_string(), version.to_string(), description.to_string()));
            i += 2;
        } else {
            i += 1;
        }
    }

    // sort by score descending
    results.sort_by(|a, b| b.0.cmp(&a.0));

    for (_score, app_name_owned, version_owned, description_owned) in results {
        // same button making code as before
        let button_wrapper = GtkBox::new(Orientation::Vertical, 0);
        button_wrapper.set_hexpand(true);
        button_wrapper.set_vexpand(false);

        let button = Button::builder()
            .halign(gtk::Align::Fill)
            .hexpand(true)
            .height_request(100)
            .build();

        button.set_margin_top(8);
        button.set_margin_bottom(4);

        let name_label = gtk::Label::new(None);
        name_label.set_markup(&format!("<span size=\"x-large\">{}</span>", app_name_owned));
        name_label.set_halign(gtk::Align::Start);

        let version_label = gtk::Label::new(None);
        version_label.set_markup(&format!("<span size=\"small\" color=\"#888888\">{}</span>", version_owned));
        version_label.set_halign(gtk::Align::Start);

        let label_box = GtkBox::new(Orientation::Vertical, 0);
        label_box.set_hexpand(true);
        label_box.append(&name_label);
        label_box.append(&version_label);

        button.set_child(Some(&label_box));
        button_wrapper.append(&button);

        app_list.insert(&button_wrapper, -1);

        button.connect_clicked(clone!(@weak info_revealer, @weak info_center, @weak separator_revealer => move |_| {
            while let Some(child) = info_center.last_child() {
                info_center.remove(&child);
            }

            let name_label = gtk::Label::new(Some(&app_name_owned));
            name_label.set_markup(&format!("<span size=\"xx-large\" weight=\"bold\">{}</span>", &app_name_owned));
            name_label.set_margin_bottom(4);
            name_label.set_halign(gtk::Align::Start);

            let version_label = gtk::Label::new(Some(&version_owned));
            version_label.set_markup(&format!("<span size=\"small\" color=\"#888888\">Version {}</span>", &version_owned));
            version_label.set_margin_bottom(8);
            version_label.set_halign(gtk::Align::Start);

            let desc_label = gtk::Label::new(Some(&description_owned));
            desc_label.set_wrap(true);
            desc_label.set_wrap_mode(pango::WrapMode::WordChar);
            desc_label.set_max_width_chars(50);
            desc_label.set_halign(gtk::Align::Start);
            desc_label.set_margin_bottom(10);

            let install_name = app_name_owned.clone();
            let install_button = Button::with_label("Install");
            install_button.connect_clicked(move |_| {
                let install_cmd = format!("pkexec pacman -S --noconfirm {}", install_name);
                let result = Command::new("sh")
                    .arg("-c")
                    .arg(&install_cmd)
                    .output();
            
                match result {
                    Ok(output) => {
                        if output.status.success() {
                            // Show notification if installed
                            let notification = gtk::MessageDialog::builder()
                                .message_type(gtk::MessageType::Info)
                                .buttons(gtk::ButtonsType::Ok)
                                .text(&format!("Successfully installed {}", install_name))
                                .build();
                            notification.connect_response(|dialog, _| {
                                dialog.close();
                            });
                            notification.show();
                        } else {
                            // Optional: Show error if install failed
                            let notification = gtk::MessageDialog::builder()
                                .message_type(gtk::MessageType::Error)
                                .buttons(gtk::ButtonsType::Ok)
                                .text(&format!("Failed to install {}", install_name))
                                .build();
                            notification.connect_response(|dialog, _| {
                                dialog.close();
                            });
                            notification.show();
                        }
                    }
                    Err(err) => {
                        println!("Error occurred: {}", err);
                    }
                }
            });

            info_center.append(&name_label);
            info_center.append(&version_label);
            info_center.append(&desc_label);
            info_center.append(&install_button);

            separator_revealer.set_reveal_child(true);
            info_revealer.set_reveal_child(true);
        }));
    }
}


