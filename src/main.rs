mod updates;
use updates::update;

mod search;
use search::search_pacman;

use adw::prelude::*;
use adw::Application;
use gtk::{Box as GtkBox, Orientation, Entry, glib::clone};



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
    search_entry.connect_activate({
    let search_entry = glib::clone::Downgrade::downgrade(&search_entry);
    move|_|{
        {
            #[deprecated = "Using old-style clone! syntax"]
            macro_rules! clone {
                () => {}
                ;
            }
            clone!();
        }{
            let search_entry = match glib::clone::Upgrade::upgrade(&search_entry){
                Some(val) => val,
                None => {
                    glib::g_debug!(glib::CLONE_MACRO_LOG_DOMAIN,"Failed to upgrade search_entry",);
                    return;
                }
            
                };
            {
                let query = search_entry.text();
                let backend_query = query.replace(" ","-");
                if!query.is_empty(){
                    search_pacman(&backend_query, &app_list, &info_revealer, &info_center, &separator_revealer);
                }
            }
        }
    }
});
    let nav_bar = GtkBox::new(Orientation::Horizontal, 24);
    nav_bar.set_valign(gtk::Align::End);
    nav_bar.set_halign(gtk::Align::Center);
    nav_bar.set_margin_top(6);
    nav_bar.set_margin_bottom(6);
    // A helper to create a nav button with icon and label vertically stacked
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
    
    // Create buttons 
    let updates_button = create_nav_button("system-software-update-symbolic", "Updates");
    let home_button = create_nav_button("go-home-symbolic", "Home");
    let settings_button = create_nav_button("preferences-system-symbolic", "Settings");
    
    // Append them to nav bar 
    nav_bar.append(&updates_button);
    nav_bar.append(&home_button);
    nav_bar.append(&settings_button);
     
    // Add nav_bar at the bottom of your main VBox
    vbox.append(&nav_bar);
    
    updates_button.connect_clicked(clone!(@weak vbox, @weak header => move |_| {
        update(&vbox, &header);
    }));

    win.set_content(Some(&vbox));
    win.show();
}
