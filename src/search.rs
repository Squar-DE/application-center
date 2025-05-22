use adw::prelude::*;
use gtk::{Box as GtkBox, Orientation, Button, glib::clone, ProgressBar};
use tokio::process::Command;
use pango;
use crate::progress_bar::{install_with_progress, uninstall_with_progress, is_package_installed};

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
                let app_name_owned = app_name_owned.clone();
                let version_owned = version_owned.clone();
                let description_owned = description_owned.clone();
                let display_name = display_name.clone();
                
                glib::spawn_future_local(clone!(@weak info_revealer, @weak info_center, @weak separator_revealer => async move {
                    // Clear previous content
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

                    // Create a progress bar for operations
                    let progress_bar = ProgressBar::new();
                    progress_bar.set_show_text(true);
                    progress_bar.set_margin_bottom(10);
                    progress_bar.set_visible(false); // Initially hidden

                    // Check if package is already installed
                    let is_installed = match is_package_installed(&app_name_owned).await {
                        Ok(installed) => installed,
                        Err(e) => {
                            eprintln!("Error checking package status: {}", e);
                            false // Default to not installed if we can't check
                        }
                    };

                    // Create status label
                    let status_label = gtk::Label::new(None);
                    if is_installed {
                        status_label.set_markup("<span color=\"#2ecc71\" weight=\"bold\">✓ Installed</span>");
                    } else {
                        status_label.set_markup("<span color=\"#95a5a6\">Not installed</span>");
                    }
                    status_label.set_halign(gtk::Align::Start);
                    status_label.set_margin_bottom(10);

                    // Create button box for multiple actions
                    let button_box = GtkBox::new(Orientation::Horizontal, 8);
                    button_box.set_halign(gtk::Align::Start);

                    if is_installed {
                        // Package is installed - show reinstall and uninstall options
                        let reinstall_button = Button::with_label("Reinstall");
                        reinstall_button.add_css_class("suggested-action");
                        reinstall_button.add_css_class("pill");
                        reinstall_button.connect_clicked(clone!(@strong app_name_owned, @weak progress_bar, @weak status_label => move |_| {
                            let package_name = app_name_owned.clone();
                            glib::spawn_future_local(clone!(@weak progress_bar, @weak status_label => async move {
                                progress_bar.set_visible(true);
                                progress_bar.set_fraction(0.0);

                                match install_with_progress(&package_name, progress_bar.clone()).await {
                                    Ok(_) => {
                                        status_label.set_markup("<span color=\"#2ecc71\" weight=\"bold\">✓ Installed</span>");
                                        let notification = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Info)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text(&format!("Successfully reinstalled {}", package_name))
                                            .build();
                                        notification.connect_response(|dialog, _| dialog.close());
                                        notification.show();
                                    },
                                    Err(e) => {
                                        let notification = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Error)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text(&format!("Failed to reinstall {}: {}", package_name, e))
                                            .build();
                                        notification.connect_response(|dialog, _| dialog.close());
                                        notification.show();
                                    }
                                }
                                progress_bar.set_visible(false);
                            }));
                        }));

                        let uninstall_button = Button::with_label("Uninstall");
                        uninstall_button.add_css_class("destructive-action");
                        uninstall_button.add_css_class("pill");
                        uninstall_button.connect_clicked(clone!(@strong app_name_owned, @weak progress_bar, @weak status_label => move |_| {
                            let package_name = app_name_owned.clone();
                            glib::spawn_future_local(clone!(@weak progress_bar, @weak status_label => async move {
                                progress_bar.set_visible(true);
                                progress_bar.set_fraction(0.0);

                                match uninstall_with_progress(&package_name, progress_bar.clone()).await {
                                    Ok(_) => {
                                        status_label.set_markup("<span color=\"#95a5a6\">Not installed</span>");
                                        let notification = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Info)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text(&format!("Successfully uninstalled {}", package_name))
                                            .build();
                                        notification.connect_response(|dialog, _| dialog.close());
                                        notification.show();
                                    },
                                    Err(e) => {
                                        let notification = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Error)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text(&format!("Failed to uninstall {}: {}", package_name, e))
                                            .build();
                                        notification.connect_response(|dialog, _| dialog.close());
                                        notification.show();
                                    }
                                }
                                progress_bar.set_visible(false);
                            }));
                        }));

                        button_box.append(&reinstall_button);
                        button_box.append(&uninstall_button);
                    } else {
                        // Package is not installed - show install option
                        let install_button = Button::with_label("Install");
                        install_button.add_css_class("suggested-action");
                        install_button.add_css_class("pill");
                        install_button.connect_clicked(clone!(@strong app_name_owned, @weak progress_bar, @weak status_label => move |_| {
                            let package_name = app_name_owned.clone();
                            glib::spawn_future_local(clone!(@weak progress_bar, @weak status_label => async move {
                                progress_bar.set_visible(true);
                                progress_bar.set_fraction(0.0);

                                match install_with_progress(&package_name, progress_bar.clone()).await {
                                    Ok(_) => {
                                        status_label.set_markup("<span color=\"#2ecc71\" weight=\"bold\">✓ Installed</span>");
                                        let notification = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Info)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text(&format!("Successfully installed {}", package_name))
                                            .build();
                                        notification.connect_response(|dialog, _| dialog.close());
                                        notification.show();
                                    },
                                    Err(e) => {
                                        let notification = gtk::MessageDialog::builder()
                                            .message_type(gtk::MessageType::Error)
                                            .buttons(gtk::ButtonsType::Ok)
                                            .text(&format!("Failed to install {}: {}", package_name, e))
                                            .build();
                                        notification.connect_response(|dialog, _| dialog.close());
                                        notification.show();
                                    }
                                }
                                progress_bar.set_visible(false);
                            }));
                        }));

                        button_box.append(&install_button);
                    }

                    info_center.append(&name_label);
                    info_center.append(&version_label);
                    info_center.append(&desc_label);
                    info_center.append(&status_label);
                    info_center.append(&progress_bar);
                    info_center.append(&button_box);

                    separator_revealer.set_reveal_child(true);
                    info_revealer.set_reveal_child(true);
                }));
            }));
        }
    });
}
