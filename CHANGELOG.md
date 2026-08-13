# Changelog

[Historial en español](CHANGELOG.es.md)

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
