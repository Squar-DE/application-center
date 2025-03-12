use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Button, Entry, Grid, Label, Orientation, ScrolledWindow, 
    CssProvider, Revealer, RevealerTransitionType, Box as GtkBox, Dialog, 
    ResponseType, PasswordEntry
};
use std::process::Command;
use strsim::levenshtein;

fn main() {
    let app = Application::builder()
        .application_id("com.example.AppStore")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("SquarDE App Store")
            .default_width(800)
            .default_height(600)
            .build();

        let provider = CssProvider::new();
        provider.load_from_data(
            ".rounded-entry { border-radius: 20px; font-size: 120%; }
             .app-tile { padding: 20px; border-radius: 10px; min-height: 200px; font-size: 150%; transition: all 0.25s ease-in-out; }
             .app-tile:hover { background-color: #575757; }
             .app-tile:active { background-color: #424242; }
             .details-box { padding: 20px; background: #1e1e1e; min-width: 300px; color: white; transition: all 0.5s ease-in-out; }
             .install-button { margin: 10px; padding: 8px 16px; border-radius: 8px; font-size: 110%; font-weight: bold; background: @accent_bg_color; color: @accent_fg_color; }
            .installed-icon { -gtk-icon-source: url('resources/installed.png'); min-width: 24px; min-height: 24px; }"
        );

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("No display found"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let main_box = GtkBox::new(gtk4::Orientation::Horizontal, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);

        let left_box = GtkBox::new(Orientation::Vertical, 10);
        left_box.set_hexpand(true);
        left_box.set_vexpand(true);
        let search_entry = Entry::new();
        search_entry.set_placeholder_text(Some("Search for an app..."));
        search_entry.add_css_class("rounded-entry");

        let search_button = Button::with_label("Search");
        let grid = Grid::new();
        grid.set_hexpand(true);
        grid.set_vexpand(true);
        grid.set_column_spacing(20);
        grid.set_row_spacing(20);

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_hexpand(true);
        scrolled_window.set_vexpand(true);
        scrolled_window.set_child(Some(&grid));

        left_box.append(&search_entry);
        left_box.append(&search_button);
        left_box.append(&scrolled_window);
        main_box.append(&left_box);

        // Right Panel for Details
        let details_revealer = Revealer::new();
        details_revealer.set_transition_type(RevealerTransitionType::SlideLeft);
        details_revealer.set_reveal_child(false);

        let details_box = GtkBox::new(Orientation::Vertical, 10);
        details_box.add_css_class("details-box");
        details_box.set_vexpand(true);

        let details_title = Label::new(None);
        details_title.set_markup("<b>App Name</b>");

        let details_version = Label::new(Some("Version:"));
        details_version.add_css_class("dim-label");

        let details_description = Label::new(Some("Description here..."));
        details_description.set_wrap(true);

        let install_button = Button::with_label("Install");
        install_button.add_css_class("install-button");

        let details_title_clone = details_title.clone();
        install_button.connect_clicked(move |_| {
            let package_name = details_title_clone.text().to_string();
            if package_name.is_empty() {
                return;
            }
            
           // Use GTK's built-in authentication with pkexec
           let output = Command::new("pkexec")
               .arg("pacman")
               .arg("-S")
               .arg(&package_name)
               .arg("--noconfirm")
               .output();
       
           match output {
               Ok(output) => {
                   if output.status.success() {
                       println!("Installation successful: {}", package_name);
                   } else {
                       eprintln!("Installation failed: {}", String::from_utf8_lossy(&output.stderr));
                   }
               }
               Err(err) => eprintln!("Failed to execute command: {}", err),
           }
        });

        details_box.append(&details_title);
        details_box.append(&details_version);
        details_box.append(&details_description);
        details_box.append(&install_button);
        details_revealer.set_child(Some(&details_box));
        main_box.append(&details_revealer);

        let search_entry_clone = search_entry.clone();
        let grid_clone = grid.clone();
        let details_revealer_clone = details_revealer.clone();
        let details_title_clone = details_title.clone();
        let details_version_clone = details_version.clone();
        let details_description_clone = details_description.clone();

        search_button.connect_clicked(move |_| {
            let query = search_entry_clone.text().trim().to_lowercase();
            let output = Command::new("pacman")
                .args(["-Ss", query.as_str()])
                .output();

            match output {
                Ok(output) => {
                    let result = String::from_utf8_lossy(&output.stdout);
                    while let Some(child) = grid_clone.first_child() {
                        child.unparent();
                    }

                    let mut packages: Vec<(String, String, String, bool, usize)> = Vec::new();
                    let mut current_package: Option<(String, String, bool)> = None;

                    for line in result.lines() {
                        if line.starts_with(" ") {
                            if let Some((title, version, installed)) = current_package.take() {
                                packages.push((
                                    title.clone(),
                                    version.clone(),
                                    line.trim().to_string(),
                                    installed,
                                    levenshtein(&query, &title),
                                ));
                            }
                        } else {
                            let clean_line = line.split_once('/').map(|(_, rest)| rest).unwrap_or(line);
                            let parts: Vec<&str> = clean_line.splitn(3, ' ').collect();
                            let title = parts.get(0).unwrap_or(&"").to_string();
                            let version = parts.get(1).unwrap_or(&"Unknown").to_string();
                            let installed = version.contains("[installed]");
                            let clean_version = version.replace("[installed]", "").trim().to_string();
                            current_package = Some((title, clean_version, installed));
                        }
                    }

                    packages.sort_by_key(|(_, _, _, _, score)| *score);
                   let mut row = 0;
                    let mut col = 0;
                    for (title, version, description, _installed, _) in packages {
                        let button = Button::new();
                        button.set_hexpand(true);
                        button.set_vexpand(false);
                        let hbox = GtkBox::new(Orientation::Horizontal, 5);
                        let label = Label::new(Some(&title));
                        label.set_hexpand(true);
                        label.set_vexpand(false);
                        hbox.append(&label);
                        button.set_child(Some(&hbox));
                        button.add_css_class("app-tile");
                        button.set_size_request(-1, 60);  
                        let details_revealer_clone = details_revealer_clone.clone();
                        let details_title_clone = details_title_clone.clone();
                        let details_version_clone = details_version_clone.clone();
                        let details_description_clone = details_description_clone.clone();
                        
                        button.connect_clicked(move |_| {
                        details_title_clone.set_markup(&format!("<b>{}</b>", title));
                        details_version_clone.set_text(&format!("Version: {}", version));
                        details_description_clone.set_text(&description);
                        details_revealer_clone.set_reveal_child(true);
        });
                        
                        // Attach the button at the correct position in the grid
                        grid_clone.attach(&button, col, row, 1, 1);
                        
                        col += 1;
                        if col >= 3 { // Adjust number of columns per row
                        col = 0;
                        row += 1;
                        }
                    }
                       
                },
                Err(_) => {}
            }
        });

        window.set_child(Some(&main_box));
        window.show();
    });

    app.run();
}
