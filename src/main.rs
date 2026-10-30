#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    if let Err(error) = codexpet::platform::run() {
        if std::env::args().any(|arg| arg == "--preview" || arg == "--smoke-test") {
            eprintln!("CodexPet: {error}");
        } else {
            codexpet::platform::show_error(&error.to_string());
        }
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    eprintln!("CodexPet currently supports Windows only.");
}
