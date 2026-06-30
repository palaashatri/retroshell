# RetroShell

A modern desktop environment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is NOT a Linux desktop theme. RetroShell is a complete desktop platform.

## Technology

| Component | Description | Language |
|-----------|-------------|----------|
| RetroRender | GPU rendering engine (wgpu/Vulkan) | Rust |
| RetroKit | Native UI widget framework | Rust |
| RetroShell | Desktop environment runtime | Rust |
| RetroBus | IPC and service communication | Rust |
| RetroSDK | Application development SDK | Rust |

## Architecture

```
Applications (Finder, Settings, TextEdit, Terminal)
    ↓
RetroKit (Widgets, Layout, Events, Accessibility)
    ↓
RetroShell (Menu, Windows, Dock, Desktop, Workspaces)
    ↓
RetroRender (wgpu, Vulkan, cosmic-text)
    ↓
Wayland → Linux
```

## Getting Started

```bash
cargo build --release
cargo run -p retro-shell
```

## Themes

- Platinum (light, default)
- Graphite (light, gray accent)
- OLED Graphite (dark, OLED-optimized)
- High Contrast (accessibility)

## Built-in Applications

- Finder - File management and desktop
- Settings - System configuration
- TextEdit - Text/document editor
- Terminal - Command line (planned)

## Project Structure

```
crates/
  retro-render/     GPU rendering engine
  retro-kit/        UI widget toolkit
  retro-shell/      Desktop environment
  retro-bus/        IPC layer
  retro-sdk/        Application SDK
apps/
  finder/           File manager
  settings/         System settings
  textedit/         Text editor
themes/
  platinum/         Classic theme
  graphite/         Gray accent theme
  oled-graphite/    OLED dark theme
  high-contrast/    Accessibility theme
```
