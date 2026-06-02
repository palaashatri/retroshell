# AGENTS.md

# RetroShell

A modern desktop environment inspired by Classic Mac OS, NeXTSTEP, and BeOS.

RetroShell is NOT a Linux desktop theme.

RetroShell is NOT a KDE fork.

RetroShell is NOT a GNOME fork.

RetroShell is a complete desktop platform consisting of:

* RetroShell (desktop environment)
* RetroKit (GUI toolkit)
* RetroRender (rendering engine)
* RetroBus (IPC layer)
* First-party applications

The long-term goal is migration onto RetroOS, a future microkernel operating system.

---

# Mission

Build the desktop environment that Apple might have created if the Classic Mac OS design language had continued evolving into the modern era.

The desktop must preserve:

* Global menu bar
* Desktop metaphor
* Application bundles
* Consistent widgets
* Simplicity
* Low resource usage

The desktop must support:

* HDR
* VRR
* HiDPI
* Accessibility
* Multiple monitors
* Modern input devices

---

# Technology Stack

Language:

* Rust (stable)

Rendering:

* wgpu

Graphics Backend:

* Vulkan

Display Protocol:

* Wayland

Text Rendering:

* cosmic-text

Audio:

* PipeWire

IPC:

* D-Bus (temporary)
* RetroBus (future)

Target Platforms:

* Linux (Phase 1)
* BSD (Phase 2)
* RetroOS (Phase 3)

---

# Core Principles

## Principle 1

Desktop First.

RetroShell is designed for desktop computers.

Mobile-first design is prohibited.

Touch-first design is prohibited.

---

## Principle 2

Applications Feel Related.

Every application must look like it belongs to the same operating system.

No application should appear visually disconnected from the rest of the platform.

---

## Principle 3

One Menu Bar.

There is exactly one menu bar.

Applications define menu content.

RetroShell owns menu rendering.

Applications never draw their own menu bars.

---

## Principle 4

No Web Technologies.

Do not introduce:

* Electron
* React
* Angular
* Vue
* Chromium UI
* WebView UI

All interface components must be native RetroKit widgets.

---

## Principle 5

Performance Matters.

Prefer:

* Static dispatch
* Efficient allocations
* GPU acceleration

Avoid:

* Heavy abstractions
* Runtime reflection
* Unnecessary background services

---

## Principle 6

Visual Consistency.

All widgets originate from RetroKit.

No custom widget implementations unless approved by architecture RFC.

---

## Principle 7

Theme Independence.

Widgets never hardcode colors.

Widgets never hardcode fonts.

Widgets consume theme tokens only.

---

## Principle 8

Application Bundles.

Applications are distributed as:

Application.app

Bundles contain:

* executable
* assets
* metadata
* resources

No package-manager dependency assumptions.

---

# Visual Design Rules

Inspired by:

* Mac OS 8
* Mac OS 9
* NeXTSTEP
* BeOS

Avoid visual inspiration from:

* Material Design
* Fluent Design
* GNOME
* KDE
* Mobile operating systems

Design characteristics:

* Sharp edges
* High contrast
* Minimal transparency
* Visible window borders
* Distinct active/inactive states
* Small efficient toolbars
* Compact controls

---

# Themes

Supported themes:

1. Platinum
2. Graphite
3. OLED Graphite
4. High Contrast

Every widget must support every theme.

Theme support is mandatory.

---

# Architecture

Applications
↓
RetroKit
↓
RetroShell
↓
RetroRender
↓
Wayland
↓
Linux

---

# Component Ownership

## RetroShell

Owns:

* Desktop
* Dock
* Menu Bar
* Window Management
* Notifications
* Workspace Management

Must not:

* Render custom widgets
* Contain business logic belonging to applications

---

## RetroKit

Owns:

* Window
* Button
* Menu
* ListView
* TreeView
* IconView
* TextField
* Toolbar
* Dialog
* ScrollView

Must not:

* Manage windows
* Manage desktop state

---

## RetroRender

Owns:

* GPU rendering
* Text rendering
* HDR pipeline
* VRR support
* Theme rasterization

Must not:

* Contain UI logic

---

## RetroBus

Owns:

* Service communication
* Event dispatch
* Application integration

Must not:

* Render UI

---

# Built-in Applications

Priority 1

* Finder
* Settings
* Terminal
* TextEdit

Priority 2

* Calculator
* Image Viewer
* Archive Utility

Priority 3

* Music
* Calendar
* Mail

---

# Finder Requirements

Finder is the canonical file manager.

Capabilities:

* Desktop icons
* Volume management
* Application launching
* File operations
* Drag and drop

Finder defines expected platform behavior.

---

# Dark Mode Requirements

Dark mode is not an inversion filter.

Dark mode is a first-class theme.

Every asset must support:

* Platinum
* Graphite
* OLED Graphite

Dark mode support is required before merge.

---

# Accessibility

Required:

* Keyboard navigation
* Screen reader compatibility
* High contrast support
* Scalable text

Accessibility is not optional.

---

# Testing Requirements

Every feature requires:

1. Unit tests
2. Integration tests

UI components require:

3. Visual regression tests

No feature is complete without tests.

---

# AI Agent Roles

## Architect Agent

Responsible for:

* RFCs
* API contracts
* Long-term architecture

Never writes production code.

---

## Framework Agent

Responsible for:

* RetroKit

Never changes shell architecture.

---

## Shell Agent

Responsible for:

* Menu bar
* Desktop
* Window manager
* Dock

Never changes RetroKit internals.

---

## Application Agent

Responsible for:

* Finder
* Settings
* TextEdit
* Terminal

Never modifies framework code.

---

## Rendering Agent

Responsible for:

* wgpu
* Vulkan integration
* HDR
* VRR

Never modifies application logic.

---

## QA Agent

Responsible for:

* Testing
* Regression validation
* Performance analysis

Never implements features.

---

# Pull Request Rules

Every pull request must answer:

1. What problem is being solved?
2. Why is this solution correct?
3. Does it violate any architecture rule?
4. Does it introduce visual inconsistency?
5. Are tests included?

If any answer is missing:

Reject the change.

---

# Success Criteria

A successful RetroShell release should feel like:

Classic Mac OS evolved for modern hardware.

It should not feel like:

* KDE with a theme
* GNOME with extensions
* Windows clone
* Electron desktop

Every design decision should move the platform closer to a coherent, integrated desktop operating environment.
