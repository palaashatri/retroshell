# RFC-0001-RetroKit.md

Status: Accepted
Version: 1.0

## Abstract

RetroKit is the native UI framework for RetroShell. It provides windows, widgets, layout, events, accessibility, theme integration, and rendering integration. It is the only supported framework for first-party applications.

## Goals

1. Visual consistency.
2. Predictable behavior.
3. High performance.
4. Accessibility.
5. Theme support.
6. Long-term stability.

## Non-Goals

RetroKit is not a web framework, browser runtime, JS runtime, scene graph editor, or game engine. It does not provide networking, audio, filesystem, or process management.

## Architecture

Applications → RetroKit → RetroRender → Wayland

RetroKit never talks directly to GPU APIs.

## Core Concepts

All visible elements derive from Widget.

Pseudo-interface:

trait Widget {
    fn layout();
    fn draw();
    fn handle_event();
    fn accessibility();
}

Widgets form a tree. Widgets never own windows. Applications build the tree.

## Window

Window is the root user-visible container. It has a title, toolbar, content area, resize support, and focus handling. RetroShell decorates windows.

## Menu System

Menus are declarative. Applications define menu structure; RetroShell renders the menu bar. Applications do not render menu bars.

## Widgets

Provide: Button, ToggleButton, RadioButton, ToolbarButton, TextField, ListView, TreeView, IconView, ScrollView, Toolbar, SplitView, Dialogs.

## Layout

Supported layouts: Horizontal, Vertical, Grid, Stack, Overlay. Absolute positioning is discouraged.

## Event and Focus

Events: mouse, keyboard, focus, drag/drop. Exactly one widget owns focus. Keyboard navigation is first-class.

## Accessibility

Every widget provides role, label, description, state, and focus information. Mandatory.

## Theme System

Widgets consume tokens, never raw colors. Required themes: Platinum, Graphite, OLED Graphite, High Contrast.

## Text System

Use cosmic-text. Unicode, emoji, RTL, complex scripts, font fallback, subpixel rendering.

## Scaling

Support 100% through 300%. Hardcoded pixel assumptions are prohibited.

## Performance Targets

Window creation <16ms. Typical layout <1ms. Input latency <8ms. 60 FPS minimum.

## Testing

Unit tests, accessibility tests, keyboard tests, visual regression tests, performance benchmarks required.

## Success

A developer should be able to build Finder, Settings, Terminal, and TextEdit with only RetroKit.
