use adw::prelude::*;
use gtk::{Box as GtkBox, Orientation, Button, glib::clone};
use tokio::process::Command;
use pango;

#[allow(deprecated)] // these warnings are way too annoying and clutter everything
pub fn search_pacman(query: &str, app_list: &gtk::FlowBox, info_revealer: &gtk::Revealer, info_center: &gtk::Box, separator_revealer: &gtk::Revealer) {
    // Clear previous search results
    while let Some(child) = app_list.last_child() {
        app_list.remove(&child);
    }

    let query = query.to_string();
    let app_list = app_list.clone();
    let info_revealer = info_revealer.clone();
    let info_center = info_center.clone();
    let separator_revealer = separator_revealer.clone();

    glib::spawn_future_local(async move {
        let output = match Command::new("pacman")
            .args(["-Ss", &query])
            .output()
            .await 
        {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute pacman: {}", e);
                return;
            }
        };

        let text = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = text.lines().collect();

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

        results.sort_by(|a, b| b.0.cmp(&a.0));

        for (_score, app_name_owned, version_owned, description_owned) in results {
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
            let display_name = app_name_owned.replace("-", " ");
            name_label.set_markup(&format!("<span size=\"x-large\">{}</span>", display_name));
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
                name_label.set_markup(&format!("<span size=\"xx-large\" weight=\"bold\">{}</span>", &display_name));
                name_label.set_margin_bottom(4);
                name_label.set_halign(gtk::Align::Start);

                let version_label = gtk::Label::new(Some(&version_owned));
                version_label.set_markup(&format!("<span size=\"small\" color=\"#888888\">Version {}</span>", &version_owned));
                version_label.set_margin_bottom(8);
                version_label.set_halign(gtk::Align::Start);

                let desc_label = gtk::Label::new(Some(&description_owned));
                desc_label.set_wrap(true);
                desc_label.set_wrap_mode(pango::WrapMode::WordChar);
                desc_label.set_max_width_chars(40);
                desc_label.set_halign(gtk::Align::Start);
                desc_label.set_margin_bottom(10);

                let install_name = app_name_owned.clone();
                let install_button = Button::with_label("Install");
                install_button.add_css_class("suggested-action");
                install_button.add_css_class("pill");
                install_button.connect_clicked(clone!(@strong install_name => move |_| {
                    let install_name = install_name.clone();
                    glib::spawn_future_local(async move {
                        let result = Command::new("pkexec")
                            .args(["pacman", "-S", "--noconfirm", &install_name])
                            .status()
                            .await;
                    
                        match result {
                            Ok(status) if status.success() => {
                                let notification = gtk::MessageDialog::builder()
                                    .message_type(gtk::MessageType::Info)
                                    .buttons(gtk::ButtonsType::Ok)
                                    .text(&format!("Successfully installed {}", install_name))
                                    .build();
                                notification.connect_response(|dialog, _| {
                                    dialog.close();
                                });
                                notification.show();
                            },
                            Ok(_) => {
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
                            Err(err) => {
                                eprintln!("Error occurred: {}", err);
                            }
                        }
                    });
                }));

                info_center.append(&name_label);
                info_center.append(&version_label);
                info_center.append(&desc_label);
                info_center.append(&install_button);

                separator_revealer.set_reveal_child(true);
                info_revealer.set_reveal_child(true);
            }));
        }
    });
}
