# Architecture

The crate exposes platform-independent application logic through `src/lib.rs`; `src/main.rs` starts the native Windows application. Windows integration is selected with `#[cfg(windows)]`.

| Module | Responsibility |
| --- | --- |
| `animation.rs` | Maps elapsed time and state to frames in an 8×9 sheet. |
| `config.rs` | Pet catalog, language, version and legacy-compatible APPDATA settings. |
| `state.rs` | Typed events, pending call identities, incremental byte reading and session priority. |
| `monitor.rs` | Background discovery, process filtering and incremental polling. |
| `approval.rs` | Bounded, read-only SQLite access for approval requests and decisions. |
| `render.rs` | Runtime PNG loading and transparent composition. |
| `platform/app.rs` | Window, event loop, menus and application lifecycle. |
| `platform/graphics.rs` | GDI surfaces, text masks and bitmap-to-icon conversion. |
| `platform/processes.rs` | Single-instance mutex and Codex process enumeration. |
| `platform/startup.rs` | The current user's startup registration. |
| `platform/updates.rs` | WinHTTP updates and version comparison. |
| `platform/tray.rs` | Notification icon ownership, tooltips and Explorer recreation. |

## Threads and visibility

The main thread owns the window and native handles. A worker discovers sessions every five seconds and polls incremental changes about once per second, sending snapshots over a channel. Update checks use a separate worker so network requests do not block animation.

In watch mode, Codex activity controls visibility. Explicitly minimizing creates the notification icon before hiding the window. Its presence prevents incoming activity from showing the pet again. Clicking the icon, choosing Show pet or opening a second manual instance restores the window. Shutdown deletes the icon and stops the worker. The icon can be recreated after Explorer restarts.

## State and integration limits

Attention takes priority over working; otherwise the latest terminal event determines the state. Completion expires after ten minutes. Pending responses and approvals are correlated with call IDs. Old unfinished files predating the running Codex processes are excluded where process creation times are available.

Codex's JSONL and SQLite logs are internal integration points. Malformed and unknown events are ignored. If approval metadata is unavailable or changes schema, an escalated call can remain in attention state until its output arrives. An unrelated tool result cannot resolve a pending question.

## Data and packaging

Settings are stored in `%APPDATA%\CodexPet`, preserving the earlier position JSON and pet/language text formats. Runtime errors use a rotating local log there. The app reads `$CODEX_HOME`, defaulting to `%USERPROFILE%\.codex`. Only a manual update check contacts GitHub.

Source assets use `assets/pets/<pet>/frame_000.png` through `frame_071.png`. The executable uses adjacent assets in portable or installed packages. Development builds can find the source assets. Packaging uses a static MSVC CRT and retains the installer identity for upgrades. The installer helper remains necessary for stopping PowerShell-era installations; normal operation does not use it.
