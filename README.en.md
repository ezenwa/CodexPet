# CodexPet

[Documentación en español](README.md)

An animated Codex desktop pet for Windows. CodexPet detects Codex activity, displays its current state, and stays visible without interrupting your work.

> [!IMPORTANT]
> The official mascots and visual assets belong to OpenAI/Codex and their respective owners. CodexPet claims no authorship or ownership over them. This is an independent, unofficial project that provides an alternative way to display them on Windows.

![CodexPet states](docs/images/codexpet-states.png)

## Features

- Visual states tied to real Codex events: idle, working, attention required, task completed, blocked/error, and offline.
- Eight selectable pets: Codex, BSOD, Dewey, Fireball, Null Signal, Rocky, Seedy, and Stacky.
- English and Spanish interface, switchable from the context menu.
- Stable single window even when Codex creates temporary helper processes.
- Persistent position, size, language, and selected pet.
- Per-user Windows startup.
- Manual update checks through GitHub Releases.
- Per-user installation without administrator privileges.

## Requirements

- Windows 10 or Windows 11.
- Windows PowerShell 5.1 or PowerShell 7.
- A working Codex installation that runs `codex.exe`.
- Internet access only for update checks and downloads.

## Installation

1. Open the [latest release](https://github.com/ezenwa/CodexPet/releases/latest).
2. Download `CodexPet-Setup.exe`.
3. Run the installer and choose English or Spanish.

CodexPet is installed in `%LOCALAPPDATA%\CodexPet`. The Inno Setup installer creates desktop and Start menu shortcuts, registers the watcher at Windows startup, and provides a native uninstaller.

## Usage

- Drag the card with the left mouse button to move it.
- Resize it from the corners.
- Double-click it to open Windows Terminal with Codex.
- Right-click it to select a pet, change language, manage Windows startup, check for updates, or close CodexPet.

## Updates

Select **Check for updates** from the context menu. CodexPet only checks the latest public release from `ezenwa/CodexPet`. It never downloads or runs an update automatically.

## Privacy

CodexPet processes event types locally from `%USERPROFILE%\.codex\sessions`. It does not send prompts, responses, or file contents. Update checks only make a standard HTTPS request to GitHub.

## Development

CodexPet is built with PowerShell and WPF. Release installers require [Inno Setup 6](https://jrsoftware.org/isinfo.php).

```powershell
.\Build-Release.ps1 -Version 1.2.0
```

See [README.md](README.md) for Spanish documentation and [CHANGELOG.md](CHANGELOG.md) for release history.
