//! `config-slicer` binary entrypoint.

fn main() {
    if let Err(error) = config_slicer::run(std::env::args_os()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
