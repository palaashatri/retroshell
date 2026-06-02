# RFC-0005-ThemeSystem.md

Status: Accepted

Version: 1.0

Authors: RetroShell Architecture Team

---

# Abstract

The Retro Theme System defines the visual identity of the Retro platform.

Themes control:

* Colors
* Typography
* Icons
* Window appearance
* Menus
* Controls
* Shadows
* Spacing
* Metrics

Themes do not control behavior.

Behavior is defined by RetroKit and RetroShell.

The Theme System ensures that all applications appear as part of a unified platform.

---

# Design Vision

RetroShell should feel like:

* Classic Mac OS evolved into the modern era
* NeXTSTEP with modern graphics capabilities
* BeOS with stronger visual consistency

RetroShell should not resemble:

* GNOME
* KDE Plasma
* Material Design
* Fluent Design
* Mobile operating systems
* Browser-based applications

---

# Core Design Principles

## Principle 1

Function Before Decoration

Visual elements exist to communicate information.

Decoration must never obscure usability.

---

## Principle 2

Strong Window Identity

Users must immediately understand:

* active windows
* inactive windows
* focused controls

---

## Principle 3

Visual Density

Information density is encouraged.

Excessive whitespace is discouraged.

RetroShell is designed for productivity.

---

## Principle 4

Permanent Discoverability

Actions should be visible.

Users should not need to memorize gestures.

---

## Principle 5

Theme Independence

Widgets consume theme tokens.

Widgets never hardcode colors.

Widgets never hardcode metrics.

---

# Supported Themes

Mandatory themes:

1. Platinum
2. Graphite
3. OLED Graphite
4. High Contrast

All platform components must support all themes.

---

# Platinum Theme

Primary theme.

Inspired by:

* Mac OS 8
* Mac OS 9

Characteristics:

* Bright surfaces
* Subtle gradients
* Beveled controls
* Defined borders
* Distinct shadows

Purpose:

Default desktop experience.

---

# Graphite Theme

Dark variant of Platinum.

Characteristics:

* Reduced brightness
* Strong contrast
* Metallic appearance
* Neutral color palette

Purpose:

Low-light productivity.

---

# OLED Graphite Theme

Modern dark theme.

Characteristics:

* Near-black surfaces
* HDR-aware highlights
* Reduced power usage on OLED displays
* Improved contrast

Purpose:

Modern hardware support.

---

# High Contrast Theme

Accessibility-focused.

Characteristics:

* Maximum contrast
* Simplified visuals
* Clear boundaries
* Strong focus indicators

Purpose:

Accessibility compliance.

---

# Theme Architecture

Theme
↓
Theme Tokens
↓
RetroKit
↓
Widgets

Widgets never directly access colors.

Widgets request tokens.

---

# Theme Package Structure

Theme.bundle/

```
Theme.toml

Colors.toml

Metrics.toml

Typography.toml

Icons/

Assets/
```

Theme packages are loaded dynamically.

---

# Theme Metadata

Theme.toml

Contains:

name

identifier

author

version

supported_platform_version

Example:

name = "Platinum"

identifier = "com.retro.theme.platinum"

version = "1.0"

---

# Theme Tokens

All visual values are represented by tokens.

Example:

Window.Background

Window.Border

Window.Title

Menu.Background

Menu.Highlight

Button.Background

Button.Highlight

Button.Shadow

Text.Primary

Text.Secondary

Selection.Background

Selection.Text

---

# Color System

Colors are defined centrally.

Widgets must never define colors.

Theme colors are categorized:

Surface

Content

Accent

Selection

Status

Focus

Border

Shadow

---

# Surface Colors

Examples:

Desktop background

Window background

Panel background

Toolbar background

Menu background

Surface colors define structure.

---

# Content Colors

Examples:

Primary text

Secondary text

Disabled text

Links

Labels

Content colors define readability.

---

# Accent Colors

Used sparingly.

Examples:

Selection

Highlights

Progress indicators

Focus rings

Accent colors must not dominate the interface.

---

# Window Appearance

Windows define platform identity.

Required components:

Border

Title Bar

Content Region

Shadow

Resize Regions

Window appearance must remain consistent across applications.

---

# Active Windows

Characteristics:

Higher contrast

Visible title emphasis

Focused controls

Clearly visible shadow

Users must always identify active windows.

---

# Inactive Windows

Characteristics:

Reduced contrast

Reduced emphasis

Readable content

Inactive windows remain usable and understandable.

---

# Window Metrics

Default values:

Title Bar Height

24px

Border Width

1px

Shadow Radius

4px

Resize Region

8px

Metrics are theme-defined.

Widgets consume metrics dynamically.

---

# Menus

Menus are platform-controlled.

Characteristics:

Compact

Readable

Keyboard-friendly

Fast navigation

Menus must remain visually stable.

---

# Menu Bar

Single global menu bar.

Characteristics:

Persistent

Always visible

Minimal height

High readability

The menu bar is a defining platform feature.

---

# Buttons

Button states:

Normal

Hovered

Pressed

Focused

Disabled

Buttons must clearly communicate state.

---

# Button Style

Characteristics:

Beveled edges

Subtle depth

Visible focus

Distinct hover feedback

Flat design is discouraged.

---

# Text Fields

Characteristics:

Defined borders

Visible focus

Clear selection

Readable typography

Text fields should prioritize clarity over decoration.

---

# Lists and Tables

Characteristics:

Compact rows

Alternating row support

Strong selection visibility

Keyboard navigation support

Information density is encouraged.

---

# Icons

Icons are a primary navigation mechanism.

Required sizes:

16x16

32x32

64x64

128x128

256x256

512x512

1024x1024

Icons must remain recognizable at all sizes.

---

# Icon Philosophy

Inspired by:

Classic Macintosh

NeXTSTEP

BeOS

Characteristics:

Readable

Distinct

Literal

Minimal abstraction

Users should recognize objects immediately.

---

# Typography

Typography is platform-wide.

System fonts must prioritize:

Readability

Internationalization

Accessibility

Performance

Typography tokens include:

Heading

Title

Body

Caption

Monospace

---

# Font Scaling

Supported scaling:

100%

125%

150%

175%

200%

300%

Text must remain crisp.

---

# Focus Indicators

Every focusable element must display focus.

Focus indicators must:

Be visible

Be theme-aware

Pass accessibility requirements

Focus indicators are mandatory.

---

# Selection Appearance

Selections must be obvious.

Selected items require:

Background change

Text contrast adjustment

Focus visibility

Selection should never be ambiguous.

---

# Shadows

Shadows communicate hierarchy.

Shadows should be:

Subtle

Consistent

Theme-aware

Shadows must not become decorative effects.

---

# Animations

Animations are secondary.

Requirements:

Fast

Predictable

Interruptible

Animations must never block interaction.

---

# Animation Durations

Recommended:

Hover

100ms

Menu

150ms

Window

200ms

Workspace

250ms

Long animations are discouraged.

---

# HDR Support

Themes must support HDR-capable displays.

Capabilities:

HDR wallpapers

Wide color gamut

Enhanced highlights

Improved gradients

HDR must enhance the experience without changing behavior.

---

# Accessibility Requirements

Themes must support:

Screen readers

Large text

High contrast

Reduced motion

Keyboard navigation

Accessibility is mandatory.

---

# Custom Themes

Future versions may support third-party themes.

Custom themes must:

Use theme tokens

Respect accessibility

Pass validation

Custom themes must not break applications.

---

# Validation Rules

Theme validation verifies:

Required tokens exist

Required metrics exist

Accessibility compliance

Icon completeness

Typography completeness

Invalid themes are rejected.

---

# Performance Requirements

Theme switching:

< 100ms

Widget restyling:

< 16ms

Window refresh:

60 FPS minimum

Theme performance is part of platform quality.

---

# RetroOS Compatibility

Theme definitions must remain platform-independent.

Theme files must not depend on:

Linux

Wayland

D-Bus

Platform-specific APIs

Themes should function identically on RetroOS.

---

# Definition of Success

A screenshot of RetroShell should be immediately recognizable without seeing the logo.

Users should be able to identify:

* RetroShell windows
* RetroShell menus
* RetroShell controls
* RetroShell applications

at a glance.

The theme system succeeds when the platform has a distinct visual identity rather than appearing as a collection of unrelated applications.
