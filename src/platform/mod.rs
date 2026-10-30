#![allow(unsafe_op_in_unsafe_fn)]
//! Native Windows integration. Raw handles are kept inside this module tree.
mod app;
mod graphics;
mod install;
mod processes;
mod startup;
mod tray;
mod updates;

pub use app::run;
pub use processes::codex_processes;

use std::{
    fs, io,
    path::PathBuf,
    ptr::{null, null_mut},
};
use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::*};
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn native_error() -> io::Error {
    io::Error::last_os_error()
}
pub fn show_error(error: &str) {
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(error).as_ptr(),
            wide("CodexPet").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

unsafe fn open_url(url: &str) {
    ShellExecuteW(
        null_mut(),
        wide("open").as_ptr(),
        wide(url).as_ptr(),
        null(),
        null(),
        SW_SHOWNORMAL,
    );
}
fn log_error(error: &str) {
    if let Some(dir) = std::env::var_os("APPDATA") {
        use std::io::Write;
        let dir = PathBuf::from(dir).join("CodexPet");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("codexpet.log");
        if fs::metadata(&path).is_ok_and(|m| m.len() > 1024 * 1024) {
            let _ = fs::remove_file(&path);
        }
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{} {error}", chrono::Utc::now().to_rfc3339());
        }
    }
}
