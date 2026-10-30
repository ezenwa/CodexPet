//! Per-user installation and removal implemented by the application itself.
use super::wide;
use crate::config::VERSION;
use std::{
    error::Error,
    fs, io,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use windows::{
    Win32::{
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize, IPersistFile,
        },
        UI::Shell::{
            FOLDERID_Desktop, FOLDERID_LocalAppData, FOLDERID_Programs, IShellLinkW,
            KF_FLAG_DEFAULT, SHGetKnownFolderPath, ShellLink,
        },
    },
    core::{Interface, PCWSTR},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, WAIT_OBJECT_0},
    System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    UI::WindowsAndMessaging::{FindWindowW, GetWindowThreadProcessId, PostMessageW, WM_APP},
};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{A913EB65-C10B-4C1F-8845-627C4921488B}_is1";
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const NO_WINDOW: u32 = 0x08000000;

struct Layout {
    app: PathBuf,
    desktop: PathBuf,
    menu: PathBuf,
    uninstall: String,
    run: String,
    test: bool,
}
impl Layout {
    fn from_args(args: &[String]) -> Result<Self> {
        if let Some(i) = args.iter().position(|a| a == "--test-root") {
            let input = args.get(i + 1).ok_or("Missing test root")?;
            let root = fs::canonicalize(input)?;
            let temp = fs::canonicalize(std::env::temp_dir())?;
            let name = root
                .file_name()
                .ok_or("Missing test name")?
                .to_string_lossy();
            if root.parent() != Some(temp.as_path())
                || !name.starts_with("CodexPet-install-check-")
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err("Invalid isolated installation root".into());
            }
            // Shell links require ordinary DOS/UNC paths, not canonical \\?\ paths.
            let root = std::path::absolute(input)?;
            return Ok(Self {
                app: root.join("app"),
                desktop: root.join("desktop/CodexPet.lnk"),
                menu: root.join("startmenu/CodexPet.lnk"),
                uninstall: format!(r"Software\CodexPet\InstallTests\{name}\uninstall"),
                run: format!(r"Software\CodexPet\InstallTests\{name}\run"),
                test: true,
            });
        }
        Ok(Self {
            app: known_folder(&FOLDERID_LocalAppData)?.join("CodexPet"),
            desktop: known_folder(&FOLDERID_Desktop)?.join("CodexPet.lnk"),
            menu: known_folder(&FOLDERID_Programs)?.join("CodexPet/CodexPet.lnk"),
            uninstall: UNINSTALL.into(),
            run: RUN.into(),
            test: false,
        })
    }
    fn exe(&self) -> PathBuf {
        self.app.join("CodexPet.exe")
    }
}

fn known_folder(id: &windows::core::GUID) -> Result<PathBuf> {
    unsafe {
        let value = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None)?;
        let path = PathBuf::from(value.to_string()?);
        CoTaskMemFree(Some(value.0.cast()));
        Ok(path)
    }
}

pub(super) fn dispatch(args: &[String]) -> Result<bool> {
    let setup = std::env::current_exe()?
        .file_stem()
        .is_some_and(|n| n.eq_ignore_ascii_case("CodexPet-Setup"));
    let remove = args.iter().any(|a| a == "--uninstall");
    if !remove && !setup && !args.iter().any(|a| a == "--install") {
        return Ok(false);
    }
    let layout = Layout::from_args(args)?;
    if remove {
        uninstall(&layout, args)?;
    } else {
        install(&layout, args)?;
    }
    Ok(true)
}

fn stop_app() -> Result<()> {
    unsafe {
        let hwnd = FindWindowW(wide("CodexPet.Rust.Window").as_ptr(), std::ptr::null());
        if hwnd.is_null() {
            return Ok(());
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return Err(io::Error::last_os_error().into());
        }
        PostMessageW(hwnd, WM_APP + 1, 0, 0);
        let result = WaitForSingleObject(handle, 30_000);
        CloseHandle(handle);
        if result != WAIT_OBJECT_0 {
            return Err("The running pet did not stop within 30 seconds".into());
        }
    }
    Ok(())
}

fn shortcut(path: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(path.parent().ok_or("Shortcut has no parent")?)?;
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let result = (|| -> Result<()> {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
            let target_w = wide(&target.to_string_lossy());
            link.SetPath(PCWSTR(target_w.as_ptr()))?;
            link.SetIconLocation(PCWSTR(target_w.as_ptr()), 0)?;
            link.SetWorkingDirectory(PCWSTR(
                wide(
                    &target
                        .parent()
                        .ok_or("Executable has no parent")?
                        .to_string_lossy(),
                )
                .as_ptr(),
            ))?;
            let persist: IPersistFile = link.cast()?;
            persist.Save(PCWSTR(wide(&path.to_string_lossy()).as_ptr()), true)?;
            Ok(())
        })();
        CoUninitialize();
        result
    }
}

fn install(layout: &Layout, args: &[String]) -> Result<()> {
    if !layout.test {
        stop_app()?;
    }
    fs::create_dir_all(&layout.app)?;
    reject_reparse(&layout.app)?;
    let current = fs::canonicalize(std::env::current_exe()?)?;
    let target = layout.exe();
    if target.exists() {
        reject_reparse(&target)?;
    }
    if fs::canonicalize(&target).ok().as_ref() != Some(&current) {
        let staged = layout
            .app
            .join(format!(".codexpet-install-{}.tmp", std::process::id()));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        let result = (|| -> io::Result<()> {
            io::copy(&mut fs::File::open(&current)?, &mut file)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&staged, &target)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staged);
        }
        result?;
    }
    shortcut(&layout.desktop, &target)?;
    shortcut(&layout.menu, &target)?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey(&layout.run)?;
    let existing = hkcu.open_subkey(&layout.uninstall).is_ok();
    let start = run.get_value::<String, _>("CodexPetWatcher").is_ok() || !existing;
    if start {
        run.set_value(
            "CodexPetWatcher",
            &format!("\"{}\" --watch", target.display()),
        )?;
    }
    let (key, _) = hkcu.create_subkey(&layout.uninstall)?;
    let legacy_values: Vec<String> = key
        .enum_values()
        .filter_map(|entry| entry.ok().map(|(name, _)| name))
        .filter(|name| name.starts_with("Inno Setup:"))
        .collect();
    for name in legacy_values {
        key.delete_value(name)?;
    }
    for (name, value) in [
        ("DisplayName", "CodexPet".to_string()),
        ("DisplayVersion", VERSION.into()),
        ("Publisher", "ezenwa".into()),
        ("InstallLocation", layout.app.display().to_string()),
        ("DisplayIcon", format!("\"{}\",0", target.display())),
        (
            "InstallDate",
            chrono::Local::now().format("%Y%m%d").to_string(),
        ),
        (
            "UninstallString",
            format!("\"{}\" --uninstall", target.display()),
        ),
        (
            "QuietUninstallString",
            format!("\"{}\" --uninstall --quiet", target.display()),
        ),
        ("URLInfoAbout", "https://github.com/ezenwa/CodexPet".into()),
    ] {
        key.set_value(name, &value)?;
    }
    key.set_value("NoModify", &1u32)?;
    key.set_value("NoRepair", &1u32)?;
    key.set_value(
        "EstimatedSize",
        &((fs::metadata(&target)?.len() / 1024) as u32),
    )?;
    retire_legacy(&layout.app)?;
    if !layout.test {
        // Remove the two known legacy uninstall shortcuts; preserve other files.
        for name in ["Uninstall CodexPet.lnk", "Desinstalar CodexPet.lnk"] {
            remove_file(&layout.menu.parent().unwrap().join(name))?;
        }
    }
    if !layout.test && !args.iter().any(|a| a == "--no-launch") {
        spawn(&target, &["--watch".into()])?;
    }
    Ok(())
}

fn uninstall(layout: &Layout, args: &[String]) -> Result<()> {
    let target = layout.exe();
    if layout.app.exists() {
        reject_reparse(&layout.app)?;
    }
    if target.exists() {
        reject_reparse(&target)?;
    }
    let current = fs::canonicalize(std::env::current_exe()?)?;
    if fs::canonicalize(&target).ok().as_ref() != Some(&current) {
        if !target.exists() {
            return Err("CodexPet is not installed for this user".into());
        }
        let mut forwarded = vec!["--uninstall".into()];
        if args.iter().any(|a| a == "--quiet") {
            forwarded.push("--quiet".into());
        }
        if let Some(i) = args.iter().position(|a| a == "--test-root") {
            forwarded.extend(["--test-root".into(), args[i + 1].clone()]);
        }
        let status = command(&target).args(forwarded).status()?;
        if !status.success() {
            return Err("Installed uninstaller failed".into());
        }
        return Ok(());
    }
    reject_reparse(&layout.app)?;
    if !layout.test {
        stop_app()?;
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) = hkcu.open_subkey_with_flags(&layout.run, winreg::enums::KEY_WRITE) {
        match run.delete_value("CodexPetWatcher") {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    match hkcu.delete_subkey_all(&layout.uninstall) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    remove_file(&layout.desktop)?;
    remove_file(&layout.menu)?;
    if !layout.test {
        for name in ["Uninstall CodexPet.lnk", "Desinstalar CodexPet.lnk"] {
            remove_file(&layout.menu.parent().unwrap().join(name))?;
        }
    }
    let _ = fs::remove_dir(layout.menu.parent().unwrap());
    retire_legacy(&layout.app)?;
    self_replace::self_delete_outside_path(&layout.app)?;
    // Only remove an empty application directory; settings and unknown files survive.
    match fs::remove_dir(&layout.app) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::DirectoryNotEmpty => {}
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn command(path: &Path) -> Command {
    let mut c = Command::new(path);
    c.creation_flags(NO_WINDOW).stdin(Stdio::null());
    c
}
fn spawn(path: &Path, args: &[String]) -> Result<()> {
    command(path).args(args).spawn()?;
    Ok(())
}
fn remove_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
fn reject_reparse(path: &Path) -> Result<()> {
    use std::os::windows::fs::MetadataExt;
    if fs::symlink_metadata(path)?.file_attributes() & 0x400 != 0 {
        return Err(format!("Refusing to modify reparse point: {}", path.display()).into());
    }
    Ok(())
}

fn retire_legacy(app: &Path) -> Result<()> {
    for name in [
        "unins000.exe",
        "unins000.dat",
        "unins000.msg",
        "README.md",
        "README.es.md",
        "README.en.md",
        "CHANGELOG.md",
        "CHANGELOG.es.md",
        "VERSION",
        "CodexPet.ps1",
        "CodexPet-State.ps1",
        "CodexPet-Watcher.ps1",
        "Start-CodexPetWatcher.vbs",
        "Start-CodexPet.cmd",
        "Stop-CodexPet.ps1",
        "Uninstall-CodexPet.ps1",
        "codexpet.pid",
        "close-with-codex.request",
        "watcher.log",
    ] {
        remove_file(&app.join(name))?;
    }
    // Only the former installer's documented resource trees belong to the app.
    for name in ["assets", "docs"] {
        let dir = app.join(name);
        if dir.exists() {
            remove_owned_tree(&dir)?;
        }
    }
    Ok(())
}
fn remove_owned_tree(path: &Path) -> Result<()> {
    reject_reparse(path)?;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            remove_owned_tree(&entry?.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}
