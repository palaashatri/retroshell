# SLOPOS-I Desktop Environment Capability Matrix (`CAPABILITY_MATRIX.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Capability Audit  
**Legend:**  
- ✅ **Implemented & Tested**: Fully working in SLOPOS-I with automated test coverage.
- 🟡 **Implemented but Incomplete**: Functional core exists, edge cases or UI controls missing.
- 🔗 **Delegated to External Component**: Handled via standard Linux D-Bus / system service.
- 🎨 **Stub / Visual Mock Only**: UI exists but backend is un-wired.
- ❌ **Missing**: Not currently implemented.

---

## 1. Session Management

| Capability | Status | Implementation Details / Responsible Crate |
| :--- | :---: | :--- |
| Standalone Login Session | 🟡 | `scripts/start-slopos-i` session script; `slopos-greeter` PAM greeter stub. |
| Session Startup & Teardown | ✅ | `start-slopos-i` launches compositor, shell, portals, and session clients cleanly. |
| Logout | ✅ | `slopos-shell` System menu "Log Out" action terminates session process tree. |
| Shutdown | 🔗 | System menu "Shut Down" delegates to systemd `org.freedesktop.login1Manager.PowerOff`. |
| Reboot | 🔗 | System menu "Restart" delegates to systemd `org.freedesktop.login1Manager.Reboot`. |
| Suspend | 🔗 | Delegates to systemd `org.freedesktop.login1Manager.Suspend`. |
| Lock Screen | ✅ | `slopos-lock` standalone screen locker with PAM/env password validation. |
| Crash Recovery | 🟡 | `session_clients` reaps died app processes; shell auto-restarts managed clients. |

---

## 2. Window Management

| Capability | Status | Implementation Details / Responsible Crate |
| :--- | :---: | :--- |
| Window Launch | ✅ | LaunchServices spawns native apps & external binaries (`xdg_toplevel`). |
| Window Close | ✅ | `ShellWindow` close box & `wlr_foreign_toplevel_handle_v1.close`. |
| Window Move | ✅ | Titlebar dragging updates position in `slopos-compositor` & `slopos-shell`. |
| Window Resize | ✅ | Bottom-right resize grip dragging resizes client bounds in real time. |
| Window Minimize | ✅ | Minimize box collapses window to bottom dock. |
| Window Maximize / Zoom | ✅ | Zoom box toggles window between zoomed bounds and restored rect. |
| Fullscreen | ✅ | Fullscreen menu action toggles window to 100% monitor resolution. |
| Modal Windows | 🟡 | `ShellWindow` supports modal overlay flags; parent-child attachment wired in SDK. |
| Transient Windows | 🟡 | Supported via Wayland `xdg_toplevel.set_parent`. |
| Keyboard Focus | ✅ | Pointer click & window activation update focus in `workspace_focus.rs`. |
| Alt-Tab Switcher | ✅ | `Super+Tab` / `Alt+Tab` cycles window focus list in `slopos-shell`. |
| Stacking / Z-Order | ✅ | 10-tier z-index layer stack in `COMPOSITOR_ARCHITECTURE.md`. |
| Always-on-Top | 🟡 | Layer-shell `Layer::Overlay` supports pinned overlay windows. |
| Multiple Windows per App | ✅ | Managed via `session_clients` PID registry & foreign-toplevel list. |
| Workspaces | ✅ | 8 workspace grid cells with shortcut switching (`Super+1`..`8`). |
| Multi-Monitor Placement | 🟡 | `slopos-compositor` supports multi-output DRM layouts (`SLOPOS_OUTPUTS_LAYOUT`). |
| Per-Monitor Scaling | 🟡 | Display scaling config supported in compositor output management. |

---

## 3. Input Handling

| Capability | Status | Implementation Details / Responsible Crate |
| :--- | :---: | :--- |
| Keyboard Input | ✅ | `xkbcommon` keyboard mapping in `slopos-compositor` & `slopos-render`. |
| Mouse Pointer | ✅ | `libinput` pointer movement, click, and cursor rendering. |
| Touchpad | 🟡 | Basic pointer tap-to-click supported via `libinput`. |
| Scrolling | ✅ | Mouse wheel & touchpad 2-finger scroll events dispatched to widgets. |
| Text Input | ✅ | `slopos-sdk` `TextField` & `Terminal` keyboard input dispatching. |
| Input Methods (IBus/Fcitx)| ❌ | `text-input-v3` Wayland protocol not fully implemented. |
| Global Shortcuts | ✅ | `Super+Space` (Spotlight), `Super+Tab` (Workspaces), `Cmd+Q` (Close). |

---

## 4. Desktop Integration

| Capability | Status | Implementation Details / Responsible Crate |
| :--- | :---: | :--- |
| Clipboard Copy & Paste | ✅ | In-process clipboard in `slopos-kit` & `wl-clipboard` Wayland data device. |
| Primary Selection | 🟡 | Supported in `Terminal` selection buffer. |
| Drag and Drop | 🟡 | File drag-and-drop support in `IconView` and `Finder`. |
| File Dialogs | 🟡 | `slopos-portal` file chooser portal stub & `Dialog` widget in SDK. |
| Open-With & MIME Associations| ✅ | `LaunchServices` MIME database mapping in `slopos-shell`. |
| Default Applications | ✅ | Configured in `settings.conf` (`default_browser`, `default_texteditor`). |
| Trash | ✅ | `~/.local/share/Trash` integration in `Finder` & Desktop Trash Icon. |
| Notifications | ✅ | Notification Center in top menu bar with action buttons & timeouts. |
| App Badges | 🟡 | Dock item unread badge rendering in `DockView`. |
| XDG Desktop Files | ✅ | `.desktop` file parsing in `LaunchServices` (`/usr/share/applications`). |
| Secrets / Keyring | 🔗 | Delegates to `org.freedesktop.secrets` / `gnome-keyring`. |

---

## 5. Hardware & System Services

| Capability | Status | Implementation Details / Responsible Crate |
| :--- | :---: | :--- |
| Audio Output / Input | 🔗 | Top menu bar Volume control calls PipeWire / PulseAudio via D-Bus. |
| Network Status & Wi-Fi | 🔗 | Network menu status indicator calls NetworkManager D-Bus API. |
| Removable Storage Automount | 🔗 | Volume mounting delegates to `UDisks2` (`org.freedesktop.UDisks2`). |
| Power & Battery Status | 🔗 | Battery indicator in top menu bar calls `UPower` D-Bus API. |
| Date, Time & Timezone | ✅ | Live clock in top menu bar using `chrono` / `SystemTime`. |
| Screenshots | ✅ | Integrated screenshot capture using `sway` + `grim` / `screenshot.rs`. |
| Screen Recording | 🟡 | PipeWire screencast portal interface in `slopos-portal`. |

---

## 6. Security & Accessibility

| Capability | Status | Implementation Details / Responsible Crate |
| :--- | :---: | :--- |
| Polkit Authentication | 🔗 | Delegates privileged actions to `polkit-gnome-authentication-agent-1`. |
| High Contrast Theme | ✅ | `ThemeName::HighContrast` pure black/white theme in `theme_manager.rs`. |
| Reduced Motion | ✅ | Accessibility preference toggle in `a11y_prefs.rs`. |
| Keyboard Navigation | ✅ | Tab key focus traversal across widgets in `FocusManager`. |
