use std::{env, fs::File, path::Path};

use gtk4::prelude::*;

fn main() {
    let application = gtk4::Application::builder().application_id("test").build();
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(application);
    window.set_fullscreened(true);

    // todo: process_args

    if !get_layout_path() {
        panic!("Failed to find a layout\n"); // TODO: how to handle error instead of panicking?
    }

    if !get_css_path() {
        panic!("Failed to find a css file\n"); // TODO: how to handle error instead of panicking?
    }

    // open layout path
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    let layout_path = format!("{home}/.config/wlogout/layout");
    let mut file = File::open(layout_path.as_str());
    if file.is_err() {
        panic!("Failed to open {layout_path}"); // TODO: how to handle error instead of panicking?
    }

    // todo: get_buttons

    window.present();
}

// process_args
// todo: figure out how to process args

fn get_layout_path() -> bool {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    Path::new(format!("{home}/.config/wlogout/layout").as_str()).exists()
}

fn get_css_path() -> bool {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    Path::new(format!("{home}/.config/wlogout/style.css").as_str()).exists()
}

// get_buttons
// todo: figure out how to process jsonc
