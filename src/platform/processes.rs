use super::{native_error, wide};
use std::mem::zeroed;
use std::{
    io,
    ptr::null,
    time::{Duration, SystemTime},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{Diagnostics::ToolHelp::*, Threading::*},
};
pub(super) struct Mutex(HANDLE);
impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}
pub(super) fn single_instance(name: &str) -> io::Result<Option<Mutex>> {
    unsafe {
        let handle = CreateMutexW(null(), 1, wide(name).as_ptr());
        if handle.is_null() {
            return Err(native_error());
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(handle);
            Ok(None)
        } else {
            Ok(Some(Mutex(handle)))
        }
    }
}

pub fn codex_processes() -> Vec<SystemTime> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Vec::new();
        }
        let mut entry: PROCESSENTRY32W = zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut result = Vec::new();
        let mut found = Process32FirstW(snapshot, &mut entry);
        while found != 0 {
            let end = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..end]).eq_ignore_ascii_case("codex.exe") {
                let process =
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID);
                let mut created: FILETIME = zeroed();
                let mut exited = zeroed();
                let mut kernel = zeroed();
                let mut user = zeroed();
                let time = if !process.is_null()
                    && GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user)
                        != 0
                {
                    let ticks =
                        ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64;
                    SystemTime::UNIX_EPOCH
                        + Duration::from_nanos(ticks.saturating_sub(116444736000000000) * 100)
                } else {
                    SystemTime::UNIX_EPOCH
                };
                if !process.is_null() {
                    CloseHandle(process);
                }
                result.push(time);
            }
            found = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        result
    }
}
