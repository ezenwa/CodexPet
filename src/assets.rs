//! Build-time embedded pet frames; portable and installed apps need no sidecar assets.
include!(concat!(env!("OUT_DIR"), "/pet_assets.rs"));

pub fn pet_frames(key: &str) -> impl Iterator<Item = &'static [u8]> {
    let prefix = format!("pets/{key}/");
    PET_ASSETS
        .iter()
        .filter(move |(name, _)| name.starts_with(&prefix))
        .map(|(_, bytes)| *bytes)
}
