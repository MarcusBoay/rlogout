use std::{
    env,
    io::{self, Write},
    path::Path,
    process::Command,
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
#[derive(Parser, Debug, Clone)]
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

    /// Use layer-shell or xdg protocol
    #[arg(short, long)]
    protocol: Option<String>, // todo

    /// Show the keybinds on their corresponding button
    #[arg(short, long, default_value_t = false)]
    show_binds: bool,

    /// Stops from spanning across multiple monitors
    #[arg(short, long, default_value_t = false)]
    no_span: bool, // todo

    /// Set the primary monitor
    #[arg(short = 'P', long)]
    primary_monitor: Option<u32>, // todo
}

#[derive(Serialize, Deserialize, Clone)]
struct ButtonData {
    label: String,
    action: String,
    text: String,
    keybind: String,
}

fn main() -> glib::ExitCode {
    let args = Rc::new(Args::parse());

    let app = gtk::Application::builder()
        .application_id("rlogout")
        .build();
    app.connect_startup(clone!(@strong args => move |_| load_css(&args)));
    app.connect_activate(clone!(@weak app, @strong args => move |_| build_ui(&app, &args)));
    let empty: Vec<String> = vec![];
    app.run_with_args(&empty) // workaround to make clap parse arguments
}

fn build_ui(app: &gtk::Application, args: &Args) {
    let grid = gtk::Grid::builder()
        .margin_top(args.margin_top.try_into().unwrap())
        .margin_bottom(args.margin_bottom.try_into().unwrap())
        .margin_start(args.margin_left.try_into().unwrap())
        .margin_end(args.margin_right.try_into().unwrap())
        .row_spacing(args.row_spacing.try_into().unwrap())
        .column_spacing(args.column_spacing.try_into().unwrap())
        .build();

    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    gtk_box.append(&grid);
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(clone!(@weak app => move |gesture, _, _, _| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        app.quit();
    }));
    gtk_box.add_controller(gesture);
    let esc_event = gtk::EventControllerKey::new();
    esc_event.connect_key_released(clone!(@weak app => move |_, key, _, _| {
        if key.name().is_some_and(|k| k == "Escape") {
            app.quit();
        }
    }));
    gtk_box.add_controller(esc_event);

    let buttons = build_buttons(&app, &gtk_box, &args);
    let mut i: u32 = 0; // row
    loop {
        let mut break_out = false;
        for j in 0..args.buttons_per_row {
            let k: usize = (i * args.buttons_per_row + j).try_into().unwrap();
            if k >= buttons.len() {
                break_out = true;
                break;
            }
            let button = &buttons[k];
            grid.attach(button, j.try_into().unwrap(), i.try_into().unwrap(), 1, 1);
        }
        if break_out {
            break;
        }
        i += 1;
    }

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .child(&gtk_box)
        .build();
    window.set_fullscreened(true);
    window.present();
}

fn build_buttons(app: &gtk::Application, gtk_box: &gtk::Box, args: &Args) -> Vec<Button> {
    let layout_path = get_layout_path(&args);
    let layout_path = match layout_path {
        Ok(layout_path) => layout_path,
        _ => panic!("{}\n", layout_path.unwrap_err()), // TODO: how to handle error instead of panicking?
    };
    let json = std::fs::read_to_string(layout_path).unwrap(); // todo: handle error properly
    let json: Value = serde_json::from_str(&json).unwrap(); // todo: handle error properly
    let json = json.as_array().unwrap();
    let margin: i32 = args.margin.try_into().unwrap();
    let mut buttons: Vec<Button> = vec![];
    for button_json in json {
        let button_data: ButtonData = serde_json::from_value(button_json.clone()).unwrap(); // todo: handle error properly
        let button_data_clone = button_data.clone();
        let label_text = if args.show_binds {
            button_data.text + "[" + &button_data.keybind + "]"
        } else {
            button_data.text
        };
        let button: Button = Button::builder()
            .name(button_data.label.clone())
            .label(label_text)
            .margin_top(margin)
            .margin_bottom(margin)
            .margin_start(margin)
            .margin_end(margin)
            .hexpand(true)
            .vexpand(true)
            // .css_name(button_data.label)
            .build();
        let action_fn = Rc::new(move |app: &gtk::Application| {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&button_data.action)
                .output()
                .expect("failed to execute process");
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
            app.quit();
        });
        let action_fn_clone = action_fn.clone();
        button.connect_clicked(clone!(@weak app => move |_| action_fn(&app)));
        let key_event = gtk::EventControllerKey::new();
        key_event.connect_key_released(clone!(@weak app => move |_, key, _, _| {
            if key.name().is_some_and(|k| k == button_data_clone.keybind) {
                action_fn_clone(&app);
            }
        }));
        gtk_box.add_controller(key_event);

        buttons.push(button);
    }
    buttons
}

fn get_config_path(
    file: &str,
    config_path: &Option<String>,
    err_text: &'static str,
) -> Result<String, &'static str> {
    let xdg_config_home = env::var("XDG_CONFIG_HOME");
    let mut default_config_path = String::new();
    if xdg_config_home.is_err() {
        let home = env::var("HOME");
        if home.is_err() {
            return Err("Cannot find environment variable: HOME");
        }
        default_config_path = home.unwrap() + "/.config";
    }
    default_config_path = default_config_path + "/rlogout/" + file;

    if config_path.is_some()
        && Path::new(format!("{}", &config_path.as_ref().unwrap()).as_str()).exists()
    {
        Ok(config_path.clone().unwrap())
    } else if Path::new(&default_config_path).exists() {
        Ok(default_config_path)
    } else if Path::new(&format!("/etc/rlogout/{}", &file)).exists() {
        Ok(String::from(&format!("/etc/rlogout/{}", &file)))
    } else if Path::new(&format!("/usr/local/etc/rlogout/{}", &file)).exists() {
        Ok(String::from(&format!("/usr/local/etc/rlogout/{}", &file)))
    } else {
        Err(err_text)
    }
}

fn get_layout_path(args: &Args) -> Result<String, &'static str> {
    get_config_path("layout.json", &args.layout, "Failed to find a layout")
}

fn get_css_path(args: &Args) -> Result<String, &'static str> {
    get_config_path("style.css", &args.css, "Failed to find css file")
}

fn load_css(args: &Args) {
    let css_path = get_css_path(&args);
    let css_path = match css_path {
        Ok(css_path) => css_path,
        _ => panic!("{}\n", css_path.unwrap_err()), // TODO: how to handle error instead of panicking?
    };
    let provider = CssProvider::new();
    let path = Path::new(&css_path);
    provider.load_from_path(path);

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
