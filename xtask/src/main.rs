//! Build, package and exercise the Windows application using Cargo only.
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    error::Error,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::env::set_current_dir(root)?;
    match std::env::args().nth(1).as_deref() {
        Some("check") => check(),
        Some("release") => {
            check()?;
            cargo(&[
                "build",
                "--release",
                "--locked",
                "--package",
                "codexpet",
                "--bin",
                "CodexPet",
            ])?;
            package(root)?;
            verify(root)
        }
        Some("verify") => verify(root),
        _ => Err("Usage: cargo xtask <check|release|verify>".into()),
    }
}
fn command(path: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut c = Command::new(path);
    #[cfg(windows)]
    c.creation_flags(0x08000000);
    c.stdin(Stdio::null());
    c
}
fn cargo(args: &[&str]) -> Result<()> {
    let exe = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = command(exe).args(args).output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    if !output.status.success() {
        return Err(format!("cargo {} failed", args.join(" ")).into());
    }
    Ok(())
}
fn check() -> Result<()> {
    cargo(&["fmt", "--all", "--", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo(&["test", "--workspace", "--locked"])
}
fn package(root: &Path) -> Result<()> {
    let dist = root.join("dist");
    if dist.exists() {
        let resolved = dist.canonicalize()?;
        if resolved.parent() != Some(root.canonicalize()?.as_path())
            || resolved.file_name() != Some(std::ffi::OsStr::new("dist"))
        {
            return Err("Unexpected distribution directory".into());
        }
        fs::remove_dir_all(&resolved)?;
    }
    fs::create_dir(&dist)?;
    let executable = root.join("target/release/CodexPet.exe");
    for name in ["CodexPet.exe", "CodexPet-Setup.exe"] {
        fs::copy(&executable, dist.join(name))?;
    }
    let mut zip = ZipWriter::new(fs::File::create(dist.join("CodexPet-Portable.zip"))?);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (path, name) in [
        (executable, "CodexPet.exe"),
        (root.join("README.md"), "README.md"),
        (root.join("README.es.md"), "README.es.md"),
    ] {
        zip.start_file(format!("CodexPet/{name}"), options)?;
        zip.write_all(&fs::read(path)?)?;
    }
    zip.finish()?;
    let mut sums = String::new();
    for name in [
        "CodexPet.exe",
        "CodexPet-Setup.exe",
        "CodexPet-Portable.zip",
    ] {
        sums.push_str(&format!("{}  {name}\n", hash(&dist.join(name))?));
    }
    fs::write(dist.join("SHA256SUMS"), sums)?;
    println!("Packaged CodexPet {} with Rust.", codexpet::config::VERSION);
    Ok(())
}
fn hash(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn child(exe: &Path, args: &[&str]) -> Result<()> {
    let mut process = command(exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let start = Instant::now();
    loop {
        if let Some(status) = process.try_wait()? {
            let output = process.wait_with_output()?;
            std::io::stdout().write_all(&output.stdout)?;
            std::io::stderr().write_all(&output.stderr)?;
            if !status.success() {
                return Err(format!("{} {:?} exited with {status}", exe.display(), args).into());
            }
            return Ok(());
        }
        if start.elapsed() > Duration::from_secs(90) {
            process.kill()?;
            process.wait()?;
            return Err("Native check exceeded 90 seconds".into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}
fn verify(root: &Path) -> Result<()> {
    let dist = root.join("dist");
    for line in fs::read_to_string(dist.join("SHA256SUMS"))?.lines() {
        let (expected, name) = line.split_once("  ").ok_or("Invalid checksum entry")?;
        if hash(&dist.join(name))? != expected {
            return Err(format!("Checksum failed: {name}").into());
        }
    }
    if hash(&dist.join("CodexPet.exe"))? != hash(&dist.join("CodexPet-Setup.exe"))? {
        return Err("Setup must use the same Rust executable".into());
    }
    let temp = tempfile::Builder::new()
        .prefix("CodexPet-portable-check-")
        .tempdir()?;
    let mut zip = ZipArchive::new(fs::File::open(dist.join("CodexPet-Portable.zip"))?)?;
    if zip.len() != 3 {
        return Err("Unexpected portable archive contents".into());
    }
    zip.extract(temp.path().join("path with spaces"))?;
    let exe = temp.path().join("path with spaces/CodexPet/CodexPet.exe");
    if hash(&exe)? != hash(&dist.join("CodexPet.exe"))? {
        return Err("Portable executable differs".into());
    }
    check_pe(&exe)?;
    let marker = temp.path().join("native.ok");
    child(
        &exe,
        &["--smoke-test", marker.to_str().ok_or("Non-UTF8 marker")?],
    )?;
    if !fs::read_to_string(marker)?.contains("passed") {
        return Err("Native smoke marker missing".into());
    }
    verify_install(&exe)?;
    println!(
        "Verified checksums, standalone portable, embedded icon, native UI and isolated install/upgrade/uninstall."
    );
    Ok(())
}
fn check_pe(path: &Path) -> Result<()> {
    let bytes = fs::read(path)?;
    let u16at = |p: usize| -> Result<u16> {
        Ok(u16::from_le_bytes(
            bytes.get(p..p + 2).ok_or("Truncated PE")?.try_into()?,
        ))
    };
    let pe = u32::from_le_bytes(
        bytes
            .get(0x3c..0x40)
            .ok_or("Missing PE offset")?
            .try_into()?,
    ) as usize;
    if bytes.get(pe..pe + 4) != Some(b"PE\0\0")
        || u16at(pe + 4)? != 0x8664
        || u16at(pe + 24 + 68)? != 2
    {
        return Err("Expected a Windows x64 GUI executable".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "shell32")]
        unsafe extern "system" {
            fn ExtractIconExW(
                file: *const u16,
                index: i32,
                large: *mut isize,
                small: *mut isize,
                count: u32,
            ) -> u32;
        }
        let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        if unsafe {
            ExtractIconExW(
                name.as_ptr(),
                -1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        } == 0
        {
            return Err("Executable icon resource missing".into());
        }
    }
    Ok(())
}
#[cfg(windows)]
fn verify_install(exe: &Path) -> Result<()> {
    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, KEY_WRITE},
    };
    let temp = tempfile::Builder::new()
        .prefix("CodexPet-install-check-")
        .rand_bytes(12)
        .tempdir()?;
    let root = temp.path().to_str().ok_or("Non-UTF8 test path")?;
    let namespace = format!(
        r"Software\CodexPet\InstallTests\{}",
        temp.path().file_name().unwrap().to_string_lossy()
    );
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let result = (|| -> Result<()> {
        let setup = temp.path().join("CodexPet-Setup.exe");
        fs::copy(exe, &setup)?;
        child(&setup, &["--quiet", "--no-launch", "--test-root", root])?;
        let installed = temp.path().join("app/CodexPet.exe");
        if hash(&installed)? != hash(exe)? {
            return Err("Installed executable differs".into());
        }
        for link in ["desktop/CodexPet.lnk", "startmenu/CodexPet.lnk"] {
            if !temp.path().join(link).is_file() {
                return Err("Shortcut missing".into());
            }
        }
        let key = hkcu.open_subkey(format!(r"{namespace}\uninstall"))?;
        if key.get_value::<String, _>("DisplayVersion")? != codexpet::config::VERSION
            || !key
                .get_value::<String, _>("UninstallString")?
                .contains("--uninstall")
        {
            return Err("Incorrect uninstall registration".into());
        }
        drop(key);
        let run = hkcu.open_subkey_with_flags(format!(r"{namespace}\run"), KEY_WRITE)?;
        run.delete_value("CodexPetWatcher")?;
        drop(run);
        fs::create_dir_all(temp.path().join("app/assets/pets"))?;
        fs::write(temp.path().join("app/assets/pets/old.png"), b"legacy")?;
        fs::write(temp.path().join("app/unins000.dat"), b"legacy")?;
        let (old, _) = hkcu.create_subkey(format!(r"{namespace}\uninstall"))?;
        old.set_value("Inno Setup: Setup Version", &"retired")?;
        drop(old);
        child(
            exe,
            &["--install", "--quiet", "--no-launch", "--test-root", root],
        )?;
        if hkcu
            .open_subkey(format!(r"{namespace}\run"))?
            .get_value::<String, _>("CodexPetWatcher")
            .is_ok()
        {
            return Err("Upgrade re-enabled disabled startup".into());
        }
        if temp.path().join("app/assets").exists() || temp.path().join("app/unins000.dat").exists()
        {
            return Err("Legacy files survived upgrade".into());
        }
        if hkcu
            .open_subkey(format!(r"{namespace}\uninstall"))?
            .get_value::<String, _>("Inno Setup: Setup Version")
            .is_ok()
        {
            return Err("Legacy installer metadata survived upgrade".into());
        }
        let marker = temp.path().join("installed.ok");
        child(&installed, &["--smoke-test", marker.to_str().unwrap()])?;
        if !marker.exists() {
            return Err("Installed application failed native check".into());
        }
        child(&installed, &["--uninstall", "--quiet", "--test-root", root])?;
        let start = Instant::now();
        while installed.exists() && start.elapsed() < Duration::from_secs(10) {
            thread::sleep(Duration::from_millis(100));
        }
        if temp.path().join("app").exists()
            || temp.path().join("desktop/CodexPet.lnk").exists()
            || temp.path().join("startmenu/CodexPet.lnk").exists()
            || hkcu.open_subkey(format!(r"{namespace}\uninstall")).is_ok()
        {
            return Err("Uninstall left registered application files".into());
        }
        // Uninstall invoked from a portable executable must delegate to the installed
        // copy, and it must preserve files that the application does not own.
        child(
            exe,
            &["--install", "--quiet", "--no-launch", "--test-root", root],
        )?;
        let unknown = temp.path().join("app/user-note.txt");
        fs::write(&unknown, b"preserve")?;
        child(exe, &["--uninstall", "--quiet", "--test-root", root])?;
        if installed.exists() || fs::read(unknown)? != b"preserve" || !exe.exists() {
            return Err("Delegated uninstall removed unowned files or retained the app".into());
        }
        Ok(())
    })();
    let cleanup = hkcu.delete_subkey_all(namespace);
    result?;
    cleanup?;
    Ok(())
}
#[cfg(not(windows))]
fn verify_install(_: &Path) -> Result<()> {
    Err("Native package verification requires Windows".into())
}
