use std::{cell::Cell, env, fs::File, path::Path};

use gtk4::{glib, prelude::*, Button, Grid};

fn main() -> glib::ExitCode {
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

    let app = gtk4::Application::builder()
        .application_id("rlogout")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(app);
    window.set_fullscreened(true);

    // test area
    let number = Cell::new(0);
    let button = Button::builder()
        .label("Oclick me!!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    button.connect_clicked(move |_| {
        number.set(number.get() + 1);
        println!("clickety!! {}", number.get());
    });
    window.set_child(Some(&button));
    // test area

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

fn load_buttons() {
    let grid = Grid::new();
}
