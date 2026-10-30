//! Notification-area icon ownership and Explorer restart recovery.
use super::{graphics::icon_from_image, wide};
use image::RgbaImage;
use std::{
    io,
    mem::{size_of, zeroed},
};
use windows_sys::Win32::{
    Foundation::*,
    UI::{Shell::*, WindowsAndMessaging::*},
};

pub(super) const CALLBACK: u32 = WM_APP + 3;
pub(super) const ID: u32 = 1;
// NIN_KEYSELECT is the Win32 macro NIN_SELECT | NINF_KEY, omitted by the bindings.
pub(super) const KEY_SELECT: u32 = NIN_SELECT | 1;

struct Icon(HICON);
impl Drop for Icon {
    fn drop(&mut self) {
        unsafe {
            DestroyIcon(self.0);
        }
    }
}

pub(super) struct TrayIcon {
    data: NOTIFYICONDATAW,
    icon: Icon,
}
impl TrayIcon {
    pub fn new(hwnd: HWND, image: &RgbaImage, tooltip: &str) -> io::Result<Self> {
        let icon = Icon(icon_from_image(image)?);
        let mut data: NOTIFYICONDATAW = unsafe { zeroed() };
        data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = ID;
        data.uCallbackMessage = CALLBACK;
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        data.hIcon = icon.0;
        set_tooltip(&mut data, tooltip);
        let result = Self { data, icon };
        result.recreate()?;
        Ok(result)
    }
    pub fn recreate(&self) -> io::Result<()> {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.data);
            if Shell_NotifyIconW(NIM_ADD, &self.data) == 0 {
                return Err(io::Error::other(
                    "Windows could not create the notification-area icon.",
                ));
            }
            let mut version = self.data;
            version.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            if Shell_NotifyIconW(NIM_SETVERSION, &version) == 0 {
                Shell_NotifyIconW(NIM_DELETE, &self.data);
                return Err(io::Error::other(
                    "Windows could not configure the notification-area icon.",
                ));
            }
            Ok(())
        }
    }
    pub fn set_tooltip(&mut self, tooltip: &str) {
        set_tooltip(&mut self.data, tooltip);
        unsafe {
            Shell_NotifyIconW(NIM_MODIFY, &self.data);
        }
    }
    pub fn set_image(&mut self, image: &RgbaImage) -> io::Result<()> {
        let icon = Icon(icon_from_image(image)?);
        let mut data = self.data;
        data.hIcon = icon.0;
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &data) } == 0 {
            return Err(io::Error::other(
                "Windows could not update the notification-area icon.",
            ));
        }
        self.data = data;
        self.icon = icon;
        Ok(())
    }
    pub fn bounds(&self) -> Option<RECT> {
        unsafe {
            let identifier = NOTIFYICONIDENTIFIER {
                cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
                hWnd: self.data.hWnd,
                uID: ID,
                ..zeroed()
            };
            let mut rect: RECT = zeroed();
            if Shell_NotifyIconGetRect(&identifier, &mut rect) >= 0 {
                Some(rect)
            } else {
                None
            }
        }
    }
}
impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.data);
        }
    }
}
fn set_tooltip(data: &mut NOTIFYICONDATAW, text: &str) {
    data.szTip.fill(0);
    // Leave a terminating NUL, including when the localized status is long.
    for (slot, ch) in data.szTip.iter_mut().take(127).zip(wide(text)) {
        *slot = ch;
    }
}
