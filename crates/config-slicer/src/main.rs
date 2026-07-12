//! Configuration Slicer CLI Tool (binary shim)

fn main() {
    let _ = config_slicer::run(std::env::args_os());
}
