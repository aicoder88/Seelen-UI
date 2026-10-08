# Local monitor feature work

Owner: Codex. Started 2026-09-14 from upstream v2.8.4, commit 7870b82.
Scope authorized by Drago: install and configure Seelen, separate pinned apps per
monitor, remember the last-used monitor when reopening apps. Manual launch only:
no Windows logon task or recurring job. Preserve Seelen's existing UI.

Claimed files: service startup/task registration; dock state/types/context menus;
window-monitor memory integration; focused tests; local installation documentation.
No publication, upstream PR, or remote fork is authorized.

The upstream `.npmrc` and signing material are excluded from the working tree.
No existing Seelen installation or user configuration was detected.

## Behavior

- Right-click an app and use **Pin to this monitor** or **Unpin from this monitor**.
  File drops and the dock's file picker also pin only on the receiving monitor.
- Pins survive closing apps and restarting Seelen. Existing Windows taskbar pins
  remain shared initially; each monitor can independently override them.
- Running-window indicators, previews, and keyboard launch indexes respect the
  configured monitor visibility. Item order remains shared.
- App monitor memory uses a Windows AppUserModelId, falling back to the executable
  path. Moving or focusing an app updates its last-used monitor. Reopened windows
  return there after an 800 ms settling interval.
- Memory is per application, not per document or browser tab. It remembers the
  monitor, not an exact saved window rectangle. New windows retain their normal
  size and offset, adjusted for the target display's DPI and available work area.
- Windows already open when Seelen starts are not moved. Fullscreen, minimized,
  actively dragged windows, explicit monitor rules, and automatic tiling take
  precedence. Missing displays retain their saved assignment; there is no forced
  migration of existing windows when a display reconnects.

## Manual installation

The local branch enables `manual-launch` by default. It blocks task creation,
logon enablement, the upstream service installer, and stock automatic updates.
It creates no Windows service registration or scheduled job. The helper process
runs only with the manually launched application and stops when Seelen exits.
Normal launch uses the current user's privileges. Elevated applications may reject
window moves; controlling those requires explicitly launching the helper as administrator.

The install script copies the binaries and bundled resources into
`%LOCALAPPDATA%\Programs\SeelenUI-Manual` and adds an ordinary Start Menu shortcut
named **Seelen UI (manual launch)**. It is not placed in the Startup folder.

The initial configuration used 32 px icons, bottom placement, content width, and
automatic hiding. The top toolbar, wallpaper manager, automatic tiling, Alt-Tab
replacement, workspace-switcher replacement, Start-menu replacement, cloud
backup, and Discord presence are disabled. Telemetry and stock updates are
disabled in this local build. Existing settings are backed up before configuration.
During verification, the user changed the dock to 55 px icons and always visible,
and enabled the top toolbar. Those choices were preserved. Reinstallation leaves
existing settings untouched unless `scripts/install-manual.ps1 -Configure` is used.
The latest binary update preserved the settings file byte for byte.

Local data:

- Pins: `%APPDATA%\com.seelen.seelen-ui\data\seelen-weg\state.yml`
- App monitor memory: `%APPDATA%\com.seelen.seelen-ui\last-monitors.json`
- Settings: `%APPDATA%\com.seelen.seelen-ui\settings.json`

This is an unsigned local debug build, based on official v2.8.4 sources. Local
bundled-file checksums remain verified. Seelen's upstream signature cannot apply
to modified local binaries/resources. Upgrades require rebuilding this branch
to preserve manual-launch behavior and these features.

## Verification

Completed before installation:

- Core library: 76 upstream tests passed.
- Monitor-pin serialization: 2 additional Rust tests passed.
- Pin isolation and persistence: 3 TypeScript tests passed.
- Window relocation: 3 Rust geometry tests passed (DPI, smaller work area,
  and a display left of the primary monitor).
- Full UI build and TypeScript/Svelte/Deno checks passed.
- Native `cargo check` and full debug binary builds passed, including the final
  maximized-window restoration fix.
- Live normal-window restoration passed: the disposable app started on the primary
  monitor and moved to its remembered secondary monitor after approximately 800 ms.
- Live maximized-window restoration passed: the app moved to the secondary monitor
  and remained maximized. The first implementation failed this check and was fixed.
- The user confirmed that right-clicking the dock shows **Pin to this monitor**.
  Desktop automation could not click the transparent dock reliably, so persistence
  and isolation were verified through the focused tests, not a full GUI round trip.
- No Seelen scheduled tasks existed after installation, launch, or restart.
  Attempting to enable startup was rejected by the manual-launch guard.
- Exiting Seelen stopped its helper and restored the native Windows taskbars and
  the original 1032 px work areas on both displays.
- The disposable test application was closed and its monitor-memory entry removed.
- `git diff --check` passed. Changes remain local and uncommitted; nothing was published.

An initial cold launch logged upstream lock timeouts and widget reload warnings.
A clean restart resolved the observed lock timeouts. Some flyout and WMI warnings
remain in upstream logging; these were not broadened into unrelated changes.

Two active 1920×1080 displays were detected. A disposable verification app always
starts on the primary display, allowing restoration to be distinguished from an
application's own position memory. It is not installed as a user application.
