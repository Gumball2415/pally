use std::process;

fn main() {
    // TODO: GUI or WASM interface?
    if let Err(e) = pally::run_cli() {
        eprintln!("Error: {e}");
        process::exit(1);
    };
}
