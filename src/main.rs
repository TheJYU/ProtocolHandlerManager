mod config;
mod registry;
mod handler;
mod gui;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // The first argument is the program name.
    // If there is at least one argument after the program name, and the first one contains ":",
    // treat it as a URL to handle.
    if args.len() > 1 && args[1].contains(':') {
        handler::handle_url(&args[1]);
    } else {
        // Run the GUI configuration tool
        gui::run_gui();
    }
}
