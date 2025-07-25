use std::process;

fn main() {
    // save colors
    if let Err(e) = pally::run_cli() {
        eprintln!("Error: {e}");
        process::exit(1);
    };
}