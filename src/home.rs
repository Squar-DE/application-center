use crate::updates::update;

use crate::search::search_pacman;

use adw::prelude::*;
use gtk::{Box as GtkBox, Orientation, Entry, glib::clone};


#[allow(deprecated)] // these warnings are way too annoying and clutter everything
pub fn home(vbox: &GtkBox) {
    // Clear all existing children
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }
    let header = adw::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("Home -- Application Center")))
        .show_end_title_buttons(true)
        .build();
    vbox.append(&header);
    let search_box = GtkBox::new(Orientation::Horizontal, 0);
    search_box.add_css_class("linked");
    search_box.set_halign(gtk::Align::Center);
     
    search_box.set_margin_top(15); // gotta make the Updates button and search box match
    search_box.set_margin_bottom(5);
     
    let search_entry = Entry::new();
    let search_button = create_nav_button("system-search", "");
    
    search_box.append(&search_entry);
    
    search_box.append(&search_button);
       vbox.append(&search_box);
    
    let seperator = gtk::Separator::new(Orientation::Horizontal);
    seperator.set_margin_top(10);
    vbox.append(&seperator);
    
    let content_box = GtkBox::new(Orientation::Horizontal, 12);
    vbox.append(&content_box);
    
    let app_list = gtk::FlowBox::new();
    app_list.set_valign(gtk::Align::Start);
    app_list.set_max_children_per_line(3);
    app_list.set_selection_mode(gtk::SelectionMode::None);
    let app_list_scroll = gtk::ScrolledWindow::builder()
        .child(&app_list)
        .vexpand(true)
        .hexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .build();
    
    content_box.append(&app_list_scroll);
    
    let separator_revealer = gtk::Revealer::new();
    separator_revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
    separator_revealer.set_transition_duration(300);

    let vertical_separator = gtk::Separator::new(Orientation::Vertical);
    vertical_separator.set_margin_top(10);
    vertical_separator.set_margin_bottom(10);
    separator_revealer.set_child(Some(&vertical_separator)); 
    content_box.append(&separator_revealer);
    content_box.append(&vertical_separator);
    
    let info_revealer = gtk::Revealer::new();
    info_revealer.set_transition_type(gtk::RevealerTransitionType::SlideLeft);
    info_revealer.set_transition_duration(300);
    
    let info_center = GtkBox::new(Orientation::Vertical, 12);
    info_center.set_margin_top(12);
    info_center.set_margin_start(12);
    info_center.set_margin_end(12);
    info_center.set_width_request(200);
    info_center.set_vexpand(true);
    
    info_revealer.set_child(Some(&info_center));
    content_box.append(&info_revealer);
    
    search_entry.connect_activate(clone!(@weak search_entry, @weak app_list, @weak info_revealer, @weak info_center, @weak separator_revealer => move |_| {
        let query = search_entry.text();
        let backend_query = query.replace(" ","-");
        if !query.is_empty() {
            search_pacman(&backend_query, &app_list, &info_revealer, &info_center, &separator_revealer);
        }
    }));

    search_button.connect_clicked(clone!(@weak search_entry, @weak app_list, @weak info_revealer, @weak info_center, @weak separator_revealer => move |_| {
        let query = search_entry.text();
        let backend_query = query.replace(" ","-");
        if !query.is_empty() {
            search_pacman(&backend_query, &app_list, &info_revealer, &info_center, &separator_revealer);
        }
    }));

    
    let nav_bar = GtkBox::new(Orientation::Horizontal, 24);
    nav_bar.set_valign(gtk::Align::End);
    nav_bar.set_halign(gtk::Align::Center);
    nav_bar.set_margin_top(6);
    nav_bar.set_margin_bottom(6);
    
    fn create_nav_button(icon_name: &str, label: &str) -> gtk::Button {
        let button = gtk::Button::new();
        button.set_valign(gtk::Align::Center);
        button.set_halign(gtk::Align::Center);
    
        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(16);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        content.set_valign(gtk::Align::Center);
        content.set_halign(gtk::Align::Center);
        if label != "" {
            let label = gtk::Label::new(Some(label));
            label.set_valign(gtk::Align::Center);
            label.set_halign(gtk::Align::Center);
            content.append(&icon);
            content.append(&label);
        } else {
            println!("yo"); // i'm keeping this cuz it's funny LOL
            content.append(&icon);
        }

    
        button.set_child(Some(&content));
        button
    }
    
    let updates_button = create_nav_button("system-software-update", "Updates");
    let home_button = create_nav_button("go-home", "Home");
    let settings_button = create_nav_button("preferences-system", "Settings");
    
    nav_bar.append(&updates_button);
    nav_bar.append(&home_button);
    nav_bar.append(&settings_button);
     
    vbox.append(&nav_bar);
    
    updates_button.connect_clicked(clone!(@weak vbox => move |_| {
        update(&vbox);
    }));
}
