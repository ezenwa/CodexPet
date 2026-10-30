use super::wide;
use std::{
    io,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{Foundation::*, System::Registry::*};
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "CodexPetWatcher";
pub(super) fn startup_enabled() -> bool {
    unsafe {
        let mut key = null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wide(RUN_KEY).as_ptr(),
            0,
            KEY_READ,
            &mut key,
        ) != 0
        {
            return false;
        }
        let result = RegQueryValueExW(
            key,
            wide(RUN_VALUE).as_ptr(),
            null(),
            null_mut(),
            null_mut(),
            null_mut(),
        ) == 0;
        RegCloseKey(key);
        result
    }
}
pub(super) fn set_startup(enabled: bool) -> io::Result<()> {
    unsafe {
        let mut key = null_mut();
        let code = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide(RUN_KEY).as_ptr(),
            0,
            null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            null(),
            &mut key,
            null_mut(),
        );
        if code != 0 {
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        let code = if enabled {
            let exe = std::env::current_exe()?;
            let value = wide(&format!("\"{}\" --watch", exe.display()));
            RegSetValueExW(
                key,
                wide(RUN_VALUE).as_ptr(),
                0,
                REG_SZ,
                value.as_ptr().cast(),
                (value.len() * 2) as u32,
            )
        } else {
            RegDeleteValueW(key, wide(RUN_VALUE).as_ptr())
        };
        RegCloseKey(key);
        if code != 0 && code != ERROR_FILE_NOT_FOUND {
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        Ok(())
    }
}
