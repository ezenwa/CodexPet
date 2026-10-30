# Development

CodexPet is a Cargo application for Windows x64. Its UI uses Win32 directly through `windows-sys`; there is no web frontend or WPF runtime.

## Tools

Use the current stable Rust MSVC toolchain with rustfmt and Clippy, plus Visual Studio C++ Build Tools and a Windows SDK. For a project-local toolchain, run:

```powershell
powershell -NoProfile -File tools/Setup-Rust.ps1
```

The local installation stays in `.build/cargo` and `.build/rustup`. `tools/Cargo.ps1` selects it when available and otherwise uses Cargo from PATH. It does not change the global PATH.

## Build and check

With a system Rust installation:

```powershell
cargo fmt -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

For local tools, replace `cargo` with `powershell -NoProfile -File tools/Cargo.ps1`, preserving the remaining arguments. Download dependencies with `tools/Cargo.ps1 fetch` when necessary. The executable is `target/release/CodexPet.exe`.

Unit tests live beside their modules and include state correlation, multi-session priority, incremental reading, animation bounds, legacy settings and all runtime pet assets. Add Cargo's conventional `tests/` directory only for real integration tests; do not keep empty placeholder folders.

On a Windows desktop with Explorer running, verify the actual window and notification-area integration:

```powershell
powershell -NoProfile -File tools/Verify-Native.ps1
```

This exercises hidden rendering, all eight pets, languages, resizing, tray registration, activation, recreation and shutdown. A test tray icon may appear briefly. User preferences are not changed. Temporary previews and markers are deleted when verification finishes.

## Package

`build.rs` compiles the Windows icon and version resources through the build-only `embed-resource` dependency. It reads `assets/icons/codexpet.ico` and derives executable metadata from Cargo's version. A missing icon fails the Windows build. The ICO contains ten RGBA sizes from 16 to 256 pixels. Keep the icon source here; it is embedded, so a separate copy is not shipped in runtime assets.

The source of the version is `Cargo.toml`. Update it and both changelogs for a release. `Cargo.lock` should remain checked in. `VERSION` is generated inside the portable payload, not maintained as a second source file.

```powershell
powershell -NoProfile -File tools/Build-Release.ps1
# Without Inno Setup:
powershell -NoProfile -File tools/Build-Release.ps1 -PortableOnly
# Verify installer metadata and the extracted portable package:
powershell -NoProfile -File tools/Verify-Package.ps1
```

Inno Setup 6 is required for the installer. `dist/CodexPet/` is the runnable portable folder, `dist/CodexPet-Portable.zip` is its archive and `dist/CodexPet-Setup.exe` is the installer. Rebuilding refreshes the portable folder to avoid shipping removed resources. Package verification extracts into a temporary path containing spaces and removes it afterward.

Publishing the repository or creating a GitHub release is a separate operation. Local packaging does not push commits or upload artifacts.

## Repository boundaries

- `src/`: application and native platform code; see [architecture](ARCHITECTURE.md).
- `assets/pets/<pet>/`: only the 72 PNG frames consumed by the application. Preserve names, ordering and attribution.
- `installer/`: Inno Setup, language defaults and the helper needed to stop existing installations before upgrading.
- `tools/`: reusable build and validation scripts. One-off workstation scripts do not belong here.
- `docs/`: technical documentation; user installation and usage stay in the root READMEs.
- `.build/`, `target/`, `dist/`: ignored local tools and generated outputs. Do not move installed dependencies during source reorganization.

Use rustfmt for Rust. EditorConfig and Git attributes define text encoding and line endings. Keep secrets and personal configuration outside the repository. The project does not keep source backups or obsolete release trees.
