use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const PETS: [(&str, &str); 8] = [
    ("codex", "Codex"),
    ("bsod", "BSOD"),
    ("dewey", "Dewey"),
    ("fireball", "Fireball"),
    ("null-signal", "Null Signal"),
    ("rocky", "Rocky"),
    ("seedy", "Seedy"),
    ("stacky", "Stacky"),
];
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Language {
    #[default]
    English,
    Spanish,
}
impl Language {
    pub fn text(self, en: &'static str, es: &'static str) -> &'static str {
        match self {
            Self::English => en,
            Self::Spanish => es,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Position {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}
impl Default for Position {
    fn default() -> Self {
        Self {
            left: 40.,
            top: 40.,
            width: 168.,
            height: 140.,
        }
    }
}
impl Position {
    pub fn sanitized(self) -> Self {
        let default = Self::default();
        Self {
            left: if self.left.is_finite() {
                self.left
            } else {
                default.left
            },
            top: if self.top.is_finite() {
                self.top
            } else {
                default.top
            },
            width: if self.width.is_finite() {
                self.width.clamp(120., 347.)
            } else {
                default.width
            },
            height: if self.height.is_finite() {
                self.height.clamp(100., 289.)
            } else {
                default.height
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub dir: PathBuf,
    pub position: Position,
    pub pet: String,
    pub language: Language,
}
impl Settings {
    pub fn load(dir: PathBuf, default_language: Language) -> Self {
        let position = fs::read(dir.join("window-position.json"))
            .ok()
            .and_then(|bytes| {
                serde_json::from_slice::<Position>(
                    bytes.strip_prefix(&[239, 187, 191]).unwrap_or(&bytes),
                )
                .ok()
            })
            .unwrap_or_default()
            .sanitized();
        let pet = fs::read_to_string(dir.join("selected-pet.txt"))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|key| PETS.iter().any(|(k, _)| k == key))
            .unwrap_or_else(|| "codex".into());
        let language = match fs::read_to_string(dir.join("language.txt"))
            .unwrap_or_default()
            .trim()
        {
            "es" => Language::Spanish,
            "en" => Language::English,
            _ => default_language,
        };
        Self {
            dir,
            position,
            pet,
            language,
        }
    }
    pub fn save_position(&self) -> io::Result<()> {
        self.write(
            "window-position.json",
            &serde_json::to_vec_pretty(&self.position)?,
        )
    }
    pub fn save_pet(&self) -> io::Result<()> {
        self.write("selected-pet.txt", self.pet.as_bytes())
    }
    pub fn save_language(&self) -> io::Result<()> {
        self.write("language.txt", self.language.text("en", "es").as_bytes())
    }
    fn write(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let target = self.dir.join(name);
        let temp = self.dir.join(format!("{name}.{}.tmp", std::process::id()));
        fs::write(&temp, bytes)?;
        fs::rename(temp, target)
    }
}

pub fn asset_root() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    // An installed or portable executable always uses its adjacent assets first.
    let mut candidates = vec![exe.parent().unwrap_or(Path::new(".")).to_path_buf()];
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    for root in candidates {
        if root.join("assets/pets").is_dir() {
            return Ok(root);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Missing assets/pets beside CodexPet.exe",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_legacy_pascal_case_position() {
        let position: Position =
            serde_json::from_str(r#"{"Left":21,"Top":32,"Width":180,"Height":150}"#).unwrap();
        assert_eq!(position.left, 21.);
        assert_eq!(position.height, 150.);
    }
    #[test]
    fn clamps_corrupt_dimensions() {
        let p = Position {
            width: f64::NAN,
            height: 99999.,
            ..Position::default()
        }
        .sanitized();
        assert_eq!(p.width, 168.);
        assert_eq!(p.height, 289.);
    }
}
