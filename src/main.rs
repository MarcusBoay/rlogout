use std::{
    cell::Cell,
    env,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    rc::Rc,
};

use gtk::{
    gdk::Display,
    glib::{self, clone},
    prelude::*,
    Button, CssProvider,
};

use clap::{arg, Parser};

use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

/// Rewrite of wlogout in Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Specify a layout file
    #[arg(short, long)]
    layout: Option<String>,

    /// Specify a css file
    #[arg(short = 'C', long)]
    css: Option<String>,

    /// Set the number of buttons per row
    #[arg(short, long, default_value_t = 3)]
    buttons_per_row: u32,

    /// Set space between buttons columns
    #[arg(short, long, default_value_t = 0)]
    column_spacing: u32,

    /// Set space between buttons rows
    #[arg(short, long, default_value_t = 0)]
    row_spacing: u32,

    /// Set margin around buttons
    #[arg(short, long, default_value_t = 0)]
    margin: u32,

    /// Set margin for left of buttons
    #[arg(short = 'L', long, default_value_t = 230)]
    margin_left: u32,

    /// Set margin for right of buttons
    #[arg(short = 'R', long, default_value_t = 230)]
    margin_right: u32,

    /// Set margin for right of buttons
    #[arg(short = 'T', long, default_value_t = 230)]
    margin_top: u32,

    /// Set margin for right of buttons
    #[arg(short = 'B', long, default_value_t = 230)]
    margin_bottom: u32,

    /// Use layer-shell or xdg protocol (todo: ???)
    #[arg(short, long)]
    protocol: Option<String>,

    /// Show the keybinds on their corresponding button
    #[arg(short, long, default_value_t = false)]
    show_binds: bool,

    /// Stops from spanning across multiple monitors (todo: ???)
    #[arg(short, long, default_value_t = false)]
    no_span: bool,

    /// Set the primary monitor
    #[arg(short = 'P', long)]
    primary_monitor: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct ButtonData {
    label: String,
    action: String,
    text: String,
    keybind: String,
}

fn main() -> glib::ExitCode {
    // todo: process_args
    let args = Args::parse();

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
    app.connect_activate(clone!(@weak app => move |_| build_ui(&app, &args)));
    let empty: Vec<String> = vec![];
    app.run_with_args(&empty) // workaround to make clap parse arguments
}

fn build_ui(app: &gtk::Application, args: &Args) {
    // test area
    // let number = Rc::new(Cell::new(0));
    // button_decrease.connect_clicked(clone!(@strong number => move |_| {
    //     number.set(number.get() - 1);
    //     println!("clickety!! {}", number.get());
    // }));
    // button_increase.connect_clicked(move |_| {
    //     number.set(number.get() + 1);
    //     println!("clickety!! {}", number.get());
    // });
    // Create buttons
    let buttons = build_buttons(&args);

    let grid = gtk::Grid::builder()
        .margin_top(args.margin_top.try_into().unwrap())
        .margin_bottom(args.margin_bottom.try_into().unwrap())
        .margin_start(args.margin_left.try_into().unwrap())
        .margin_end(args.margin_right.try_into().unwrap())
        .build();

    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    gtk_box.append(&grid);
    for j in 0..3 {
        // fixme: why does this not create more buttons in row?
        for (i, button) in buttons.iter().enumerate() {
            grid.attach(button, i.try_into().unwrap(), j.try_into().unwrap(), 1, 1);
        }
    }

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .child(&gtk_box)
        .build();
    window.set_fullscreened(true);
    window.present();
}

// todo: get actual layout path
fn build_buttons(args: &Args) -> Vec<Button> {
    let json = std::fs::read_to_string("layout.json").unwrap(); // todo: handle error properly
    let json: Value = serde_json::from_str(&json).unwrap(); // todo: handle error properly
                                                            // end test area
    let json = json.as_array().unwrap();
    let margin: i32 = args.margin.try_into().unwrap();
    let mut buttons: Vec<Button> = vec![];
    for button_json in json {
        let button_data: ButtonData = serde_json::from_value(button_json.clone()).unwrap(); // todo: handle error properly
        let button: Button = Button::builder()
            .name(button_data.label)
            .label(button_data.text)
            .margin_top(margin)
            .margin_bottom(margin)
            .margin_start(margin)
            .margin_end(margin)
            .hexpand(true)
            .vexpand(true)
            .build();
        // todo: command for button
        buttons.push(button);
    }
    buttons
}

// todo: fix this nonsense
fn get_layout_path() -> bool {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    Path::new(format!("{home}/.config/wlogout/layout").as_str()).exists()
}

// todo: fix this nonsense
fn get_css_path() -> bool {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    Path::new(format!("{home}/.config/wlogout/style.css").as_str()).exists()
}

// todo: clean up this nonsense
fn load_css() {
    let home = env::var("HOME");
    if home.is_err() {
        panic!("Cannot find home.");
    }
    let home = home.unwrap();
    let home = format!("{home}/.config/wlogout/style.css");
    // let home = format!("style.css");
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
