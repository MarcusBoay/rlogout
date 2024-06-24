use std::{env, path::Path};

use gtk4::prelude::*;

fn main() {
    let application = gtk4::Application::builder().application_id("test").build();
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(application);

    window.set_title(Some("rlogout"));
    window.set_default_size(350, 70);

    // let button = gtk4::Button::with_label("Click me!");

    // window.set_child(Some(&button));

    // process_args

    // get_layout_path
    if !get_layout_path() {
        panic!("Failed to find a layout\n"); // TODO: how to handle error instead of panicking?
    }

    // get_css_path

    // open layout path

    // get_buttons

    window.present();
}

// process_args
// need to figure out how to process args

fn get_layout_path() -> bool {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    Path::new(format!("{home}/.config/wlogout/layout").as_str()).exists()
}

// get_css_path

// get_buttons
