use super::native_error;
use super::wide;
use crate::render;
use image::RgbaImage;
use std::{
    io,
    mem::{size_of, zeroed},
    ptr::null_mut,
};
use windows_sys::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};
pub(super) struct Surface {
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
    bits: *mut u8,
    pub(super) width: i32,
    pub(super) height: i32,
}
impl Surface {
    pub(super) unsafe fn new(width: i32, height: i32) -> io::Result<Self> {
        let dc = CreateCompatibleDC(null_mut());
        if dc.is_null() {
            return Err(native_error());
        }
        let mut info: BITMAPINFO = zeroed();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..zeroed()
        };
        let mut bits = null_mut();
        let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if bitmap.is_null() {
            DeleteDC(dc);
            return Err(native_error());
        }
        let old = SelectObject(dc, bitmap);
        Ok(Self {
            dc,
            bitmap,
            old,
            bits: bits.cast(),
            width,
            height,
        })
    }
    pub(super) unsafe fn present(&mut self, hwnd: HWND, image: &RgbaImage) -> io::Result<()> {
        let out =
            std::slice::from_raw_parts_mut(self.bits, (self.width * self.height * 4) as usize);
        for (src, dst) in image
            .as_raw()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(out.as_chunks_mut::<4>().0.iter_mut())
        {
            let a = src[3] as u16;
            dst[0] = ((src[2] as u16 * a + 127) / 255) as u8;
            dst[1] = ((src[1] as u16 * a + 127) / 255) as u8;
            dst[2] = ((src[0] as u16 * a + 127) / 255) as u8;
            dst[3] = src[3];
        }
        let mut rect: RECT = zeroed();
        GetWindowRect(hwnd, &mut rect);
        let point = POINT {
            x: rect.left,
            y: rect.top,
        };
        let size = SIZE {
            cx: self.width,
            cy: self.height,
        };
        let source = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        if UpdateLayeredWindow(
            hwnd,
            null_mut(),
            &point,
            &size,
            self.dc,
            &source,
            0,
            &blend,
            ULW_ALPHA,
        ) == 0
        {
            return Err(native_error());
        }
        Ok(())
    }
}
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(self.bitmap);
            DeleteDC(self.dc);
        }
    }
}

pub(super) fn draw_label(canvas: &mut RgbaImage, text: &str) -> io::Result<()> {
    unsafe {
        let mask = Surface::new(canvas.width() as i32, canvas.height() as i32)?;
        std::ptr::write_bytes(mask.bits, 0, (mask.width * mask.height * 4) as usize);
        let layout = render::Layout::new(canvas.width(), canvas.height());
        let font = CreateFontW(
            -(14. * layout.scale).round() as i32,
            0,
            0,
            0,
            FW_BOLD as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            ANTIALIASED_QUALITY as u32,
            DEFAULT_PITCH as u32,
            wide("Segoe UI").as_ptr(),
        );
        if font.is_null() {
            return Err(native_error());
        }
        let previous = SelectObject(mask.dc, font);
        SetBkMode(mask.dc, TRANSPARENT as i32);
        SetTextColor(mask.dc, 0xffffff);
        let (left, top, right, bottom) = layout.text_rect();
        let mut rect = RECT {
            left,
            top,
            right,
            bottom,
        };
        let text = wide(text);
        DrawTextW(
            mask.dc,
            text.as_ptr(),
            (text.len() - 1) as i32,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
        );
        // DIB memory must not be read until GDI's batched drawing has completed.
        GdiFlush();
        let bytes = std::slice::from_raw_parts(mask.bits, (mask.width * mask.height * 4) as usize);
        for (dst, src) in canvas.pixels_mut().zip(bytes.as_chunks::<4>().0.iter()) {
            let alpha = src[0].max(src[1]).max(src[2]) as f32 / 255.;
            if alpha > 0. {
                for (i, c) in [192., 202., 245.].iter().enumerate() {
                    dst[i] = (dst[i] as f32 * (1. - alpha) + c * alpha).round() as u8;
                }
                dst[3] = dst[3].max((alpha * 255.) as u8);
            }
        }
        SelectObject(mask.dc, previous);
        DeleteObject(font);
        Ok(())
    }
}

/// Class icons are shared resource handles and must not be destroyed by the caller.
pub(super) fn application_icon(instance: HINSTANCE, small: bool) -> io::Result<HICON> {
    unsafe {
        let dpi = windows_sys::Win32::UI::HiDpi::GetDpiForSystem().max(96);
        let (x, y) = if small {
            (SM_CXSMICON, SM_CYSMICON)
        } else {
            (SM_CXICON, SM_CYICON)
        };
        let width = windows_sys::Win32::UI::HiDpi::GetSystemMetricsForDpi(x, dpi);
        let height = windows_sys::Win32::UI::HiDpi::GetSystemMetricsForDpi(y, dpi);
        let icon = LoadImageW(
            instance,
            101usize as *const u16,
            IMAGE_ICON,
            width,
            height,
            LR_SHARED,
        ) as HICON;
        if icon.is_null() {
            Err(native_error())
        } else {
            Ok(icon)
        }
    }
}

/// Returns an owned icon; the notification-area owner releases it with DestroyIcon.
pub(super) fn icon_from_image(image: &RgbaImage) -> io::Result<HICON> {
    unsafe {
        let size = windows_sys::Win32::UI::HiDpi::GetSystemMetricsForDpi(
            SM_CXSMICON,
            windows_sys::Win32::UI::HiDpi::GetDpiForSystem().max(96),
        )
        .max(32);
        let mut canvas = RgbaImage::new(size as u32, size as u32);
        let thumb = image::imageops::thumbnail(image, size as u32, size as u32);
        image::imageops::overlay(
            &mut canvas,
            &thumb,
            (size as i64 - thumb.width() as i64) / 2,
            (size as i64 - thumb.height() as i64) / 2,
        );
        let surface = Surface::new(size, size)?;
        let pixels = std::slice::from_raw_parts_mut(surface.bits, (size * size * 4) as usize);
        for (src, dst) in canvas
            .as_raw()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(pixels.as_chunks_mut::<4>().0.iter_mut())
        {
            let alpha = src[3] as u16;
            dst[0] = ((src[2] as u16 * alpha + 127) / 255) as u8;
            dst[1] = ((src[1] as u16 * alpha + 127) / 255) as u8;
            dst[2] = ((src[0] as u16 * alpha + 127) / 255) as u8;
            dst[3] = src[3];
        }
        let mask_bits = vec![0u8; (((size + 15) / 16) * 2 * size) as usize];
        let mask = CreateBitmap(size, size, 1, 1, mask_bits.as_ptr().cast());
        if mask.is_null() {
            return Err(native_error());
        }
        // CreateIconIndirect requires its bitmaps to be deselected from all DCs.
        SelectObject(surface.dc, surface.old);
        let info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: surface.bitmap,
        };
        let icon = CreateIconIndirect(&info);
        let result = if icon.is_null() {
            Err(native_error())
        } else {
            Ok(icon)
        };
        DeleteObject(mask);
        SelectObject(surface.dc, surface.bitmap);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_label_rendering_preserves_alpha() {
        let mut canvas = RgbaImage::from_pixel(168, 140, image::Rgba([26, 27, 38, 240]));
        draw_label(&mut canvas, "CODEX WORKING").unwrap();
        assert!(canvas.pixels().any(|pixel| pixel[0] > 100));
    }
}
