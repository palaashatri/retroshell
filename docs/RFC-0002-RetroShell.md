# RFC-0002-RetroShell.md

Status: Accepted

Version: 1.0

Authors: RetroShell Architecture Team

---

# Abstract

RetroShell is the desktop runtime environment of the Retro platform.

RetroShell is responsible for:

* Desktop management
* Window management
* Global menu bar
* Dock
* Notifications
* Session management
* Workspaces
* Application launching

RetroShell is the primary user-facing component of the platform.

RetroShell is analogous to:

* Finder + System UI on Classic Mac OS
* Workspace Manager on NeXTSTEP
* Explorer Shell on Windows
* GNOME Shell
* KDE Plasma

Unlike those systems, RetroShell is designed as a tightly integrated desktop platform with a single coherent user experience.

---

# Goals

Provide:

* Consistent desktop experience
* Global menu bar
* Fast startup
* Low memory usage
* Stable APIs
* Future portability to RetroOS

---

# Non-Goals

RetroShell does not:

* Define widgets
* Render controls directly
* Replace RetroKit
* Replace RetroRender
* Replace application logic

---

# Architecture Overview

Applications
↓
RetroKit
↓
RetroShell Services
↓
RetroRender
↓
Wayland

---

# Core Components

RetroShell consists of:

MenuServer

WindowManager

DesktopManager

Dock

NotificationCenter

WorkspaceManager

LaunchServices

SessionManager

ThemeManager

ApplicationRegistry

---

# Ownership Model

RetroShell owns:

Desktop state

Window state

Menu state

Dock state

Notifications

Session state

Applications own:

Documents

Menus

Application logic

Views

RetroKit owns:

Widgets

Layout

Accessibility

Themes

---

# MenuServer

MenuServer implements the global menu bar.

MenuServer is always active.

Exactly one menu bar exists.

Location:

Top edge of primary display.

MenuServer owns:

Rendering

Menu switching

Keyboard shortcuts

Apple menu equivalent

Status menus

Applications never render menu bars.

---

# Menu Flow

Application launches.

Application publishes menu definition.

MenuServer registers menus.

User focuses application.

MenuServer activates corresponding menus.

Focus changes.

MenuServer updates menu contents.

Menu bar never changes position.

---

# Menu Structure

Required menus:

Application

File

Edit

View

Window

Help

Applications may add custom menus.

Applications may not remove mandatory menus.

---

# Status Area

Right side of menu bar.

Contains:

Clock

Network

Audio

Battery

Notifications

Future system indicators

Managed exclusively by RetroShell.

---

# WindowManager

WindowManager owns:

Window focus

Window ordering

Window placement

Workspace assignment

Fullscreen transitions

Window decorations

Applications own window contents only.

---

# Window Model

Every window exists in one state.

States:

Active

Inactive

Minimized

Maximized

Fullscreen

Hidden

Destroyed

Only one window is active at a time.

---

# Focus Model

Exactly one application is focused.

Exactly one window is focused.

Exactly one widget is focused.

Focus hierarchy:

Application
→ Window
→ Widget

---

# Window Placement

New windows should:

Avoid overlap when possible.

Remain visible.

Respect workspace boundaries.

Respect display boundaries.

Never spawn off-screen.

---

# Window Decorations

Provided exclusively by RetroShell.

Components:

Title bar

Close button

Minimize button

Zoom button

Resize regions

Shadow

Applications may not customize window chrome.

Consistency is mandatory.

---

# DesktopManager

DesktopManager controls:

Desktop background

Desktop icons

Mounted volumes

Desktop selections

Desktop interactions

Finder is responsible for rendering desktop contents.

DesktopManager owns desktop behavior.

---

# Desktop Behavior

Single click:

Select

Double click:

Open

Drag:

Move

Right click:

Context menu

Desktop behavior must be identical across themes.

---

# Dock

Dock provides:

Application launching

Running application indicators

Window switching

Trash

Recent applications

Dock remains visible by default.

Optional:

Auto-hide

Magnification

Positioning

Bottom position is default.

---

# Dock Item Types

Application

Folder

Document

Volume

Trash

Separator

Each item supports:

Selection

Drag-and-drop

Context menus

---

# WorkspaceManager

Supports multiple workspaces.

Capabilities:

Create workspace

Destroy workspace

Switch workspace

Move windows between workspaces

Workspace count configurable.

Default:

4 workspaces

---

# Workspace Behavior

Applications remain active.

Windows maintain state.

Workspace switching must be smooth.

Workspace switching must not restart applications.

---

# NotificationCenter

Centralized notification service.

Applications emit notifications.

RetroShell renders notifications.

Applications do not render system notifications.

---

# Notification Types

Information

Warning

Error

Progress

Notifications support:

Actions

Dismissal

Grouping

Persistence

---

# LaunchServices

LaunchServices owns:

Application discovery

Bundle registration

File associations

Document launching

Application launching

Applications are identified by:

Bundle Identifier

Example:

com.retro.finder

com.retro.textedit

com.retro.terminal

---

# Application Discovery

LaunchServices scans:

/Applications

/User/Applications

Bundles are registered automatically.

No package manager required.

---

# Bundle Format

Applications use:

Application.app

Structure:

Application.app/

```
App.toml

Executable/

Resources/

Assets/

Localization/
```

---

# File Associations

LaunchServices maintains mappings.

Example:

txt
→ TextEdit

png
→ ImageViewer

retrodoc
→ RetroEditor

Users may override defaults.

---

# SessionManager

Responsible for:

Login

Logout

Lock screen

Shutdown

Restart

Session restoration

---

# Session Restoration

Optional.

Supported:

Window positions

Open documents

Workspace assignments

Applications may opt out.

---

# ThemeManager

ThemeManager owns:

Theme loading

Theme switching

Theme persistence

Theme validation

Themes are loaded dynamically.

No application restart required.

---

# Supported Themes

Platinum

Graphite

OLED Graphite

High Contrast

Custom themes later.

---

# ApplicationRegistry

Tracks:

Installed applications

Running applications

Foreground application

Application metadata

Registry is owned by LaunchServices.

---

# HDR Integration

RetroShell must support:

HDR displays

HDR wallpapers

HDR-aware applications

HDR rendering occurs through RetroRender.

RetroShell never performs direct HDR rendering.

---

# VRR Integration

RetroShell should support:

Adaptive sync

Variable refresh displays

Frame pacing

RetroShell delegates implementation to RetroRender.

---

# Accessibility

Every RetroShell component must support:

Keyboard navigation

Screen readers

Focus traversal

High contrast

Large text

Accessibility compliance is mandatory.

---

# Input Handling

Supported devices:

Mouse

Keyboard

Trackpad

Graphics tablet

Future:

Touchscreens

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

Applications do not directly manage hardware input.

---

# Logging

Every component logs through shared APIs.

Required levels:

TRACE

DEBUG

INFO

WARN

ERROR

Structured logging preferred.

---

# Performance Targets

Cold startup:

< 500ms

Menu switching:

< 16ms

Workspace switch:

< 100ms

Dock interaction:

< 16ms

Notification display:

< 16ms

Memory usage:

Minimal and predictable.

---

# Security Boundaries

Applications cannot:

Control other applications

Control RetroShell

Manipulate workspace state

Override menu bar ownership

All privileged actions occur through approved APIs.

---

# Future RetroOS Compatibility

RetroShell must not assume:

Linux process model

Linux filesystems

Linux-specific IPC

Linux-specific services

Platform abstractions are mandatory.

---

# Definition of Success

A user should perceive RetroShell as:

A complete operating environment.

Not a collection of components.

Not a Linux customization.

Not a themed desktop.

The desktop should feel cohesive, intentional, and integrated, similar to how Classic Mac OS, NeXTSTEP, and early Macintosh systems presented a unified computing experience.
