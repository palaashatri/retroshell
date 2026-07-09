# RetroShell Configuration

---

## settings.conf

The main configuration file is read and written by the Settings application and
by `retro-shell`'s `ThemeManager`. It is a flat key=value text file (no sections,
no TOML/INI syntax — just `key=value` lines).

### Location

| Priority | Path                                                        |
|----------|-------------------------------------------------------------|
| 1        | `$RETROSHELL_CONFIG_DIR/settings.conf`                      |
| 2        | `$HOME/.config/retroshell/settings.conf`                    |
| 3        | `/tmp/retroshell/settings.conf` (fallback)                  |

The directory is created automatically if it does not exist.

### Keys

#### Appearance

| Key          | Values                                                                       | Default   | Description                              |
|--------------|----------------------------------------------------------------------|-----------|------------------------------------------|
| `theme`      | `classic` `dark` `grape` `blueberry` `strawberry` `solarized` `dracula` `highcontrast` | `classic` | Named color theme. Takes precedence over `appearance`. |
| `appearance` | `light` `dark`                                  | `light`   | Legacy light/dark toggle. Used if `theme` is absent. |

#### Sound

| Key              | Values    | Default | Description                        |
|------------------|-----------|---------|------------------------------------|
| `sound_volume`   | `0`–`100` | `50`    | System volume preference (integer) |

#### Pointing device

| Key                | Values    | Default | Description                        |
|--------------------|-----------|---------|-------------------------------------|
| `mouse_speed`      | `0`–`100` | `50`    | Mouse pointer speed preference      |

#### Display

| Key             | Values     | Default | Description                                |
|-----------------|------------|---------|--------------------------------------------|
| `hdr_request`   | `true` `false` | `false` | Request HDR output when available. Requires compositor-level color management — not yet active. |
| `vrr_adaptive`  | `true` `false` | `false` | Prefer VRR/adaptive sync when available. Requires compositor-level presentation control — not yet active. |

#### Network (preferences only — no live network control)

| Key              | Values                | Default   | Description                        |
|------------------|-----------------------|-----------|------------------------------------|
| `wifi_enabled`   | `true` `false`        | `true`    | Stored preference                  |
| `network_location` | any string          | `Automatic` | Stored location name              |

#### Keyboard

| Key                   | Values         | Default | Description                                      |
|-----------------------|----------------|---------|--------------------------------------------------|
| `key_repeat_delay`    | integer (ms)   | `500`   | Delay before key repeat begins                   |
| `key_repeat_interval` | integer (ms)   | `50`    | Interval between repeated keys                   |

#### Accessibility

| Key                       | Values         | Default | Description                              |
|---------------------------|----------------|---------|------------------------------------------|
| `reduce_motion`           | `true` `false` | `false` | Preference flag (no animation engine yet) |
| `increase_contrast`       | `true` `false` | `false` | Preference flag                          |
| `bold_text`               | `true` `false` | `false` | Preference flag                          |

#### Notifications

| Key                    | Values         | Default | Description                                    |
|------------------------|----------------|---------|------------------------------------------------|
| `do_not_disturb`       | `true` `false` | `false` | Suppress notification banners when enabled     |

### Example settings.conf

```
theme=classic
sound_volume=70
mouse_speed=50
hdr_request=false
vrr_adaptive=false
wifi_enabled=true
key_repeat_delay=500
key_repeat_interval=50
reduce_motion=false
increase_contrast=false
bold_text=false
do_not_disturb=false
```

---

## Environment Variables

### Runtime variables

| Variable                           | Description                                                                           |
|------------------------------------|---------------------------------------------------------------------------------------|
| `RETROSHELL_CONFIG_DIR`            | Override the directory that contains `settings.conf`. Overrides `$HOME/.config/retroshell`. |
| `RETROSHELL_GLOBAL_MENU`           | Set to `1` by `retro-shell` when launching first-party apps. Suppresses the app's own in-window menu bar so the shell can show it system-wide. |
| `RETROSHELL_MENU_MANIFEST_DIR`     | Directory where apps write JSON menu manifests. Defaults to `$XDG_RUNTIME_DIR/retroshell/menus`. |
| `RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES` | Set to `1` to allow App Store to execute package-install/remove transactions. Off by default for safety. |
| `WAYLAND_DISPLAY`                  | Standard Wayland socket name (e.g. `wayland-0`). Required when running outside a session that sets it automatically. |
| `XDG_RUNTIME_DIR`                  | Standard XDG runtime directory. Used for menu manifests, clipboard persistence, and PTY socket paths. |
| `DISPLAY`                          | X11 display variable. Used by the VM stack (labwc via Xvfb). Unset it (`env -u DISPLAY`) when running as a Wayland client. |

### VM / Docker variables

| Variable              | Default  | Description                                     |
|-----------------------|----------|-------------------------------------------------|
| `RETROSHELL_VM_WIDTH` | `1280`   | Virtual framebuffer width in pixels             |
| `RETROSHELL_VM_HEIGHT`| `800`    | Virtual framebuffer height in pixels            |
| `RETROSHELL_VM_DEPTH` | `24`     | Color depth for Xvfb                            |

---

## Theme System

### Named themes

RetroShell ships eight named themes. Set the `theme` key in `settings.conf` or select
via Settings > Appearance.

| Theme name    | Key          | Mode  | Accent color          | Description                              |
|---------------|--------------|-------|-----------------------|------------------------------------------|
| Classic       | `classic`    | Light | Blue (0.36, 0.54, 0.85) | Mac OS 7–9 Platinum; default             |
| Dark          | `dark`       | Dark  | Blue (0.36, 0.54, 0.85) | Dark variant of Classic                  |
| Grape         | `grape`      | Dark  | Purple (0.55, 0.28, 0.72) | Purple-tinted dark theme               |
| Blueberry     | `blueberry`  | Dark  | Deep blue (0.15, 0.25, 0.62) | Deep blue dark theme                |
| Strawberry    | `strawberry` | Light | Red-orange (0.82, 0.23, 0.28) | Warm red-orange tinted theme        |
| Solarized     | `solarized`  | Dark  | Blue (0.16, 0.54, 0.82) | Solarized dark theme                     |
| Dracula       | `dracula`    | Dark  | Purple (0.74, 0.58, 0.98) | Dracula dark theme                       |
| HighContrast  | `highcontrast` | Light | Yellow (1.0, 0.84, 0.0) | Pure black/white with yellow accent      |

### Internal palette layers

The theme system has two additional internal palettes used as building blocks. These
are not selectable from Settings but underpin the named themes:

| Internal key    | Description                                       |
|-----------------|---------------------------------------------------|
| `platinum`      | Base light palette; Classic and Graphite inherit from it |
| `graphite`      | Grayscale accent variant of Platinum              |
| `oled-graphite` | Pure-black OLED variant; no dark/light variants   |
| `high-contrast` | Accessibility high-contrast; pure black/white     |

### Theme tokens

Every widget resolves colors through `ThemeToken` values rather than hard-coded RGBA.
The full token list is in `crates/retro-kit/src/theme.rs`. Key tokens:

| Token                  | Light (Classic)          | Dark (Dark theme)        |
|------------------------|--------------------------|--------------------------|
| `WindowBackground`     | #F2F2F2 (0.95 gray)      | #262626 (0.15 gray)      |
| `WindowBorder`         | #808080                  | #4D4D4D                  |
| `MenuBackground`       | #FAFAFA                  | #1F1F1F                  |
| `MenuHighlight`        | #3870D9 (blue accent)    | #3870D9                  |
| `ButtonBackground`     | #E0E0E0                  | #333333                  |
| `DesktopBackground`    | #404073 (indigo)         | #141428 (dark indigo)    |
| `DockBackground`       | Semi-transparent light   | Semi-transparent dark    |
| `TextPrimary`          | #000000                  | #FFFFFF                  |
| `SelectionBackground`  | #3870D9                  | #3870D9                  |
| `FocusRing`            | #3870D9                  | #3870D9                  |

### Adding a custom palette

Custom palettes can be added in `ThemeManager::load_default()` in
`crates/retro-shell/src/theme_manager.rs`. Insert a new method following the pattern
of `load_platinum()` and call it from `load_default()`. The palette is then available
via `ThemeManager::set_theme("your-key")`.

Custom palettes are not yet selectable from the Settings UI; that requires adding a
new `ThemeName` variant and wiring it through the Settings preference pane.

---

## App Bundle Format

Each application is distributed as a bundle directory:

```
MyApp.app/
  App.toml          — bundle metadata
  Executable/
    myapp           — compiled binary
  Resources/        — icons, localization files
  Assets/           — additional data files
```

`App.toml` fields:

| Field              | Type          | Description                                            |
|--------------------|---------------|--------------------------------------------------------|
| `name`             | string        | Human-readable application name                        |
| `bundle_id`        | string        | Reverse-DNS identifier, e.g. `com.retro.finder`        |
| `version`          | string        | Semantic version                                       |
| `author`           | string        | Author name                                            |
| `minimum_platform` | string        | Minimum RetroShell platform version                    |
| `entrypoint`       | string        | Relative path to binary inside bundle                  |
| `file_types`       | string array  | File extensions this app can open; `["*"]` means all   |
| `permissions`      | string array  | Required permissions: `filesystem`, `process`, `pty`, `documents`, `system` |
| `category`         | string        | App Store category (optional)                          |

The shell scans for bundles at startup using `LaunchServices::scan_applications()`.
It searches the directory of the `retro-shell` binary and `target/{debug,release}/Applications/`.
