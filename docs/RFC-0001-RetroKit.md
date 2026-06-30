# RFC-0001-RetroKit.md

Status: Accepted

Version: 1.0

Authors: RetroShell Architecture Team

---

# Abstract

RetroKit is the native user interface framework for RetroShell.

RetroKit serves the role performed by:

* Cocoa on macOS
* AppKit on NeXTSTEP
* Qt on KDE
* GTK on GNOME
* WinUI on Windows

RetroKit provides:

* Windowing primitives
* Widgets
* Layout systems
* Event systems
* Accessibility support
* Theme integration
* Rendering integration

RetroKit is the only supported UI framework for first-party applications.

---

# Goals

RetroKit exists to provide:

1. Visual consistency
2. Predictable behavior
3. High performance
4. Accessibility
5. Theme support
6. Long-term platform stability

Applications built today must continue to function on future RetroShell and RetroOS releases.

---

# Non-Goals

RetroKit is not:

* A web framework
* A browser runtime
* A JavaScript runtime
* A scene graph editor
* A game engine

RetroKit does not provide:

* Networking
* Audio
* Filesystem access
* Process management

Those responsibilities belong elsewhere.

---

# Design Principles

## Principle 1

Every application should look like it belongs to the platform.

---

## Principle 2

Every widget should behave identically across applications.

---

## Principle 3

Themes should alter appearance but never behavior.

---

## Principle 4

Accessibility is mandatory.

---

## Principle 5

Keyboard navigation is a first-class feature.

---

## Principle 6

Widgets must remain lightweight.

---

# Architecture

Applications
↓
RetroKit
↓
RetroRender
↓
Wayland

RetroKit never talks directly to GPU APIs.

RetroRender owns rendering.

---

# Framework Structure

retrokit/

```
core/
widgets/
layout/
accessibility/
theme/
events/
text/
rendering/
platform/
```

---

# Core Concepts

Every visible element derives from:

Widget

Pseudo-interface:

trait Widget {

```
fn layout();

fn draw();

fn handle_event();

fn accessibility();
```

}

All visual controls derive from Widget.

---

# Widget Tree

Applications build a widget hierarchy.

Example:

Window

```
Toolbar

    Button

    Button

SplitView

    Sidebar

    ContentView
```

Widgets form a tree.

Widgets never own windows.

---

# Window

Window is the root user-visible container.

Capabilities:

* Title
* Toolbar
* Content area
* Resize support
* Focus handling

Applications create windows.

RetroShell decorates windows.

Applications never draw:

* borders
* title bars
* window controls

---

# Menu System

Menus are declarative.

Applications define menu structure.

Example:

MenuBar

```
File

    New

    Open

    Save

Edit

    Cut

    Copy

    Paste
```

RetroShell renders the menu bar.

Applications do not.

---

# Buttons

Button types:

PushButton

ToggleButton

RadioButton

ToolbarButton

Buttons support:

* keyboard activation
* hover state
* focus state
* accessibility

Buttons never contain hardcoded colors.

---

# TextField

Capabilities:

* selection
* clipboard integration
* undo
* redo
* IME support
* accessibility

Required:

UTF-8 support

Unicode support

Bi-directional text support

---

# ListView

Displays ordered collections.

Capabilities:

* single selection
* multi-selection
* sorting
* keyboard navigation

Must support thousands of items efficiently.

---

# TreeView

Displays hierarchical structures.

Examples:

File browsers

Settings categories

Outline views

Supports:

expand

collapse

selection

keyboard navigation

---

# IconView

Displays icon grids.

Primary usage:

Finder

Capabilities:

icon

title

selection

drag and drop

multi-selection

---

# ScrollView

Provides scrolling behavior.

Features:

vertical scrolling

horizontal scrolling

touchpad scrolling

keyboard scrolling

Scrollbars are theme-aware.

---

# Toolbar

Toolbar appears beneath title area.

Contains:

Buttons

Search fields

Menus

Spacers

Toolbar contents are application-defined.

Toolbar appearance is theme-defined.

---

# SplitView

Supports resizable panes.

Examples:

Finder

Settings

Mail

Capabilities:

Horizontal split

Vertical split

Persistent sizes

---

# Dialogs

Dialog types:

Alert

Confirmation

Input

Progress

Dialogs follow platform appearance.

Applications may not create custom dialog chrome.

---

# Layout System

Supported layouts:

Horizontal

Vertical

Grid

Stack

Overlay

Layouts are constraint-based.

Widgets should not use absolute positioning except where necessary.

---

# Event System

Event types:

MouseMove

MouseDown

MouseUp

KeyDown

KeyUp

Focus

Blur

DragStart

DragEnd

Drop

Events propagate through widget tree.

---

# Focus System

Only one widget may own focus.

Focus traversal:

Tab

Shift+Tab

Arrow keys where appropriate

All focus behavior must be predictable.

---

# Drag and Drop

Supported system-wide.

Data types:

Files

Text

Images

Custom objects

Finder behavior defines reference implementation.

---

# Clipboard

System-wide clipboard.

Supported formats:

Text

Rich Text

Images

Files

Applications interact through platform APIs.

---

# Accessibility

Every widget must provide:

Role

Label

Description

State

Focus information

Accessibility support is mandatory.

No widget may ship without accessibility metadata.

---

# Theme System

Widgets consume tokens.

Example:

Button.Background

Button.Text

Button.Highlight

Window.Background

Window.Border

Menu.Background

Menu.Selection

Widgets never reference colors directly.

---

# Theme Variants

Required support:

Platinum

Graphite

OLED Graphite

High Contrast

Every widget must render correctly under all themes.

---

# Animation System

Animations are optional.

Animations must:

Never block input.

Never exceed platform animation durations.

Animations must degrade gracefully.

---

# Text System

Text rendering provided by:

cosmic-text

Features:

Unicode

Emoji

RTL

Complex scripts

Font fallback

Subpixel rendering

Text must remain crisp at all scaling factors.

---

# Scaling

RetroKit supports:

100%

125%

150%

175%

200%

300%

All widgets must scale correctly.

Hardcoded pixel assumptions are prohibited.

---

# Performance Targets

Window creation:

< 16ms

Widget layout:

< 1ms typical

Input latency:

< 8ms

Scrolling:

60 FPS minimum

120 FPS preferred

Rendering performance is a platform feature.

---

# Public API Philosophy

The API should resemble:

Cocoa

AppKit

Qt

More than:

HTML

CSS

JavaScript

Example:

Window
Button
Toolbar
SplitView
ListView

Explicit object-oriented structures are preferred.

---

# Testing Requirements

Every widget requires:

Unit tests

Accessibility tests

Keyboard navigation tests

Visual regression tests

Performance benchmarks

A widget without tests is incomplete.

---

# Future Compatibility

RetroKit APIs are expected to remain stable.

Breaking changes require:

New RFC

Migration plan

Compatibility analysis

Backward compatibility is preferred.

---

# Definition of Success

A developer should be able to build:

Finder

Settings

Terminal

TextEdit

using only RetroKit.

If applications routinely need custom controls outside RetroKit, then RetroKit has failed its design goals.
