use std::{cell::Cell, env, fs::File, path::Path, rc::Rc};

use gtk::{
    gdk::Display,
    glib::{self, clone},
    prelude::*,
    Button, CssProvider,
};

use clap::{arg, Parser};

/// Rewrite of wlogout in Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of buttons
    #[arg(short, long, default_value_t = 1)]
    number_of_buttons: u8,
}

fn main() -> glib::ExitCode {
    // todo: process_args
    let args = Args::parse();

    println!("number of buttons hehe: {}", args.number_of_buttons);

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

    let app = gtk::Application::builder()
        .application_id("rlogout")
        .build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &gtk::Application) {
    // test area
    let number = Rc::new(Cell::new(0));
    let button_increase = Button::builder()
        .label("+++")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(50)
        .margin_end(50)
        .build();
    let button_decrease = Button::builder()
        .label("---")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(50)
        .margin_end(50)
        .build();
    button_decrease.connect_clicked(clone!(@strong number => move |_| {
        number.set(number.get() - 1);
        println!("clickety!! {}", number.get());
    }));
    button_increase.connect_clicked(move |_| {
        number.set(number.get() + 1);
        println!("clickety!! {}", number.get());
    });

    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    gtk_box.append(&button_increase);
    gtk_box.append(&button_decrease);
    // test area

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .child(&gtk_box)
        .build();
    // window.set_fullscreened(true);
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

fn load_css() {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    let home = format!("{home}/.config/wlogout/style.css");
    let provider = CssProvider::new();
    let path = Path::new(&home);
    provider.load_from_path(path);

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
