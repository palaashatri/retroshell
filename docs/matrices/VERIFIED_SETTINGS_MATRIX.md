# SLOPOS-I Verified Settings Matrix (`VERIFIED_SETTINGS_MATRIX.md`)

**Date:** 2026-07-31  
**Status:** Runtime Audit & Control Mapping  
**Scope:** Honest classification of every panel and control in `apps/settings/src/main.rs`.

---

## 1. Control Classification Legend

- **`FULLY FUNCTIONAL`**: Reads real system state, mutates real system state, and persists across restarts.
- **`PARTIALLY FUNCTIONAL`**: Mutates shell UI state immediately, but persists to local TOML file without D-Bus system daemon mutation.
- **`READ-ONLY`**: Displays system information but provides no mutation controls.
- **`MOCK / DISCONNECTED`**: UI control exists, but no backend implementation or D-Bus service is connected.
- **`UNSAFE SHELL COMMAND`**: Mutates state by executing unvalidated sub-process shell strings.

---

## 2. Settings Controls Audit

| Category / Panel | Control Label / Button | Event Handler Function | Backend Implementation | Audit Classification |
| :--- | :--- | :--- | :--- | :---: |
| **Appearance** | Theme Selector (`Light`, `Dark`, `Classic`, `Grape`, `Solarized`, `Dracula`, `High Contrast`) | `SettingsView::apply_theme` | `slopos_shell::theme_manager::ThemeManager::set_theme` + `settings.conf` persistence. | **PARTIALLY FUNCTIONAL** |
| **General** | Computer Name Field | `SettingsView::on_change` | Writes `computer_name` to `~/.config/slopos-i/settings.conf`. | **PARTIALLY FUNCTIONAL** |
| **Desktop & Dock**| Dock Position / Icon Size | `SettingsView::on_change` | Writes `dock_position` to `~/.config/slopos-i/settings.conf`. | **PARTIALLY FUNCTIONAL** |
| **Display** | Resolution / Scale Factor | `SettingsView::on_change` | Writes `display_scale` to `~/.config/slopos-i/settings.conf`; `CompositorAdapter` D-Bus IPC un-wired. | **MOCK / DISCONNECTED** |
| **Sound** | Output Volume Slider / Mute | `SettingsView::on_change` | Writes `volume` to `~/.config/slopos-i/settings.conf`; PipeWire D-Bus adapter (`AudioServiceTrait`) un-wired. | **MOCK / DISCONNECTED** |
| **Network** | Wi-Fi Network List / Connect | `SettingsView::on_change` | Writes `network_ssid` to `~/.config/slopos-i/settings.conf`; NetworkManager D-Bus adapter (`NetworkServiceTrait`) un-wired. | **MOCK / DISCONNECTED** |
| **Keyboard** | Key Repeat Rate / Delay | `SettingsView::on_change` | Writes `key_repeat` to `~/.config/slopos-i/settings.conf`; `xkbcommon` update un-wired. | **MOCK / DISCONNECTED** |
| **Mouse** | Pointer Speed / Double-Click | `SettingsView::on_change` | Writes `pointer_speed` to `~/.config/slopos-i/settings.conf`; `libinput` update un-wired. | **MOCK / DISCONNECTED** |
| **Accessibility**| High Contrast / Reduced Motion | `SettingsView::on_change` | `slopos_shell::a11y_prefs::A11yPrefs` + `settings.conf` persistence. | **PARTIALLY FUNCTIONAL** |
| **Notifications**| Enable Notifications / Sound | `SettingsView::on_change` | Writes `notifications_enabled` to `~/.config/slopos-i/settings.conf`. | **PARTIALLY FUNCTIONAL** |

---

## 3. Summary

- **0 Controls** are **FULLY FUNCTIONAL** via live Linux D-Bus daemons.
- **4 Panels** (Appearance, General, Desktop & Dock, Accessibility) are **PARTIALLY FUNCTIONAL** (updating shell UI and TOML configuration files).
- **6 Panels** (Display, Sound, Network, Keyboard, Mouse, Notifications) are **MOCK / DISCONNECTED** (UI controls store TOML values without driving NetworkManager, PipeWire, BlueZ, UPower, or libinput system daemons).
