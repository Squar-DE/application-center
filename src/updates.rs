use gtk::{Box as GtkBox, Button, Orientation, Label, glib::clone, ScrolledWindow, PolicyType, ProgressBar};
use gtk::prelude::*;
use tokio::process::Command;
use adw;

use crate::home::home;
use crate::progress_bar::install_with_progress;

#[allow(deprecated)] // these warnings are way too annoying and clutter everything
pub fn update(vbox: &GtkBox) {
    // Clear all existing children
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }

    let header = adw::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("Updates -- Application Center")))
        .show_end_title_buttons(true)
        .build();

    vbox.append(&header);

    let update_button = Button::builder()
        .label("Check for Updates")
        .halign(gtk::Align::Center)
        .build();
    update_button.set_margin_top(10);
    update_button.add_css_class("pill");

    let seperator = gtk::Separator::new(Orientation::Horizontal);
    seperator.set_margin_top(10);
    
    let status_label = Label::new(None);
    status_label.set_halign(gtk::Align::Center);
    status_label.set_margin_top(20);
    status_label.set_margin_bottom(20);
    status_label.set_wrap(true);
    status_label.set_max_width_chars(50);

    // Add a main progress bar for "Update All" operation
    let main_progress_bar = ProgressBar::new();
    main_progress_bar.set_show_text(true);
    main_progress_bar.set_halign(gtk::Align::Fill);
    main_progress_bar.set_hexpand(true);
    main_progress_bar.set_margin_start(20);
    main_progress_bar.set_margin_end(20);
    main_progress_bar.set_visible(false); // Initially hidden

    let packages_box = GtkBox::new(Orientation::Vertical, 5);
    packages_box.set_margin_start(20);
    packages_box.set_margin_end(20);
    
    let scroll = ScrolledWindow::new();
    scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    scroll.set_child(Some(&packages_box));
    scroll.set_vexpand(true);
    scroll.set_hexpand(true);
    scroll.set_min_content_height(300);

    update_button.connect_clicked(clone!(@weak status_label, @weak packages_box, @weak main_progress_bar => move |_| {
    status_label.set_markup("<span size='large'>Checking for updates...</span>");
    
    while let Some(child) = packages_box.last_child() {
        packages_box.remove(&child);
    }

    main_progress_bar.set_visible(false);

    // Spawn async task
    glib::spawn_future_local(clone!(@weak status_label, @weak packages_box, @weak main_progress_bar => async move {
        // First, update the package database
        let sync_result = Command::new("pkexec")
            .args(["pacman", "-Sy"])
            .output()
            .await;
            
        if let Err(e) = sync_result {
            status_label.set_markup(&format!(
                "<span size='large' color='red'>Error updating package database:</span>\n<span size='small'>{}</span>",
                e
            ));
            return;
        }
        
        // Now check for updates
        match Command::new("pacman")
            .args(["-Qu"])
            .output()
            .await 
            {
                Ok(output) if !output.stdout.is_empty() => {
                    let updates = String::from_utf8_lossy(&output.stdout);
                    let update_count = updates.lines().count();
                
                    status_label.set_markup(&format!(
                        "<span size='large' weight='bold'>{}</span>\n<span size='small'>{}</span>",
                        "Updates Available!",
                        format!("{} packages can be updated", update_count)
                    ));

                    let update_all_button = Button::builder()
                        .label("Update All Packages")
                        .halign(gtk::Align::Center)
                        .margin_top(20)
                        .margin_bottom(20)
                        .build();
                    update_all_button.add_css_class("pill");
                    update_all_button.add_css_class("suggested-action");

                    let packages_box_clone = packages_box.clone();
                    update_all_button.connect_clicked(clone!(@weak status_label, @weak main_progress_bar => move |_| {
                        // Show and reset progress bar
                        main_progress_bar.set_visible(true);
                        main_progress_bar.set_fraction(0.0);
                        main_progress_bar.set_text(Some("Starting system update..."));
                        
                        glib::spawn_future_local(clone!(@weak status_label, @weak packages_box_clone, @weak main_progress_bar => async move {
                            // Use install_with_progress for the system update
                            match install_with_progress("", main_progress_bar.clone()).await {
                                Ok(_) => {
                                    status_label.set_markup("<span size='large' weight='bold'>System Updated Successfully!</span>");
                                    
                                    while let Some(child) = packages_box_clone.last_child() {
                                        packages_box_clone.remove(&child);
                                    }
                                },
                                Err(e) => {
                                    status_label.set_markup(&format!(
                                        "<span size='large' weight='bold' color='red'>Update Failed: {}</span>",
                                        e
                                    ));
                                }
                            }
                        }));
                    }));

                    packages_box.append(&update_all_button);

                    for line in updates.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            let package_name = parts[0];
                            let old_version = parts[1];
                            let new_version = parts[3].trim_end_matches('\n');

                            let package_box = GtkBox::new(Orientation::Vertical, 5);
                            package_box.set_halign(gtk::Align::Fill);
                            package_box.set_margin_top(10);
                            package_box.set_margin_bottom(10);
                            package_box.add_css_class("card");
                            package_box.set_margin_start(5);
                            package_box.set_margin_end(5);
                            
                            let top_box = GtkBox::new(Orientation::Horizontal, 10);
                            top_box.set_margin_start(10);
                            top_box.set_margin_end(10);
                            top_box.set_margin_top(10);
                            
                            let name_label = Label::new(Some(package_name));
                            name_label.set_halign(gtk::Align::Start);
                            name_label.set_hexpand(true);
                            name_label.set_markup(&format!("<b>{}</b>", package_name));

                            let version_box = GtkBox::new(Orientation::Horizontal, 5);
                            let old_label = Label::new(Some(old_version));
                            old_label.add_css_class("dim-label");
                            
                            let arrow_label = Label::new(Some("→"));
                            arrow_label.set_margin_start(5);
                            arrow_label.set_margin_end(5);
                            
                            let new_label = Label::new(Some(new_version));
                            new_label.add_css_class("accent");

                            version_box.append(&old_label);
                            version_box.append(&arrow_label);
                            version_box.append(&new_label);

                            let update_button = Button::with_label("Update");
                            update_button.add_css_class("suggested-action");
                            update_button.set_margin_start(10);
                            
                            top_box.append(&name_label);
                            top_box.append(&version_box);
                            top_box.append(&update_button);
                            
                            // Add progress bar for individual package updates
                            let package_progress = ProgressBar::new();
                            package_progress.set_show_text(true);
                            package_progress.set_halign(gtk::Align::Fill);
                            package_progress.set_hexpand(true);
                            package_progress.set_margin_start(10);
                            package_progress.set_margin_end(10);
                            package_progress.set_margin_bottom(10);
                            package_progress.set_visible(false); // Initially hidden
                            
                            package_box.append(&top_box);
                            package_box.append(&package_progress);
                            
                            let package_name = package_name.to_string();
                            update_button.connect_clicked(clone!(@weak status_label, @strong package_name, @weak package_progress => move |btn| {
                                let package_name = package_name.clone();
                                
                                // Disable the button during update
                                btn.set_sensitive(false);
                                
                                // Show progress bar
                                package_progress.set_visible(true);
                                package_progress.set_fraction(0.0);
                                package_progress.set_text(Some("Starting update..."));
                                
                                glib::spawn_future_local(clone!(@weak status_label, @strong package_name, @weak package_progress, @weak btn => async move {
                                    match install_with_progress(&package_name, package_progress.clone()).await {
                                        Ok(_) => {
                                            status_label.set_markup(&format!(
                                                "<span size='large' weight='bold'>{} updated successfully!</span>",
                                                package_name
                                            ));
                                            btn.set_label("Updated");
                                            btn.set_sensitive(false);
                                        },
                                        Err(e) => {
                                            status_label.set_markup(&format!(
                                                "<span size='large' weight='bold' color='red'>Error updating {}: {}</span>",
                                                package_name, e
                                            ));
                                            // Re-enable the button on failure
                                            btn.set_sensitive(true);
                                        }
                                    }
                                }));
                            }));

                            packages_box.append(&package_box);
                        }
                    }
                },
                Ok(_) => {
                    status_label.set_markup("<span size='large' weight='bold'>Your system is up to date!</span>");
                },
                Err(e) => {
                    status_label.set_markup(&format!(
                        "<span size='large' weight='bold' color='red'>Error checking updates: {}</span>",
                        e
                    ));
                }
            }
        }));
    }));

    vbox.append(&update_button);
    vbox.append(&seperator);
    vbox.append(&status_label);
    vbox.append(&main_progress_bar);
    vbox.append(&scroll);
    
    let nav_bar = GtkBox::new(Orientation::Horizontal, 24);
    nav_bar.set_valign(gtk::Align::End);
    nav_bar.set_halign(gtk::Align::Center);
    nav_bar.set_margin_top(6);
    nav_bar.set_margin_bottom(6);
    
    fn create_nav_button(icon_name: &str, label: &str) -> gtk::Button {
        let button = gtk::Button::new();
        button.set_valign(gtk::Align::Center);
        button.set_halign(gtk::Align::Center);
        button.add_css_class("nav-button");
    
        let icon = gtk::Image::from_icon_name(icon_name);
        icon.add_css_class("nav-icon");
    
        let label = gtk::Label::new(Some(label));
        label.set_valign(gtk::Align::Center);
        label.set_halign(gtk::Align::Center);
    
        let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        content.set_valign(gtk::Align::Center);
        content.set_halign(gtk::Align::Center);
        content.append(&icon);
        content.append(&label);
    
        button.set_child(Some(&content));
        button
    }
    
    let updates_button = create_nav_button("system-software-update", "Updates");
    let home_button = create_nav_button("go-home", "Home");
    let settings_button = create_nav_button("preferences-system", "Settings");
    
    nav_bar.append(&updates_button);
    nav_bar.append(&home_button);
    nav_bar.append(&settings_button);
    
    home_button.connect_clicked(clone!(@weak vbox => move |_| {
        home(&vbox);
    }));
    vbox.append(&nav_bar);
}
