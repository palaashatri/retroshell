# ARCHITECTURE.md

# RetroShell Architecture

Version: 1.0

Status: Authoritative

This document defines the architecture of RetroShell.

Any implementation that conflicts with this document is considered incorrect.

---

# Purpose

RetroShell is a desktop platform inspired by:

- Classic Mac OS
- NeXTSTEP
- BeOS

RetroShell is not:

- A Linux desktop theme
- A KDE customization
- A GNOME extension

RetroShell is a complete desktop environment with its own:

- Window manager
- Menu system
- Theme engine
- Desktop manager
- Application framework
- Application suite

The long-term objective is portability to RetroOS.

---

# System Overview

Applications
↓
RetroKit
↓
RetroShell Services
↓
RetroRender
↓
Wayland
↓
Linux

---

# Top-Level Components

The system consists of:

1. RetroShell
2. RetroKit
3. RetroRender
4. RetroBus
5. Applications

Each component has strict ownership boundaries.

---

# Component: RetroShell

RetroShell owns desktop behavior.

Responsibilities:
- Menu bar
- Desktop
- Dock
- Window management
- Workspace management
- Notifications
- Session management

RetroShell must not:
- Define widgets
- Implement application logic
- Implement rendering primitives

---

# Component: RetroKit

RetroKit is the UI framework.

Responsibilities:
- Widgets
- Layout systems
- Event handling
- Accessibility
- Theme integration

RetroKit provides:

Window
Button
Menu
MenuItem
Toolbar
ListView
TreeView
IconView
TextField
ScrollView
Dialog
StatusBar

RetroKit must not:
- Manage windows globally
- Manage desktop state
- Own system services

---

# Component: RetroRender

RetroRender owns all rendering.

Responsibilities:
- GPU rendering
- Vulkan abstraction
- Text rendering
- HDR pipeline
- VRR support
- Asset rendering

RetroRender exposes:

Renderer
Texture
Surface
Font
Shader
ThemeRenderer

RetroRender must not:
- Know application logic
- Know desktop logic

---

# Component: RetroBus

RetroBus is the communication layer.

Initial implementation:
D-Bus

Future implementation:
Native IPC

Responsibilities:
- Service discovery
- Event dispatch
- Broadcast notifications
- Menu synchronization

---

# Component: Applications

Applications contain user functionality.

Examples:
Finder
Settings
Terminal
TextEdit

Applications must use RetroKit.

Applications may not implement custom window systems.

Applications may not bypass RetroShell.

---

# Window System

RetroShell owns all windows.

Applications request windows.

RetroShell controls:
- Position
- Activation
- Focus
- Workspaces

Applications control:
- Contents
- Menus
- Document state

---

# Window Lifecycle

Application requests:
CreateWindow()

RetroShell:
- allocates window
- registers window
- applies theme
- renders frame

Application receives:
WindowHandle

Application populates content.

---

# Window States

Supported:
Normal
Minimized
Maximized
Fullscreen
Hidden
Inactive
Destroyed

---

# Window Decorations

Provided exclusively by RetroShell.

Applications may not draw:
- title bars
- borders
- window controls

Decorations remain visually consistent.

---

# Global Menu Bar

One menu bar exists.

Location:
Top of primary display.

Ownership:
RetroShell.

Applications provide menu definitions.

RetroShell renders menus.

Applications do not.

---

# Menu Flow

Application gains focus.

Application publishes:
File
Edit
View
Help

RetroShell receives menu definition.

RetroShell redraws menu bar.

Application loses focus.

RetroShell switches menus.

---

# Desktop

Desktop is a first-class component.

Desktop displays:
Files
Folders
Mounted Volumes

Desktop is implemented by Finder.

---

# Finder Ownership

Finder owns:
Desktop icons
File browsing
Volume browsing
Drag-and-drop file operations
Application launching

Finder does not own:
Window management
Menu rendering
Dock behavior

---

# Dock

The Dock is managed by RetroShell.

Capabilities:
Application launch
Running application indicators
Trash
Recent applications
Window switching

The Dock remains visible unless auto-hide is enabled.

---

# Theme System

Themeing is token based.

Widgets consume theme tokens.

Widgets never consume raw colors.

Example:
Window.Background
Window.Border
Window.Title
Menu.Background
Menu.Highlight
Button.Light
Button.Dark
Button.Text

---

# Theme Variants

Platinum
Graphite
OLED Graphite
High Contrast

Every widget must support every theme.

---

# Dark Mode

Dark mode is not generated automatically.

Dark mode assets are designed intentionally.

Every asset requires:
Light variant
Dark variant
Optional HDR variant

---

# Rendering Pipeline

Application
↓
RetroKit
↓
Render Tree
↓
RetroRender
↓
wgpu
↓
Vulkan
↓
GPU

Applications never access Vulkan directly.

---

# HDR Architecture

HDR support exists at renderer level.

Requirements:
Wide color gamut
HDR surfaces
Color-managed rendering
Theme support

Application support through RetroKit.

Applications should not implement HDR logic.

---

# VRR Architecture

VRR is managed by RetroRender.

RetroShell requests frame pacing.

Display backend determines refresh synchronization.

Applications remain unaware.

---

# Accessibility Architecture

Every widget must expose:
Accessible role
Accessible name
Accessible state
Keyboard navigation
Focus traversal
Screen reader support

Accessibility support is mandatory.

---

# Input System

Supported devices:
Keyboard
Mouse
Trackpad
Tablet

Input flow:
Device
↓
Wayland
↓
RetroShell
↓
RetroKit
↓
Application

Applications do not communicate directly with input devices.

---

# Bundle Architecture

Applications are bundles.

Example:
Finder.app

Bundle structure:
Finder.app/
    App.toml
    Executable/
    Resources/
    Assets/
    Localization/

---

# Application Metadata

App.toml contains:
Name
Identifier
Version
Author
Minimum Platform Version
Supported Themes
Declared Menus
Declared Permissions

---

# Settings Architecture

Centralized.

Applications never maintain their own settings storage.

Settings service provides:
ReadSetting()
WriteSetting()
ObserveSetting()

---

# Notification Architecture

Applications emit notifications.

RetroShell renders notifications.

Applications never render system notifications.

---

# Process Model

One process per application.
One process for RetroShell.
Shared services run separately.

Crash isolation is required.

Application crashes must not terminate RetroShell.

---

# Logging

All components log through common interfaces.

Levels:
TRACE
DEBUG
INFO
WARN
ERROR

Logs must be structured.

---

# Testing Architecture

Every component requires:
Unit tests
Integration tests
Visual regression tests
Accessibility tests
Performance tests

---

# Future Portability

RetroShell must avoid direct Linux assumptions.

Platform-specific code belongs behind abstraction layers.

Required abstraction boundaries:
Filesystem
Process Management
IPC
Audio
Graphics
Display
Input

This ensures future migration to RetroOS.

---

# Architectural Priorities

Priority 1
Consistency

Priority 2
Performance

Priority 3
Simplicity

Priority 4
Portability

Priority 5
Extensibility

Every architectural decision should be evaluated in this order.

---

# Definition of Success

A user should be able to boot RetroShell and immediately recognize:
- a coherent desktop platform
- a unified application ecosystem
- a modern continuation of the Classic Mac OS philosophy

The desktop should feel purpose-built rather than assembled from unrelated components.
