# Changelog

[Historial en español](CHANGELOG.es.md)

## 2.2.0 - 2026-10-30

- Replaced Inno Setup with native Rust installation, migration and self-removal.
- Embedded all pet frames so the portable application is a standalone executable.
- Replaced PowerShell build scripts with Cargo xtask checks, packaging and isolated installation tests.
- Preserved per-user preferences, startup choices, application icon and existing uninstall registration during upgrades.
- Removed obsolete installer definitions, helpers and installed resource copies.

## 2.1.1 - 2026-10-30

- Added a transparent mascot application icon with ten sizes from 16 to 256 pixels.
- Embedded the icon and Cargo-derived version metadata in the Windows executable.
- Applied the icon to native windows, desktop/Start menu shortcuts, installer and uninstall entry.
- Added package checks for embedded icon and executable version resources.

## 2.1.0 - 2026-10-30

- Added **Minimize to tray**, with a pet icon, localized status, restore and close actions.
- Kept manually minimized pets hidden while Codex activity changes, and restored the tray icon after Explorer restarts.
- Split native Windows integration into application, graphics, process, startup, update and tray modules.
- Grouped reusable scripts under `tools/` and installer helpers under `installer/`; updated all packaging paths.
- Flattened each pet's 72 PNG frames and removed unused Sixel exports, obsolete launchers, one-off installation scripts, backups and temporary build files.
- Added development and architecture documentation, file-placement guidance and consistent text settings.
- Verification removes its temporary files, and the version is defined only in Cargo.toml.

## 2.0.0 - 2026-10-29

- Replaced the PowerShell/WPF application with a native Rust/Win32 executable.
- Integrated session monitoring into the application; closing it stops the observer.
- Preserved all eight pets, animation timings and existing user preferences.
- Correlated pending questions and approval decisions by tool call ID and aggregated active sessions.
- Moved log reading and update checks off the UI thread.
- Added portable packaging, native window smoke checks and Rust regression tests.
- Used Cargo.toml as the source of the application and installer version.

## 1.2.3 - 2026-08-12

- Added a localized **About** menu entry.
- The About dialog identifies Joshua Ezenwa as the author and attributes the official mascots and visual assets to OpenAI/Codex and their respective owners.

## 1.2.2 - 2026-08-12

- Real-time detection of pending approvals through local Codex SQLite telemetry.
- Previously remembered approvals no longer trigger a false **Needs attention** state.
- **Task complete** timing is calculated from the actual event, even when Codex preserves an old modification time on the active session file.

## 1.2.1 - 2026-08-12

- **Task complete** remains visible until another task begins or ten minutes of inactivity pass.
- Added an automated regression test to prevent the completed state from returning to idle prematurely.

## 1.2.0 - 2026-08-12

- Fixed corrupted **Check for updates** text under Windows PowerShell 5.1.
- Normalized PowerShell scripts to UTF-8 with BOM to preserve Spanish text correctly.
- Added an English and Spanish interface with a persistent language selector.
- Migrated the bilingual installer from IExpress to Inno Setup 6.
- Fixed incremental detection for large sessions and real start, attention, completion, and error events.

## 1.1.0 - 2026-08-12

- Added **Check for updates** to the context menu.
- Added secure GitHub Releases checks without automatic installation.
- Expanded documentation and added a preview of the different states.
- Added a reproducible release process using the CodexPet project name.

## 1.0.0 - 2026-08-12

- First public release.
- Animated states based on Codex session events.
- Selector with eight pets.
- Persistent position, size, and selected pet.
- Stable watcher for primary and helper Codex processes.
