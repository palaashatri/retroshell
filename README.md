# RetroShell

A native Rust desktop environment experiment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is NOT a Linux desktop theme. It is a complete, self-contained desktop platform with its own window manager, native widget toolkit, GPU-accelerated rendering pipeline, first-party SDK, and application suite.

---

## 1. System Architecture

```
Applications (Finder, Settings, TextEdit, Terminal, App Store)
    ↓ (linked statically or dynamically via SDK)
RetroSDK (Application wrapper, preference engine, appearance API)
    ↓
RetroKit (Native widgets, custom Layout grids, Event dispatching)
    ↓
RetroShell (Session manager, global menu bar, window workspace routing, Dock)
    ↓
RetroRender (wgpu rendering engine, cosmic-text rasterizer, vulkan context)
    ↓
Wayland / X11 Compositor (e.g., labwc / Xvfb) -> Linux Kernel
```

### Core Components
* **`retro-shell`**: The primary desktop compositor runtime, managing global menus, dock layouts, active workspaces, notification dispatch, and desktop/window management.
* **`retro-kit`**: A custom, lightweight native GUI toolkit written in Rust, defining widgets, scroll containers, list view behaviors, grids, and event handling.
* **`retro-render`**: A fast graphics pipeline built on `wgpu`. Handles low-level pipeline setup, vertex shaders, and texture upload. Integrated with `cosmic-text` for glyph layout and rasterization.
* **`retro-sdk`**: The software development kit for RetroShell. Houses the native menu routing, configuration stores, and the common application entrypoint loops.
* **`retro-bus`**: A low-level IPC implementation supporting service discovery, cross-process event propagation, and communication between apps and shell services.

---

## 2. Technology Stack & Specs
* **Language**: Rust (edition 2021)
* **Rendering API**: WebGPU (`wgpu` targeting Vulkan/Wayland/X11 backends)
* **Text Rendering**: `cosmic-text` font database and rasterizer
* **Audio**: PipeWire & PulseAudio
* **Compositor integration**: Wayland client protocol + wlr-protocols (running nested or direct)

---

## 3. Visual & Coding Standards

### Design Guidelines (Style Guide)
* **Metaphor Consistency**: Retain a compact, highly dense, classic desktop visual metaphor (Mac OS System 7 to OS 9 style).
* **Color Palette**: Calm, low-saturation gray tones (Platinum) by default. Use token-based colors to avoid ad-hoc values.
* **No Web Noise**: Avoid rounded borders exceeding 4px, heavy drop shadows, or high-contrast modern web visual noise.
* **Native Dark Mode**: Support dark mode natively using system theme preference stores. Color selection dynamically switches based on appearance keys.

### Coding Standards
* **Rust Idioms**: Avoid unnecessary `unsafe` blocks. Prefer clear ownership boundaries and leverage type systems for state machines.
* **Widget Structure**: All widgets must inherit from base layouts and pass rendering tasks back to the active `Canvas` using NDC coordinate translation.
* **Portability**: Keep the core crates decoupled from OS-specific backends where possible to facilitate future portability to custom kernels.

---

## 4. First-Party Application Suite
* **Finder**: The file and spatial navigation shell. Displays directory items in grids, manages file operations (New Folder, Duplicate, Delete), and shows file metadata (`INFO`).
* **Settings**: Manages system state. Allows toggling appearance (Light/Dark) and writing `settings.conf`.
* **TextEdit**: Lightweight text editor supporting multi-line text fields, dirty-state tracking, and disk I/O.
* **Terminal**: PTY-backed terminal emulator supporting interactive shells, selection copy/paste, and custom scrollback.
* **App Store**: Package manager front-end querying system package indices (e.g. `APT`) and running package searches.

---

## 5. Development & Verification VM

RetroShell comes with a Docker-based visual environment (`retroshell-vm`) for visual development and automated testing in a sandboxed Wayland/X11 compositor.

### Building and Running the VM
1. **Build the container**:
   ```bash
   docker build -f Dockerfile.vm -t retroshell-vm .
   ```
2. **Start the environment**:
   ```bash
   docker run -d -p 6080:6080 -v "$(pwd):/app" --name retroshell-running retroshell-vm
   ```
3. **Connect to the Desktop**:
   Open a browser and navigate to `http://localhost:6080/vnc.html` to access the desktop session.

### Running Applications inside the VM
Compile the workspace and launch the desktop shell:
```bash
docker exec -t retroshell-running cargo build --release
docker exec -d retroshell-running env -u DISPLAY WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/tmp/runtime-root /app/target-docker/release/retro-shell
```
