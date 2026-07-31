# SLOPOS-I Settings Subsystem Backend Architecture (`SETTINGS_BACKEND_ARCHITECTURE.md`)

**Date:** 2026-07-31  
**Status:** Authoritative Service Specification  
**Scope:** Architecture document defining the separation between Settings UI panels, typed Rust service interfaces, and system D-Bus adapters.

---

## 1. Subsystem Architecture Overview

Settings in SLOPOS-I strictly separate the retro System 7 UI from system mutation logic. UI controls NEVER execute raw shell commands directly. Instead, UI events invoke typed Rust service interfaces, which delegate to standard Linux D-Bus services and system daemons.

```
+-----------------------------------------------------------------------------------+
|                            Settings UI Layer (apps/settings)                      |
|   - AppearancePanel    - NetworkPanel    - SoundPanel     - DisplayPanel          |
|   - MousePanel         - KeyboardPanel   - PowerPanel     - AccountsPanel         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
|                        Settings Service Interface (crates/slopos-bus)             |
|   - NetworkServiceTrait                  - AudioServiceTrait                      |
|   - DisplayServiceTrait                  - PowerServiceTrait                      |
|   - SessionServiceTrait                  - StorageServiceTrait                    |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
|                     Native D-Bus / System Service Adapters                        |
|   - NetworkManagerAdapter   (org.freedesktop.NetworkManager)                      |
|   - PipeWireAdapter         (org.pulseaudio.ServerLookup / PipeWire IPC)          |
|   - UPowerAdapter           (org.freedesktop.UPower)                              |
|   - LogindAdapter           (org.freedesktop.login1)                              |
|   - UDisksAdapter           (org.freedesktop.UDisks2)                             |
|   - CompositorAdapter       (slopos-compositor RANDR IPC)                         |
+-----------------------------------------------------------------------------------+
```

---

## 2. Service Interface Contracts

### 2.1 NetworkServiceTrait
```rust
pub trait NetworkServiceTrait {
    fn scan_wifi_networks(&self) -> Result<Vec<WifiNetwork>, NetworkError>;
    fn connect_wifi(&self, ssid: &str, password: &str) -> Result<(), NetworkError>;
    fn disconnect(&self, interface: &str) -> Result<(), NetworkError>;
    fn get_status(&self) -> NetworkStatus;
}
```

### 2.2 AudioServiceTrait
```rust
pub trait AudioServiceTrait {
    fn get_master_volume(&self) -> f32;
    fn set_master_volume(&self, volume: f32) -> Result<(), AudioError>;
    fn is_muted(&self) -> bool;
    fn set_muted(&self, muted: bool) -> Result<(), AudioError>;
}
```

### 2.3 PowerServiceTrait
```rust
pub trait PowerServiceTrait {
    fn get_battery_level(&self) -> Option<f32>;
    fn is_charging(&self) -> bool;
    fn suspend(&self) -> Result<(), PowerError>;
    fn shutdown(&self) -> Result<(), PowerError>;
    fn reboot(&self) -> Result<(), PowerError>;
}
```

---

## 3. Persistence & Configuration Sync

1. **Local Settings Persistence**: User preferences (theme, font scale, sound volume, mouse speed) are stored in `~/.config/slopos-i/settings.conf` using typed TOML serialization.
2. **System State Sync**: On startup, `Settings` syncs local preferences with active D-Bus services (e.g. applying saved volume to PipeWire and applying saved display layout to `slopos-compositor`).
3. **Immediate Shell Notification**: Theme updates emit a high-priority IPC event on `/tmp/slopos-bus.sock` so `slopos-shell` repaints immediately without requiring a restart.
