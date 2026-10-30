use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    println!("cargo:rerun-if-changed=assets/pets");
    let mut embedded = String::from("pub static PET_ASSETS: &[(&str, &[u8])] = &[\n");
    let mut pets: Vec<_> = fs::read_dir(root.join("assets/pets"))
        .expect("Pet assets")
        .map(|entry| entry.expect("Pet directory").path())
        .collect();
    pets.sort();
    for pet in pets {
        assert!(pet.is_dir(), "Unexpected pet asset entry");
        let key = pet
            .file_name()
            .expect("Pet key")
            .to_str()
            .expect("UTF-8 pet key");
        for frame in 0..72 {
            let name = format!("frame_{frame:03}.png");
            let path = pet.join(&name);
            assert!(path.is_file(), "Missing pet frame: {}", path.display());
            println!("cargo:rerun-if-changed={}", path.display());
            embedded.push_str(&format!(
                "({:?}, include_bytes!({:?})),\n",
                format!("pets/{key}/{name}"),
                path.to_str().expect("UTF-8 asset path")
            ));
        }
    }
    embedded.push_str("];\n");
    fs::write(output.join("pet_assets.rs"), embedded).expect("Write embedded pet manifest");
    println!("cargo:rerun-if-changed=assets/icons/codexpet.ico");
    println!("cargo:rerun-if-changed=Cargo.toml");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = root.join("assets/icons/codexpet.ico");
    let icon = icon.to_str().expect("UTF-8 icon path").replace('\\', "/");
    let version = env::var("CARGO_PKG_VERSION").expect("Cargo version");
    let numeric_version = format!("{},0", version.replace('.', ","));
    let resource = format!(
        r#"
101 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {numeric_version}
PRODUCTVERSION {numeric_version}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "ezenwa\0"
      VALUE "FileDescription", "CodexPet - Native desktop pet for Codex\0"
      VALUE "FileVersion", "{version}\0"
      VALUE "InternalName", "CodexPet\0"
      VALUE "OriginalFilename", "CodexPet.exe\0"
      VALUE "ProductName", "CodexPet\0"
      VALUE "ProductVersion", "{version}\0"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );
    let path =
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory")).join("codexpet.rc");
    fs::write(&path, resource).expect("Write Windows resource definition");
    // Shipping the generic Windows icon is a build failure, not a silent fallback.
    embed_resource::compile(&path, embed_resource::NONE)
        .manifest_required()
        .expect("Compile and embed CodexPet icon/version resources");
}
