use super::{native_error, wide};
use crate::config::VERSION;
use std::{
    error::Error,
    ffi::c_void,
    ptr::{null, null_mut},
};
use windows_sys::Win32::Networking::WinHttp::*;
pub(super) enum UpdateResult {
    Current,
    Available(String, String),
    Failed(String),
}
struct Internet(*mut c_void);
impl Drop for Internet {
    fn drop(&mut self) {
        unsafe {
            WinHttpCloseHandle(self.0);
        }
    }
}
pub(super) fn check_update() -> Result<UpdateResult, Box<dyn Error>> {
    unsafe {
        let session = Internet(WinHttpOpen(
            wide(&format!("CodexPet/{VERSION}")).as_ptr(),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            null(),
            null(),
            0,
        ));
        if session.0.is_null() {
            return Err(Box::new(native_error()));
        }
        WinHttpSetTimeouts(session.0, 5000, 5000, 5000, 5000);
        let connect = Internet(WinHttpConnect(
            session.0,
            wide("api.github.com").as_ptr(),
            443,
            0,
        ));
        if connect.0.is_null() {
            return Err(Box::new(native_error()));
        }
        let request = Internet(WinHttpOpenRequest(
            connect.0,
            wide("GET").as_ptr(),
            wide("/repos/ezenwa/CodexPet/releases/latest").as_ptr(),
            null(),
            null(),
            null(),
            WINHTTP_FLAG_SECURE,
        ));
        if request.0.is_null() {
            return Err(Box::new(native_error()));
        }
        let headers = wide("Accept: application/vnd.github+json\r\n");
        if WinHttpSendRequest(request.0, headers.as_ptr(), u32::MAX, null(), 0, 0, 0) == 0
            || WinHttpReceiveResponse(request.0, null_mut()) == 0
        {
            return Err(Box::new(native_error()));
        }
        let mut status = 0u32;
        let mut len = 4u32;
        if WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            (&mut status as *mut u32).cast(),
            &mut len,
            null_mut(),
        ) == 0
            || status != 200
        {
            return Err(format!("GitHub HTTP {status}").into());
        }
        let mut body = Vec::new();
        loop {
            let mut buffer = [0u8; 8192];
            let mut read = 0;
            if WinHttpReadData(
                request.0,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
                &mut read,
            ) == 0
            {
                return Err(Box::new(native_error()));
            }
            if read == 0 {
                break;
            }
            body.extend_from_slice(&buffer[..read as usize]);
            if body.len() > 1024 * 1024 {
                return Err("GitHub response exceeded 1 MiB".into());
            }
        }
        #[derive(serde::Deserialize)]
        struct Release {
            tag_name: String,
            html_url: String,
        }
        let release: Release = serde_json::from_slice(&body)?;
        if !release
            .html_url
            .starts_with("https://github.com/ezenwa/CodexPet/releases/")
        {
            return Err("Unexpected release URL".into());
        }
        let latest = parse_version(&release.tag_name).ok_or("Invalid release version")?;
        let current = parse_version(VERSION).ok_or("Invalid application version")?;
        if latest > current {
            Ok(UpdateResult::Available(release.tag_name, release.html_url))
        } else {
            Ok(UpdateResult::Current)
        }
    }
}
fn parse_version(version: &str) -> Option<[u64; 3]> {
    let version = version.trim_start_matches(['v', 'V']);
    let mut result = [0; 3];
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    for (i, part) in parts.iter().enumerate() {
        result[i] = part.parse().ok()?;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_version_comparison() {
        assert!(parse_version("v2.10.0") > parse_version("2.9.0"));
        assert!(parse_version("evil").is_none());
    }
}
