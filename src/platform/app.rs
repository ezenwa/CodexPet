use super::{
    graphics::{Surface, draw_label},
    log_error, native_error, open_url,
    processes::single_instance,
    show_error,
    startup::{set_startup, startup_enabled},
    tray::{self, TrayIcon},
    updates::{UpdateResult, check_update},
    wide,
};
use crate::{
    animation,
    config::{self, Language, PETS, Settings, VERSION},
    monitor::{self, Snapshot},
    render::{self, PetImages},
    state::PetState,
};
use std::{
    cell::RefCell,
    error::Error,
    fs, io,
    mem::{size_of, zeroed},
    path::PathBuf,
    ptr::{null, null_mut},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::Instant,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::{
    Foundation::*,
    Globalization::GetUserDefaultUILanguage,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Input::KeyboardAndMouse::ReleaseCapture, WindowsAndMessaging::*},
};
const CLASS: &str = "CodexPet.Rust.Window";
const MUTEX: &str = "Local\\CodexPet.Ezenwa";
const STOP_MESSAGE: u32 = WM_APP + 1;
const SHOW_MESSAGE: u32 = WM_APP + 2;
const PET_MENU: usize = 100;
const LANGUAGE_EN: usize = 201;
const LANGUAGE_ES: usize = 202;
const STARTUP: usize = 203;
const UPDATE: usize = 204;
const ABOUT: usize = 205;
const EXIT: usize = 206;
const MINIMIZE: usize = 207;
const RESTORE: usize = 208;

struct App {
    settings: Settings,
    assets: PathBuf,
    images: PetImages,
    state: PetState,
    clock: Instant,
    receiver: Receiver<Snapshot>,
    update_rx: Option<Receiver<UpdateResult>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    watch: bool,
    visible: bool,
    surface: Option<Surface>,
    smoke: bool,
    tray: Option<TrayIcon>,
    taskbar_created: u32,
}
impl Drop for App {
    fn drop(&mut self) {
        self.tray.take();
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
impl App {
    fn tooltip(&self) -> String {
        format!("CodexPet — {}", self.state.label(self.settings.language))
    }
    unsafe fn minimize_to_tray(&mut self, hwnd: HWND) -> io::Result<()> {
        if self.tray.is_none() {
            self.tray = Some(TrayIcon::new(
                hwnd,
                &self.images.frames[0],
                &self.tooltip(),
            )?);
        }
        self.save_position(hwnd);
        self.visible = false;
        ShowWindow(hwnd, SW_HIDE);
        Ok(())
    }
    unsafe fn restore(&mut self, hwnd: HWND) {
        self.tray.take();
        self.watch = false;
        self.visible = true;
        if !self.smoke {
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        let _ = self.render(hwnd);
    }
    fn render(&mut self, hwnd: HWND) -> io::Result<()> {
        unsafe {
            let mut rect: RECT = zeroed();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return Err(native_error());
            }
            let width = (rect.right - rect.left).clamp(1, 4096);
            let height = (rect.bottom - rect.top).clamp(1, 4096);
            if self
                .surface
                .as_ref()
                .is_none_or(|s| s.width != width || s.height != height)
            {
                self.surface = Some(Surface::new(width, height)?);
            }
            let index = animation::frame(self.state, self.clock.elapsed().as_millis() as u64);
            let mut canvas = render::paint(
                width as u32,
                height as u32,
                &self.images.frames[index],
                self.state,
                self.clock.elapsed().as_secs_f32() * 1.4,
            );
            draw_label(&mut canvas, self.state.label(self.settings.language))?;
            self.surface.as_mut().unwrap().present(hwnd, &canvas)
        }
    }
    fn save_position(&mut self, hwnd: HWND) {
        if self.smoke {
            return;
        }
        unsafe {
            let mut rect: RECT = zeroed();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return;
            }
            let dpi = GetDpiForWindow(hwnd).max(96) as f64 / 96.;
            self.settings.position = config::Position {
                left: rect.left as f64 / dpi,
                top: rect.top as f64 / dpi,
                width: (rect.right - rect.left) as f64 / dpi,
                height: (rect.bottom - rect.top) as f64 / dpi,
            };
            if let Err(error) = self.settings.save_position() {
                log_error(&format!("Saving position: {error}"));
            }
        }
    }
    unsafe fn tick(&mut self, hwnd: HWND) {
        while let Ok(snapshot) = self.receiver.try_recv() {
            if snapshot.state != self.state {
                self.state = snapshot.state;
                self.clock = Instant::now();
                let tooltip = self.tooltip();
                if let Some(tray) = self.tray.as_mut() {
                    tray.set_tooltip(&tooltip);
                }
            }
            if self.watch && self.tray.is_none() && !self.smoke && snapshot.running != self.visible
            {
                ShowWindow(
                    hwnd,
                    if snapshot.running {
                        SW_SHOWNOACTIVATE
                    } else {
                        SW_HIDE
                    },
                );
                self.visible = snapshot.running;
            }
        }
        let update = self.update_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = update {
            self.update_rx = None;
            self.show_update(hwnd, result);
        }
        if (self.visible || self.smoke)
            && let Err(error) = self.render(hwnd)
        {
            log_error(&format!("Rendering: {error}"));
        }
    }
    unsafe fn menu(&mut self, hwnd: HWND) {
        let lang = self.settings.language;
        let menu = CreatePopupMenu();
        let pets = CreatePopupMenu();
        let languages = CreatePopupMenu();
        for (index, (key, name)) in PETS.iter().enumerate() {
            AppendMenuW(
                pets,
                MF_STRING
                    | if *key == self.settings.pet {
                        MF_CHECKED
                    } else {
                        0
                    },
                PET_MENU + index,
                wide(name).as_ptr(),
            );
        }
        AppendMenuW(
            menu,
            MF_POPUP,
            pets as usize,
            wide(lang.text("Choose pet", "Elegir mascota")).as_ptr(),
        );
        AppendMenuW(
            languages,
            MF_STRING
                | if lang == Language::English {
                    MF_CHECKED
                } else {
                    0
                },
            LANGUAGE_EN,
            wide(lang.text("English", "Inglés")).as_ptr(),
        );
        AppendMenuW(
            languages,
            MF_STRING
                | if lang == Language::Spanish {
                    MF_CHECKED
                } else {
                    0
                },
            LANGUAGE_ES,
            wide(lang.text("Spanish", "Español")).as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_POPUP,
            languages as usize,
            wide(lang.text("Language", "Idioma")).as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING | if startup_enabled() { MF_CHECKED } else { 0 },
            STARTUP,
            wide(lang.text("Start with Windows", "Iniciar con Windows")).as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING
                | if self.update_rx.is_some() {
                    MF_GRAYED
                } else {
                    0
                },
            UPDATE,
            wide(&format!(
                "{} (v{VERSION})",
                lang.text("Check for updates…", "Buscar actualizaciones…")
            ))
            .as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING,
            ABOUT,
            wide(lang.text("About", "Acerca de")).as_ptr(),
        );
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(
            menu,
            MF_STRING,
            if self.tray.is_some() {
                RESTORE
            } else {
                MINIMIZE
            },
            wide(if self.tray.is_some() {
                lang.text("Show pet", "Mostrar mascota")
            } else {
                lang.text("Minimize to tray", "Minimizar a la bandeja")
            })
            .as_ptr(),
        );
        AppendMenuW(
            menu,
            MF_STRING,
            EXIT,
            wide(lang.text("Close pet", "Cerrar mascota")).as_ptr(),
        );
        let mut point: POINT = zeroed();
        GetCursorPos(&mut point);
        SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            null(),
        );
        DestroyMenu(menu);
        PostMessageW(hwnd, WM_NULL, 0, 0);
        self.command(hwnd, command as usize);
    }
    unsafe fn command(&mut self, hwnd: HWND, command: usize) {
        match command {
            PET_MENU..=107 => {
                let key = PETS[command - PET_MENU].0;
                match PetImages::load(&self.assets, key) {
                    Ok(images) => {
                        self.images = images;
                        self.settings.pet = key.into();
                        self.clock = Instant::now();
                        if let Some(tray) = self.tray.as_mut()
                            && let Err(error) = tray.set_image(&self.images.frames[0])
                        {
                            log_error(&error.to_string());
                        }
                        if !self.smoke
                            && let Err(e) = self.settings.save_pet()
                        {
                            log_error(&e.to_string());
                        }
                        let mut rect: RECT = zeroed();
                        GetWindowRect(hwnd, &mut rect);
                        let dpi = GetDpiForWindow(hwnd).max(96) as f64 / 96.;
                        let (maxw, maxh) = self.images.maximum_size(dpi);
                        SetWindowPos(
                            hwnd,
                            null_mut(),
                            0,
                            0,
                            (rect.right - rect.left).min(maxw),
                            (rect.bottom - rect.top).min(maxh),
                            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                    Err(e) => show_error(&e.to_string()),
                }
            }
            LANGUAGE_EN | LANGUAGE_ES => {
                self.settings.language = if command == LANGUAGE_ES {
                    Language::Spanish
                } else {
                    Language::English
                };
                if !self.smoke
                    && let Err(e) = self.settings.save_language()
                {
                    log_error(&e.to_string());
                }
                let tooltip = self.tooltip();
                if let Some(tray) = self.tray.as_mut() {
                    tray.set_tooltip(&tooltip);
                }
            }
            STARTUP => {
                if let Err(e) = set_startup(!startup_enabled()) {
                    show_error(&e.to_string());
                }
            }
            UPDATE => {
                if self.update_rx.is_none() {
                    let (tx, rx) = mpsc::channel();
                    self.update_rx = Some(rx);
                    thread::spawn(move || {
                        let result =
                            check_update().unwrap_or_else(|e| UpdateResult::Failed(e.to_string()));
                        let _ = tx.send(result);
                    });
                }
            }
            ABOUT => {
                let text=self.settings.language.text("Author: Joshua Ezenwa\n\nAn independent, unofficial project.\nThe official mascots and visual assets belong to OpenAI/Codex and their respective owners.","Autor: Joshua Ezenwa\n\nProyecto independiente y no oficial.\nLas mascotas y recursos visuales oficiales pertenecen a OpenAI/Codex y a sus respectivos titulares.");
                MessageBoxW(
                    hwnd,
                    wide(&format!("CodexPet v{VERSION}\n\n{text}")).as_ptr(),
                    wide("CodexPet").as_ptr(),
                    MB_OK | MB_ICONINFORMATION,
                );
            }
            EXIT => {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
            MINIMIZE => {
                if let Err(error) = self.minimize_to_tray(hwnd) {
                    if self.smoke {
                        log_error(&error.to_string());
                    } else {
                        show_error(&error.to_string());
                    }
                }
            }
            RESTORE => self.restore(hwnd),
            _ => {}
        }
        let _ = self.render(hwnd);
    }
    unsafe fn show_update(&self, hwnd: HWND, result: UpdateResult) {
        let lang = self.settings.language;
        match result {
            UpdateResult::Current => {
                MessageBoxW(
                    hwnd,
                    wide(&format!(
                        "{} (v{VERSION})",
                        lang.text("CodexPet is up to date.", "CodexPet está actualizado.")
                    ))
                    .as_ptr(),
                    wide("CodexPet").as_ptr(),
                    MB_OK | MB_ICONINFORMATION,
                );
            }
            UpdateResult::Available(version, url) => {
                let answer = MessageBoxW(
                    hwnd,
                    wide(&format!(
                        "{}: {version}\n\n{}",
                        lang.text("New version", "Nueva versión"),
                        lang.text(
                            "Open the official download page?",
                            "¿Abrir la página oficial de descarga?"
                        )
                    ))
                    .as_ptr(),
                    wide("CodexPet").as_ptr(),
                    MB_YESNO | MB_ICONINFORMATION,
                );
                if answer == IDYES {
                    open_url(&url);
                }
            }
            UpdateResult::Failed(error) => {
                MessageBoxW(
                    hwnd,
                    wide(&format!(
                        "{}\n\n{error}",
                        lang.text(
                            "Unable to check for updates.",
                            "No fue posible buscar actualizaciones."
                        )
                    ))
                    .as_ptr(),
                    wide("CodexPet").as_ptr(),
                    MB_OK | MB_ICONWARNING,
                );
            }
        }
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    // Destruction may be delivered synchronously while another callback borrows App.
    if message == WM_DESTROY {
        KillTimer(hwnd, 1);
        KillTimer(hwnd, 2);
        PostQuitMessage(0);
        return 0;
    }
    if message == WM_TIMER && wparam == 2 {
        PostMessageW(hwnd, WM_CLOSE, 0, 0);
        return 0;
    }
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<App>;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    // Win32 can re-enter a window procedure from ShowWindow, menu and move calls.
    // Never create overlapping &mut references to the application in those callbacks.
    let Ok(mut app) = (*pointer).try_borrow_mut() else {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    };
    if message != 0 && message == app.taskbar_created {
        if let Some(tray) = app.tray.as_ref()
            && let Err(error) = tray.recreate()
        {
            log_error(&error.to_string());
            app.restore(hwnd);
        }
        return 0;
    }
    match message {
        tray::CALLBACK => {
            if ((lparam as u32 >> 16) & 0xffff) == tray::ID {
                match lparam as u32 & 0xffff {
                    windows_sys::Win32::UI::Shell::NIN_SELECT
                    | tray::KEY_SELECT
                    | WM_LBUTTONDBLCLK => app.restore(hwnd),
                    WM_CONTEXTMENU => app.menu(hwnd),
                    _ => {}
                }
            }
            0
        }
        WM_TIMER => {
            if wparam == 2 {
                DestroyWindow(hwnd);
            } else {
                app.tick(hwnd);
            }
            0
        }
        WM_MOUSEACTIVATE => MA_NOACTIVATE as isize,
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            BeginPaint(hwnd, &mut ps);
            EndPaint(hwnd, &ps);
            0
        }
        WM_NCHITTEST => {
            let x = (lparam as u16 as i16) as i32;
            let y = ((lparam >> 16) as u16 as i16) as i32;
            let mut rect: RECT = zeroed();
            GetWindowRect(hwnd, &mut rect);
            let layout = render::Layout::new(
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            );
            let band = (3. + 8. * layout.scale) as i32;
            let inset = (8. * layout.scale) as i32;
            let left = (x - (rect.left + inset)).abs() <= band;
            let right = (x - (rect.right - inset)).abs() <= band;
            let top = (y - (rect.top + inset)).abs() <= band;
            let bottom = (y - (rect.bottom - inset)).abs() <= band;
            if top && left {
                HTTOPLEFT as isize
            } else if top && right {
                HTTOPRIGHT as isize
            } else if bottom && left {
                HTBOTTOMLEFT as isize
            } else if bottom && right {
                HTBOTTOMRIGHT as isize
            } else {
                HTCLIENT as isize
            }
        }
        WM_LBUTTONDOWN => {
            drop(app);
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
            if let Ok(mut app) = (*pointer).try_borrow_mut() {
                app.save_position(hwnd);
            }
            0
        }
        WM_NCLBUTTONDOWN => {
            drop(app);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        WM_LBUTTONDBLCLK => {
            let result = ShellExecuteW(
                hwnd,
                wide("open").as_ptr(),
                wide("wt.exe").as_ptr(),
                wide("codex").as_ptr(),
                null(),
                SW_SHOWNORMAL,
            );
            if result as isize <= 32 {
                log_error("Windows Terminal could not be opened.");
            }
            0
        }
        WM_RBUTTONUP | WM_CONTEXTMENU => {
            app.menu(hwnd);
            0
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lparam as *mut MINMAXINFO);
            let dpi = GetDpiForWindow(hwnd).max(96) as f64 / 96.;
            let (maxw, maxh) = app.images.maximum_size(dpi);
            info.ptMinTrackSize = POINT {
                x: (120. * dpi) as i32,
                y: (100. * dpi) as i32,
            };
            info.ptMaxTrackSize = POINT { x: maxw, y: maxh };
            0
        }
        WM_WINDOWPOSCHANGED => {
            let _ = app.render(hwnd);
            drop(app);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        WM_EXITSIZEMOVE => {
            app.save_position(hwnd);
            0
        }
        WM_DPICHANGED => {
            let suggested = &*(lparam as *const RECT);
            let dpi = (wparam & 0xffff) as f64 / 96.;
            let (maxw, maxh) = app.images.maximum_size(dpi);
            SetWindowPos(
                hwnd,
                null_mut(),
                suggested.left,
                suggested.top,
                (suggested.right - suggested.left).min(maxw),
                (suggested.bottom - suggested.top).min(maxh),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            app.save_position(hwnd);
            0
        }
        SHOW_MESSAGE => {
            app.restore(hwnd);
            0
        }
        STOP_MESSAGE | WM_CLOSE => {
            app.tray.take();
            app.save_position(hwnd);
            DestroyWindow(hwnd);
            0
        }
        _ => {
            drop(app);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--stop") {
        unsafe {
            let hwnd = FindWindowW(wide(CLASS).as_ptr(), null());
            if !hwnd.is_null() {
                PostMessageW(hwnd, STOP_MESSAGE, 0, 0);
            }
        }
        return Ok(());
    }
    let assets = config::asset_root()?;
    if let Some(index) = args.iter().position(|a| a == "--preview") {
        let dir = PathBuf::from(
            args.get(index + 1)
                .ok_or("--preview requires an output directory")?,
        );
        fs::create_dir_all(&dir)?;
        let images = PetImages::load(&assets, "codex")?;
        for state in PetState::ALL {
            let frame = animation::frame(state, 0);
            let mut canvas = render::paint(336, 280, &images.frames[frame], state, 0.);
            draw_label(&mut canvas, state.label(Language::Spanish))?;
            canvas.save(dir.join(format!("{state:?}.png")))?;
        }
        return Ok(());
    }
    let smoke = args.iter().any(|a| a == "--smoke-test");
    let mutex_name = if smoke {
        format!("{MUTEX}.Smoke.{}", std::process::id())
    } else {
        MUTEX.into()
    };
    let Some(_mutex) = single_instance(&mutex_name)? else {
        if !args.iter().any(|arg| arg == "--watch") {
            unsafe {
                let hwnd = FindWindowW(wide(CLASS).as_ptr(), null());
                if !hwnd.is_null() {
                    PostMessageW(hwnd, SHOW_MESSAGE, 0, 0);
                }
            }
        }
        return Ok(());
    };
    let state_dir = PathBuf::from(std::env::var_os("APPDATA").ok_or("APPDATA is unavailable")?)
        .join("CodexPet");
    let default_language = unsafe {
        if GetUserDefaultUILanguage() & 0x3ff == 0x0a {
            Language::Spanish
        } else {
            Language::English
        }
    };
    let settings = Settings::load(state_dir, default_language);
    let images = PetImages::load(&assets, &settings.pet)?;
    let watch = args.iter().any(|a| a == "--watch");
    let (tx, rx) = mpsc::channel();
    let simulation = if smoke { Some(tx.clone()) } else { None };
    let stop = Arc::new(AtomicBool::new(false));
    let worker = if smoke {
        None
    } else {
        let home = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("USERPROFILE").unwrap_or_default()).join(".codex")
            });
        Some(monitor::start(home, tx, stop.clone()))
    };
    let app = Box::new(RefCell::new(App {
        settings,
        assets: assets.clone(),
        images,
        state: PetState::Idle,
        clock: Instant::now(),
        receiver: rx,
        update_rx: None,
        stop,
        worker,
        watch,
        visible: !watch && !smoke,
        surface: None,
        smoke,
        tray: None,
        taskbar_created: unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) },
    }));
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(null());
        let class_name = if smoke {
            format!("{CLASS}.Smoke.{}", std::process::id())
        } else {
            CLASS.into()
        };
        let class = wide(&class_name);
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_DBLCLKS,
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            lpszClassName: class.as_ptr(),
            ..zeroed()
        };
        if RegisterClassExW(&wc) == 0 {
            return Err(Box::new(native_error()));
        }
        let dpi = GetDpiForSystem().max(96) as f64 / 96.;
        let p = app.borrow().settings.position;
        let (maxw, maxh) = app.borrow().images.maximum_size(dpi);
        let width = ((p.width * dpi).round() as i32).clamp((120. * dpi) as i32, maxw);
        let height = ((p.height * dpi).round() as i32).clamp((100. * dpi) as i32, maxh);
        let left = (p.left * dpi).round() as i32;
        let top = (p.top * dpi).round() as i32;
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class.as_ptr(),
            wide("CodexPet").as_ptr(),
            WS_POPUP | WS_THICKFRAME,
            left,
            top,
            width,
            height,
            null_mut(),
            null_mut(),
            instance,
            (&*app as *const RefCell<App>).cast(),
        );
        if hwnd.is_null() {
            return Err(Box::new(native_error()));
        }
        let _window_guard = WindowGuard(hwnd);
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) != 0 {
            let x = left.clamp(info.rcWork.left - width + 48, info.rcWork.right - 48);
            let y = top.clamp(info.rcWork.top, info.rcWork.bottom - 48);
            SetWindowPos(
                hwnd,
                null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        app.borrow_mut().render(hwnd)?;
        if smoke {
            for index in 0..PETS.len() {
                app.borrow_mut().command(hwnd, PET_MENU + index);
            }
            app.borrow_mut().command(hwnd, LANGUAGE_EN);
            app.borrow_mut().command(hwnd, LANGUAGE_ES);
            for state in PetState::ALL {
                let mut app = app.borrow_mut();
                app.state = state;
                app.render(hwnd)?;
            }
            SetWindowPos(
                hwnd,
                null_mut(),
                left,
                top,
                240,
                200,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            app.borrow_mut().render(hwnd)?;
        }
        if smoke {
            {
                let mut app = app.borrow_mut();
                app.watch = true;
                app.minimize_to_tray(hwnd)?;
                if app.tray.as_ref().and_then(TrayIcon::bounds).is_none() {
                    return Err("The tray icon was not registered with Windows".into());
                }
                app.tray.as_ref().unwrap().recreate()?;
            }
            for running in [true, false, true] {
                simulation.as_ref().unwrap().send(Snapshot {
                    state: PetState::Working,
                    running,
                })?;
                let mut app = app.borrow_mut();
                app.tick(hwnd);
                if app.visible || app.tray.is_none() || IsWindowVisible(hwnd) != 0 {
                    return Err("Background monitoring restored a minimized pet".into());
                }
            }
            SendMessageW(
                hwnd,
                tray::CALLBACK,
                0,
                ((tray::ID << 16) | tray::KEY_SELECT) as isize,
            );
            if app.borrow().tray.is_some() || !app.borrow().visible {
                return Err("Tray activation did not restore the pet".into());
            }
            app.borrow_mut().minimize_to_tray(hwnd)?;
        }
        let visible = app.borrow().visible;
        if visible {
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        if SetTimer(hwnd, 1, 80, None) == 0 {
            DestroyWindow(hwnd);
            return Err(Box::new(native_error()));
        }
        if smoke {
            SetTimer(hwnd, 2, 400, None);
        }
        let mut msg: MSG = zeroed();
        loop {
            let result = GetMessageW(&mut msg, null_mut(), 0, 0);
            if result == 0 {
                break;
            }
            if result < 0 {
                return Err(Box::new(native_error()));
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if smoke {
            let path = args
                .iter()
                .position(|arg| arg == "--smoke-test")
                .and_then(|index| args.get(index + 1))
                .filter(|arg| !arg.starts_with("--"))
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    std::env::temp_dir().join(format!("codexpet-smoke-{}.ok", std::process::id()))
                });
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(
                path,
                b"Native window, eight pets, languages, six states, resize, tray minimize/restore/recreate and close passed.\n",
            )?;
        }
    }
    Ok(())
}

struct WindowGuard(HWND);
impl Drop for WindowGuard {
    fn drop(&mut self) {
        unsafe {
            if IsWindow(self.0) != 0 {
                DestroyWindow(self.0);
            }
        }
    }
}
