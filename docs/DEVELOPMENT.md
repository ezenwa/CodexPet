# Development

CodexPet is a Cargo workspace for Windows x64. The application, installation and packaging tools are written in Rust. The UI uses native Win32 APIs; there is no web frontend or external installer technology.

## Prerequisites

Use stable Rust MSVC with rustfmt and Clippy, Visual Studio C++ Build Tools and a Windows SDK. The linker and SDK resource compiler are normal Windows native build requirements. End users only need Windows 10 version 1703 or later, or Windows 11, x64.

If Rust is installed locally under `.build/`, select `.build/cargo` as `CARGO_HOME`, `.build/rustup` as `RUSTUP_HOME` and add `.build/cargo/bin` to the current process PATH. These directories are ignored and contain the development toolchain, not application resources.

## Build and check

```text
cargo xtask check
cargo build --release --locked
cargo xtask release
cargo xtask verify
```

`check` runs rustfmt, Clippy with warnings denied and all workspace tests. Unit tests live beside their modules and cover state correlation, session priority, incremental reads, animation bounds, existing settings and embedded pet assets. Create `tests/` only when there are integration tests to place there.

`release` runs those checks, builds the release application, refreshes `dist/`, packages it and verifies the result. `verify` checks the current distribution without rebuilding. Verification requires an interactive Windows desktop with Explorer running. It renders a hidden window, exercises every pet and state, language changes, resizing, tray activation and recreation. A test tray icon can appear briefly. It also installs, upgrades and uninstalls into an isolated temporary directory and registry namespace, checking startup choice preservation and legacy resource removal. Temporary files and test registration are cleaned afterward; actual user settings and installation are not touched.

## Packaging and installation

`Cargo.toml` is the version source; update it and both changelogs for a release. Commit `Cargo.lock`. `build.rs` embeds the 576 PNG frames and compiles the Windows icon and version resources through the build-only `embed-resource` crate. The icon source contains ten RGBA sizes from 16 to 256 pixels. The application uses a static MSVC CRT.

Generated outputs:

- `dist/CodexPet.exe`: standalone portable application.
- `dist/CodexPet-Setup.exe`: the same binary; its filename selects installation mode.
- `dist/CodexPet-Portable.zip`: the standalone application with bilingual usage documentation.
- `dist/SHA256SUMS`: checksums of the three downloadable artifacts.

`CodexPet.exe --install` installs or upgrades under `%LOCALAPPDATA%\CodexPet`, creates desktop and Start menu shortcuts, and registers removal with Windows Settings. `--no-launch` suppresses launching after installation. The default installation starts in `--watch` mode. Existing startup choices and `%APPDATA%\CodexPet` preferences survive upgrades. `--quiet` suppresses error dialogs for unattended operations; errors still produce a nonzero exit status.

`CodexPet.exe --uninstall` removes its own registration, shortcuts and executable while preserving preferences and unknown files. The retained uninstall registry identifier provides continuity with previous installations; the old installer executable is never run. Migration removes the documented old installer files and its `assets/` and `docs/` resource trees. Reparse points are rejected before traversing those trees. `self-replace` handles removal of the running executable using temporary native cleanup helpers. No PowerShell scripts or third-party installer are needed.

The internal `--test-root` option is limited to direct children of the OS temporary directory with the `CodexPet-install-check-` prefix. It changes all install, shortcut and registry destinations and skips stopping the user's running pet. Use it only through `cargo xtask verify`.

Publishing commits and creating a GitHub release are separate operations; packaging never uploads files or changes the remote repository.

## File placement

- `src/`: application logic; `src/platform/` contains Windows integration and installation.
- `xtask/src/`: Rust build, package and verification code.
- `assets/pets/<pet>/`: 72 PNG frames per pet, embedded at compilation. Preserve names, order and attribution.
- `assets/icons/`: the source application icon.
- `docs/`: technical documentation and `docs/images/` screenshots; user usage stays in the root READMEs.
- `.build/`, `target/`, `dist/`: ignored local tools and generated outputs.

Use rustfmt, EditorConfig and Git attributes for formatting and text encoding. Keep secrets and workstation configuration outside source control. The project retains no source backups or obsolete release trees.
