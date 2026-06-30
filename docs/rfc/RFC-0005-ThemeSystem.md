# RFC-0005-ThemeSystem.md

Status: Accepted
Version: 1.0

## Abstract

The theme system defines the visual identity of RetroShell. Themes control colors, typography, icons, metrics, shadows, spacing, and appearance. Themes do not control behavior.

## Design Vision

RetroShell should feel like Classic Mac OS evolved for modern hardware, with NeXTSTEP and BeOS influences. Avoid Material, Fluent, GNOME, KDE, and mobile aesthetics.

## Core Principles

Function before decoration. Strong window identity. Visual density. Permanent discoverability. Theme independence.

## Supported Themes

Mandatory: Platinum, Graphite, OLED Graphite, High Contrast.

## Theme Architecture

Theme → Tokens → RetroKit → Widgets

Widgets never consume raw colors.

## Theme Package Structure

Theme.bundle/
    Theme.toml
    Colors.toml
    Metrics.toml
    Typography.toml
    Icons/
    Assets/

## Tokens

Window.Background, Window.Border, Window.Title, Menu.Background, Menu.Highlight, Button.Background, Button.Highlight, Button.Shadow, Text.Primary, Text.Secondary, Selection.Background, Selection.Text.

## Window Appearance

Distinct active/inactive states, visible borders, title bar, shadow, resize regions.

## Buttons, Menus, Text Fields, Lists

Compact, readable, keyboard-friendly, strong focus indicators.

## Icons

Readable, distinct, literal. Must remain recognizable at 16x16 through 1024x1024.

## Typography

System-wide fonts must prioritize readability, internationalization, accessibility, and performance.

## HDR and Accessibility

Themes must support HDR displays and accessibility features, including high contrast and reduced motion.

## Validation

Themes must be validated for required tokens, metrics, icons, and accessibility compliance.

## Success

A screenshot should be recognizable as RetroShell without a logo.
