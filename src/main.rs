use std::{
    env,
    io::{self, Write},
    path::Path,
    process::Command,
    rc::Rc,
};

use gtk::{
    gdk::{self, Display, Monitor},
    glib::{self, clone},
    prelude::*,
};

use clap::{arg, Parser};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
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

    /// Automatically center buttons and set margins
    #[arg(short, long, default_value_t = false)]
    auto_margin: bool,

    /// Set height of button
    #[arg(short = 'H', long, required_if_eq("auto_margin", "true"))]
    button_height: Option<u32>,

    /// Set height of button
    #[arg(short = 'W', long, required_if_eq("auto_margin", "true"))]
    button_width: Option<u32>,

    /// Set margin for left of buttons
    #[arg(
        short = 'L',
        long,
        default_value_t = 230,
        conflicts_with("auto_margin")
    )]
    margin_left: u32,

    /// Set margin for right of buttons
    #[arg(
        short = 'R',
        long,
        default_value_t = 230,
        conflicts_with("auto_margin")
    )]
    margin_right: u32,

    /// Set margin for right of buttons
    #[arg(
        short = 'T',
        long,
        default_value_t = 230,
        conflicts_with("auto_margin")
    )]
    margin_top: u32,

    /// Set margin for right of buttons
    #[arg(
        short = 'B',
        long,
        default_value_t = 230,
        conflicts_with("auto_margin")
    )]
    margin_bottom: u32,

    /// Use layer-shell or xdg protocol
    #[arg(short, long, default_value_t = String::from("layer-shell"))]
    protocol: String,

    /// Show the keybinds on their corresponding button
    #[arg(short, long, default_value_t = false)]
    show_binds: bool,

    /// Stops from spanning across multiple monitors (Only for layer-shell protocol)
    #[arg(short, long, default_value_t = false)]
    no_span: bool,

    /// Set the primary monitor
    #[arg(short = 'P', long)]
    primary_monitor: Option<u32>,

    /// Disable mouse input
    #[arg(short, long, default_value_t = false)]
    disable_mouse_input: bool,

    /// Mirror window on other monitors
    #[arg(short = 'M', long, default_value_t = false)]
    mirror_window: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct ButtonData {
    label: String,
    action: String,
    text: String,
    keybind: String,
    label_x_align: Option<f32>,
    label_y_align: Option<f32>,
}

fn main() -> glib::ExitCode {
    let args = Rc::new(Args::parse());
    assert!(args.buttons_per_row > 0, "buttons_per_row must be > 0!");

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
        .row_spacing(args.row_spacing.try_into().unwrap())
        .column_spacing(args.column_spacing.try_into().unwrap())
        .build();

    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    gtk_box.append(&grid);
    add_exit_action(&app, args.disable_mouse_input, &gtk_box);

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .child(&gtk_box)
        .decorated(false)
        .build();
    let display = gdk::Display::default().expect("Failed to get default display");
    if gtk4_layer_shell::is_supported() && args.protocol.clone() == "layer-shell" {
        if let Some(mut primary_monitor) = args.primary_monitor {
            if primary_monitor >= display.monitors().n_items() {
                primary_monitor = display.monitors().n_items() - 1;
            }
            let monitor: Monitor = display
                .monitors()
                .item(primary_monitor)
                .unwrap()
                .dynamic_cast::<Monitor>()
                .unwrap();
            set_layer_shell(&window, Some(&monitor));
        } else {
            set_layer_shell(&window, None);
        }
    } else {
        window.set_fullscreened(true);
    }

    let buttons = build_buttons(&app, &window, &gtk_box, &args);
    place_buttons(args.buttons_per_row, &buttons, &grid);

    let number_of_rows =
        buttons.len() as u32 / args.buttons_per_row + buttons.len() as u32 % args.buttons_per_row;
    let args_clone = Rc::new(args.clone());
    window.connect_realize(clone!(@weak app => move |window| {
        if window.surface().is_none() {
            return;
        }
        let surface = window.surface().unwrap();
        surface.connect_enter_monitor(clone!(@weak app, @weak window, @weak grid, @strong args_clone => move |_, main_monitor: &Monitor| {
            set_grid_margin(&grid, &args_clone, &main_monitor, number_of_rows);

            if args_clone.no_span {
                return;
            }
            if !gtk4_layer_shell::is_supported() || args_clone.protocol != "layer-shell" {
                return;
            }

            // Add window to other monitors.
            let display = Display::default().expect("Failed to get default display");
            for i in 0..display.monitors().n_items() {
                let monitor: Monitor = display
                    .monitors()
                    .item(i)
                    .unwrap()
                    .dynamic_cast::<Monitor>()
                    .unwrap();
                if &monitor.description() != &main_monitor.description() {
                    let grid_i = gtk::Grid::builder()
                        .row_spacing(args_clone.row_spacing.try_into().unwrap())
                        .column_spacing(args_clone.column_spacing.try_into().unwrap())
                        .build();
                    set_grid_margin(&grid_i, &args_clone, &monitor, number_of_rows);

                    let gtk_box_i = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .build();
                    gtk_box_i.append(&grid_i);
                    add_exit_action(&app, args_clone.disable_mouse_input, &gtk_box_i);

                    let window_i = gtk::ApplicationWindow::builder()
                        .application(&app)
                        .child(&gtk_box_i)
                        .decorated(false)
                        .build();
                    set_layer_shell(&window_i, Some(&monitor));

                    // Place buttons on grid
                    if args_clone.mirror_window {
                        let buttons_i = build_buttons(&app, &window_i, &gtk_box_i, &args_clone);
                        place_buttons(args_clone.buttons_per_row, &buttons_i, &grid_i);
                    }

                    window_i.present();
                    // fixme: main window must have focus.
                }
            }
        }));
    }));

    window.present();
}

fn set_grid_margin(grid: &gtk::Grid, args: &Args, monitor: &Monitor, number_of_rows: u32) {
    if !args.auto_margin {
        grid.set_margin_top(args.margin_top.try_into().unwrap());
        grid.set_margin_bottom(args.margin_bottom.try_into().unwrap());
        grid.set_margin_start(args.margin_left.try_into().unwrap());
        grid.set_margin_end(args.margin_right.try_into().unwrap());
    } else {
        let monitor_height = monitor.geometry().height();
        let total_h = args.button_height.unwrap() * number_of_rows;
        let total_h = total_h + (args.margin * 2 * number_of_rows);
        let total_h = total_h + (number_of_rows - 1) * args.row_spacing;
        let margin = (monitor_height - total_h as i32) / 2;
        grid.set_margin_top(margin);
        grid.set_margin_bottom(margin);

        let monitor_width = monitor.geometry().width();
        let total_w = args.button_width.unwrap() * args.buttons_per_row;
        let total_w = total_w + (args.margin * 2 * args.buttons_per_row);
        let total_w = total_w + (args.buttons_per_row - 1) * args.column_spacing;
        let margin = (monitor_width - total_w as i32) / 2;
        grid.set_margin_start(margin);
        grid.set_margin_end(margin);
    }
}

// Set window properties
fn set_layer_shell(window: &gtk::ApplicationWindow, monitor: Option<&Monitor>) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    if let Some(monitor) = monitor {
        window.set_monitor(monitor);
    }
    window.set_namespace("rlogout_dialog");
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_exclusive_zone(-1); // makes sure that it is above waybar...
}

// Sets click/ESC key to quit application
fn add_exit_action(app: &gtk::Application, disable_mouse_input: bool, gtk_box: &gtk::Box) {
    if !disable_mouse_input {
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(clone!(@weak app => move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            app.quit();
        }));
        gtk_box.add_controller(gesture);
    }
    let esc_event = gtk::EventControllerKey::new();
    esc_event.connect_key_released(clone!(@weak app => move |_, key, _, _| {
        if key.name().is_some_and(|k| k == "Escape") {
            app.quit();
        }
    }));
    gtk_box.add_controller(esc_event);
}

// Place buttons in grid
fn place_buttons(buttons_per_row: u32, buttons: &Vec<gtk::Button>, grid: &gtk::Grid) {
    let mut i = 0;
    loop {
        let mut break_out = false;
        for j in 0..buttons_per_row {
            let k: usize = (i * buttons_per_row + j).try_into().unwrap();
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
}

fn build_buttons(
    app: &gtk::Application,
    window: &gtk::ApplicationWindow,
    gtk_box: &gtk::Box,
    args: &Args,
) -> Vec<gtk::Button> {
    let layout_path = get_layout_path(&args);
    let layout_path = match layout_path {
        Ok(layout_path) => layout_path,
        _ => panic!("{}\n", layout_path.unwrap_err()),
    };
    let json = std::fs::read_to_string(layout_path).unwrap();
    let json: Value = serde_json::from_str(&json).unwrap();
    let json = json.as_array().unwrap();

    let margin: i32 = args.margin.try_into().unwrap();
    let mut buttons: Vec<gtk::Button> = vec![];
    for button_json in json {
        let button_data: ButtonData = serde_json::from_value(button_json.clone()).unwrap();
        let button_data_clone = button_data.clone();

        let label_text = if args.show_binds {
            button_data.text + "[" + &button_data.keybind + "]"
        } else {
            button_data.text
        };

        let button = gtk::Button::builder()
            .name(button_data.label.clone())
            .margin_top(margin)
            .margin_bottom(margin)
            .margin_start(margin)
            .margin_end(margin)
            .build();
        if let Some(w) = args.button_width {
            button.set_width_request(w.try_into().unwrap());
        } else {
            button.set_hexpand(true);
        }
        if let Some(h) = args.button_height {
            button.set_height_request(h.try_into().unwrap());
        } else {
            button.set_vexpand(true);
        }

        let label = gtk::Label::builder().label(label_text).build();
        if let Some(x) = button_data.label_x_align {
            label.set_xalign(x);
        } else {
            label.set_xalign(0.5);
        }
        if let Some(y) = button_data.label_y_align {
            label.set_yalign(y);
        } else {
            label.set_yalign(0.9);
        }
        button.set_child(Some(&label));

        // Build action for clicking/key press
        let action_fn = Rc::new(
            move |app: &gtk::Application, window: &gtk::ApplicationWindow| {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&button_data.action)
                    .output()
                    .expect("failed to execute process");
                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();
                window.close();
                app.quit();
            },
        );
        let action_fn_clone1 = action_fn.clone();
        let action_fn_clone2 = action_fn.clone();
        if !args.disable_mouse_input {
            button.connect_clicked(
                clone!(@weak app, @weak window => move |_| action_fn(&app, &window)),
            );
        }
        let key_event = gtk::EventControllerKey::new();
        key_event.connect_key_released(clone!(@weak app, @weak window => move |_, key, _, _| {
            if key.name().is_some_and(|k| k == button_data_clone.keybind) {
                action_fn_clone1(&app, &window);
            }
        }));
        gtk_box.add_controller(key_event);

        // Build action for pressing 'Enter'/'Space'/?? key when button is focused
        button.connect_activate(clone!(@weak app, @weak window => move |_| {
            action_fn_clone2(&app, &window);
        }));

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
    let mut default_config_path = if xdg_config_home.is_err() {
        let home = env::var("HOME");
        if home.is_err() {
            return Err("Cannot find environment variable: HOME");
        }
        home.unwrap() + "/.config"
    } else {
        xdg_config_home.unwrap()
    };
    default_config_path = default_config_path + "/rlogout/" + file;

    if config_path.is_some() && Path::new(&config_path.as_ref().unwrap()).exists() {
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
        _ => panic!("{}\n", css_path.unwrap_err()),
    };
    let provider = gtk::CssProvider::new();
    let path = Path::new(&css_path);
    provider.load_from_path(path);

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}
