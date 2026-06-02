# RFC-0007-WindowManager.md

Status: Accepted
Version: 1.0

## Abstract

WindowManager owns window lifecycle, focus, placement, workspaces, fullscreen, and decorations. Applications own content only.

## Responsibilities

Applications create and close windows; WindowManager moves, focuses, and assigns workspaces.

## Window States

Normal, Minimized, Maximized, Fullscreen, Hidden, Destroyed.

## Focus Model

Exactly one application, one window, and one widget may have focus in the hierarchy.

## Decorations

RetroShell draws title bar, border, shadow, and resize regions.

## Workspace Model

Default 4 workspaces; windows belong to one workspace but apps may span multiple.

## Future Goal

Compositor-integrated window management.
