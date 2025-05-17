use adw::{Application, prelude::*};
use gtk::{Orientation, Box as GtkBox};
mod home;
mod search;
mod updates;

use home::home;
#[tokio::main]
async fn main() {
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
    let vbox = GtkBox::new(Orientation::Vertical, 0);
    home(&vbox);
    win.set_content(Some(&vbox));
    win.show();
}
