use crate::{config::PETS, state::PetState};
use image::{
    Rgba, RgbaImage,
    imageops::{FilterType, resize},
};
use std::io;

pub struct PetImages {
    pub frames: Vec<RgbaImage>,
}
impl PetImages {
    pub fn load(key: &str) -> io::Result<Self> {
        if !PETS.iter().any(|(k, _)| *k == key) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown pet"));
        }
        let files: Vec<_> = crate::assets::pet_frames(key).collect();
        if files.len() != 72 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected 72 PNG frames for {key}, found {}", files.len()),
            ));
        }
        let frames = files
            .into_iter()
            .map(|bytes| {
                image::load_from_memory(bytes)
                    .map(|i| i.into_rgba8())
                    .map_err(io::Error::other)
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(Self { frames })
    }
    pub fn maximum_size(&self, dpi: f64) -> (i32, i32) {
        let first = &self.frames[0];
        let scale = (first.width() as f64 / 116.).min(first.height() as f64 / 125.);
        // Do not upscale the mascot above its original physical pixel dimensions.
        (
            ((210. * scale).floor() as i32).max((120. * dpi) as i32),
            ((175. * scale).floor() as i32).max((100. * dpi) as i32),
        )
    }
}

pub struct Layout {
    pub scale: f32,
    pub x: f32,
    pub y: f32,
}
impl Layout {
    pub fn new(width: u32, height: u32) -> Self {
        let scale = (width as f32 / 210.).min(height as f32 / 175.);
        Self {
            scale,
            x: (width as f32 - 210. * scale) * 0.5,
            y: (height as f32 - 175. * scale) * 0.5,
        }
    }
    pub fn text_rect(&self) -> (i32, i32, i32, i32) {
        (
            (self.x + 14. * self.scale) as i32,
            (self.y + 133. * self.scale) as i32,
            (self.x + 196. * self.scale) as i32,
            (self.y + 163. * self.scale) as i32,
        )
    }
}

pub fn paint(
    width: u32,
    height: u32,
    sprite: &RgbaImage,
    state: PetState,
    phase: f32,
) -> RgbaImage {
    let mut canvas = RgbaImage::new(width, height);
    let l = Layout::new(width, height);
    let color = state.color();
    let left = l.x + 8. * l.scale;
    let top = l.y + 8. * l.scale;
    let right = l.x + 202. * l.scale;
    let bottom = l.y + 167. * l.scale;
    let radius = 24. * l.scale;
    for y in 0..height {
        for x in 0..width {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let distance = rounded_distance(fx, fy, left, top, right, bottom, radius);
            if distance > 0. {
                let alpha = (30. * (-distance / (5. * l.scale).max(1.)).exp()).round() as u8;
                canvas.put_pixel(x, y, Rgba([color[0], color[1], color[2], alpha]));
            } else {
                let inside = (-distance / (2. * l.scale).max(1.)).clamp(0., 1.);
                let base = if fy > l.y + 125. * l.scale {
                    [36, 40, 59]
                } else {
                    [26, 27, 38]
                };
                let mut rgba = [0, 0, 0, 240];
                for i in 0..3 {
                    rgba[i] = (color[i] as f32 * (1. - inside) + base[i] as f32 * inside) as u8;
                }
                canvas.put_pixel(x, y, Rgba(rgba));
            }
        }
    }
    let sprite_height = (125. * l.scale).round().max(1.) as u32;
    let sprite_width = (sprite.width() as f32 / sprite.height() as f32 * sprite_height as f32)
        .round()
        .max(1.) as u32;
    let sprite = resize(sprite, sprite_width, sprite_height, FilterType::Triangle);
    let sx = (l.x + (210. * l.scale - sprite_width as f32) * 0.5).round() as i32;
    let sy = (l.y - 2. * l.scale).round() as i32;
    for (x, y, pixel) in sprite.enumerate_pixels() {
        let dx = sx + x as i32;
        let dy = sy + y as i32;
        if dx >= 0 && dy >= 0 && (dx as u32) < width && (dy as u32) < height {
            blend(canvas.get_pixel_mut(dx as u32, dy as u32), *pixel);
        }
    }
    let pulse_x = l.x + 186. * l.scale;
    let pulse_y = l.y + 32. * l.scale;
    let alpha = ((0.45 + (phase.sin() + 1.) * 0.275) * 255.) as u8;
    for y in 0..height {
        for x in 0..width {
            let d =
                ((x as f32 + 0.5 - pulse_x).powi(2) + (y as f32 + 0.5 - pulse_y).powi(2)).sqrt();
            if d < 6.5 * l.scale {
                let c = if d > 5.5 * l.scale {
                    [242, 244, 255]
                } else {
                    color
                };
                blend(canvas.get_pixel_mut(x, y), Rgba([c[0], c[1], c[2], alpha]));
            }
        }
    }
    canvas
}
fn rounded_distance(x: f32, y: f32, l: f32, t: f32, r: f32, b: f32, radius: f32) -> f32 {
    let qx = (x - (l + r) * 0.5).abs() - (r - l) * 0.5 + radius;
    let qy = (y - (t + b) * 0.5).abs() - (b - t) * 0.5 + radius;
    qx.max(0.).hypot(qy.max(0.)) + qx.max(qy).min(0.) - radius
}
fn blend(dst: &mut Rgba<u8>, src: Rgba<u8>) {
    let a = src[3] as f32 / 255.;
    let da = dst[3] as f32 / 255.;
    let out = a + da * (1. - a);
    if out <= 0. {
        return;
    }
    for i in 0..3 {
        dst[i] = ((src[i] as f32 * a + dst[i] as f32 * da * (1. - a)) / out).round() as u8;
    }
    dst[3] = (out * 255.).round() as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_assets_load_and_render_without_losing_transparency() {
        for (key, _) in PETS {
            let pet = PetImages::load(key).unwrap();
            assert_eq!(pet.frames.len(), 72);
            let output = paint(168, 140, &pet.frames[0], PetState::Working, 0.);
            assert!(output.get_pixel(0, 0)[3] < 30);
            assert!(output.get_pixel(84, 110)[3] > 200);
        }
    }
}
