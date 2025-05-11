use gtk::{Box as GtkBox, prelude::*, Button};
use adw::HeaderBar;


pub fn update(vbox: &GtkBox, header: &HeaderBar) {
    // Clear all existing children
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }
    vbox.append(header);
    // Create a new label or container for the updates page
    let update_label = gtk::Label::new(Some("Checking for updates..."));

    // Fade in transition
    update_label.set_opacity(0.0);
    let update_button = Button::builder()
        .label("Check for Updates")
        .halign(gtk::Align::Center)
        .height_request(100)
        .build();

    vbox.append(&update_button);

    let update_label_clone = update_label.clone();
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
        update_label_clone.set_opacity(1.0);
    });
}

