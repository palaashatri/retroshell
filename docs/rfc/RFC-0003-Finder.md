# RFC-0003-Finder.md

Status: Accepted
Version: 1.0

## Abstract

Finder is the primary file management application. It owns the desktop, file browsing, volume browsing, application launching, drag-and-drop, and trash management.

## Goals

Simple file management, consistent desktop behavior, application launching, volume management, discoverability.

## Responsibilities

Finder owns the desktop, desktop icons, file browsing, volume browsing, drag-and-drop file operations, and trash. It does not own window management, menu rendering, dock behavior, theme management, or notifications.

## Desktop Behavior

Single click selects, double click opens, drag moves, right click opens context menu. Desktop contents are real filesystem objects.

## Views

Icon View, List View, Column View, Gallery View.

## Windows

A Finder window represents a location: Documents, Downloads, Applications, Volumes, Desktop.

## Sidebar

Favorites, Applications, Desktop, Documents, Downloads, Volumes, Network Locations, Recent Items.

## File Operations

Copy, move, rename, delete, duplicate, compress, create folder, create alias, reveal.

## Drag and Drop

File to folder, file to app, file to desktop, file to trash, volume to desktop, app to dock.

## Application Launching

LaunchServices handles app launching. Bundles appear as single objects.

## Volumes

Volumes are first-class citizens and appear on desktop, sidebar, and in Finder windows.

## Trash

Deleted files move to Trash and can be restored or permanently deleted.

## Search

Search by name, type, date, size, tags, and contents later.

## Tags and Aliases

Tags are filesystem metadata. Aliases preserve user intent after file movement.

## Accessibility

Keyboard and screen reader support mandatory.

## Performance

Finder launch <200ms, directory open <100ms, search response <100ms.

## Success

Users should not need to open a terminal to use the desktop.
