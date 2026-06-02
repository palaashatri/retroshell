# RFC-0002-RetroShell.md

Status: Accepted
Version: 1.0

## Abstract

RetroShell is the desktop runtime. It owns desktop management, window management, global menu bar, Dock, notifications, session management, workspaces, and app launching.

## Core Components

MenuServer, WindowManager, DesktopManager, Dock, NotificationCenter, WorkspaceManager, LaunchServices, SessionManager, ThemeManager, ApplicationRegistry.

## Ownership

RetroShell owns desktop state, window state, menu state, dock state, notifications, session state. Applications own documents, menus, and app logic. RetroKit owns widgets and accessibility.

## MenuServer

Exactly one menu bar exists, at the top of the primary display. RetroShell renders menus defined by applications. The menu bar never moves.

## WindowManager

Owns focus, ordering, placement, workspace assignment, fullscreen transitions, and window decorations. Applications own content only.

## DesktopManager

Controls desktop background, icons, mounted volumes, and selections. Finder renders the desktop contents.

## Dock

Provides app launching, running indicators, window switching, trash, and recent apps. Visible by default.

## WorkspaceManager

Supports multiple workspaces. Default is 4.

## NotificationCenter

Applications emit notifications; RetroShell renders them.

## LaunchServices

App discovery, bundle registration, file associations, and app launching. Scan /Applications and /User/Applications.

## SessionManager

Login, logout, lock, shutdown, restart, restore.

## ThemeManager

Loads and persists themes. Supported: Platinum, Graphite, OLED Graphite, High Contrast.

## Accessibility

Keyboard navigation, screen readers, focus traversal, high contrast, large text are mandatory.

## Performance Targets

Cold startup <500ms. Menu switching <16ms. Workspace switch <100ms.

## Security

Applications cannot control RetroShell or other apps. Privileged actions go through approved APIs.

## Success

Users should perceive RetroShell as a complete operating environment, not a themed Linux desktop.
