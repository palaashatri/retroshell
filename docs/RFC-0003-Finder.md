# RFC-0003-Finder.md

Status: Accepted

Version: 1.0

Authors: RetroShell Architecture Team

---

# Abstract

Finder is the primary file management application of RetroShell.

Finder is more than a file browser.

Finder owns:

* Desktop icons
* File browsing
* Volume browsing
* Application launching
* Drag-and-drop operations
* Trash management

Finder defines many core user interactions of the Retro platform.

Finder serves a role similar to:

* Macintosh Finder
* NeXT Workspace Manager
* Windows Explorer

Finder is considered a core platform component.

---

# Goals

Provide:

* Simple file management
* Consistent desktop behavior
* Application launching
* Volume management
* Discoverable user experience

---

# Non-Goals

Finder is not:

* A terminal replacement
* A package manager
* A cloud storage service
* A developer tool

---

# Design Philosophy

Users should rarely need to know:

* filesystem internals
* mount points
* device paths

Users interact with:

* files
* folders
* applications
* volumes

Finder abstracts system complexity.

---

# Finder Responsibilities

Finder owns:

Desktop

Desktop icons

File browsing

Volume browsing

Application launching

Trash

File operations

Finder does not own:

Window management

Menu bar rendering

Dock behavior

Theme management

Notifications

---

# Desktop Ownership

Finder renders the desktop.

Desktop is a first-class workspace.

Desktop contents include:

Files

Folders

Applications

Volumes

Aliases

Desktop icons are real filesystem objects.

Desktop is not a virtual container.

---

# Desktop Behavior

Single Click

Select

Double Click

Open

Drag

Move

Right Click

Context Menu

Drag Rectangle

Multi-select

Behavior must remain consistent everywhere.

---

# Finder Window Model

Every Finder window represents a location.

Example:

Documents

Downloads

Applications

Volumes

Desktop

A Finder window should always answer:

"What location am I viewing?"

---

# Window Layout

Default structure:

Toolbar

Sidebar

Content Area

Status Area

View Selector

The layout remains compact and information-dense.

---

# Sidebar

Sidebar displays:

Favorites

Applications

Desktop

Documents

Downloads

Volumes

Network Locations

Recent Items

Users may customize ordering.

---

# View Modes

Finder supports:

Icon View

List View

Column View

Gallery View

All views operate on the same underlying data model.

---

# Icon View

Primary view mode.

Characteristics:

Large icons

Labels beneath icons

Grid layout

Drag-and-drop support

Multi-selection

Designed for discoverability.

---

# List View

Characteristics:

Rows

Columns

Sorting

Grouping

Keyboard navigation

Suitable for large directories.

---

# Column View

Inspired by NeXTSTEP.

Displays hierarchy through columns.

Advantages:

Fast navigation

Minimal window clutter

Excellent keyboard support

Preferred for advanced users.

---

# Gallery View

Displays:

Preview

Metadata

File information

Used primarily for:

Images

Videos

Documents

Media assets

---

# Selection Model

Supported:

Single Selection

Multi Selection

Range Selection

Keyboard Selection

Selection behavior must match throughout Finder.

---

# File Operations

Supported operations:

Copy

Move

Rename

Delete

Duplicate

Compress

Create Folder

Create Alias

Reveal

Operations should be discoverable.

No hidden workflows.

---

# Drag and Drop

Drag-and-drop is a primary interaction model.

Supported:

File to Folder

File to Application

File to Desktop

File to Trash

Volume to Desktop

Application to Dock

Behavior should feel direct and predictable.

---

# Application Launching

Applications are bundles.

Example:

TextEdit.app

ImageViewer.app

Finder.app

Users launch applications by:

Double-clicking

Dock

Application menu

Open With

Finder delegates launching to LaunchServices.

---

# Bundle Representation

Applications appear as single objects.

Users should not see:

Executables

Internal resources

Bundle metadata

Bundles appear as applications.

---

# Bundle Structure

Example:

TextEdit.app

```
App.toml

Executable/

Resources/

Assets/

Localization/
```

Finder treats the bundle as one object.

---

# Package Inspection

Advanced users may inspect bundles.

Context menu:

Show Package Contents

Default behavior:

Treat bundle as application.

---

# Volumes

Volumes are first-class citizens.

Examples:

Internal Disk

USB Drive

Network Share

External SSD

Volumes appear:

Desktop

Sidebar

Finder windows

---

# Volume Operations

Mount

Unmount

Rename

Inspect

Eject

Volumes should always have visible state.

---

# Trash

Trash is managed through Finder.

Deleted files move to Trash.

Files remain recoverable.

Supported operations:

Restore

Delete Permanently

Empty Trash

Trash is represented in Dock.

---

# Search

Finder includes integrated search.

Search capabilities:

Name

Type

Date

Size

Tags

Contents (future)

Search should feel instantaneous.

---

# Tags

Finder supports tags.

Tags provide lightweight organization.

Examples:

Work

Personal

Archive

Important

Tags may have colors.

Tags are filesystem metadata.

---

# Aliases

Finder supports aliases.

Alias behavior:

Reference target

Survive file movement

Maintain user intent

Aliases are preferred over traditional symbolic links for user workflows.

---

# Context Menus

Every object supports context menus.

Examples:

Open

Open With

Rename

Duplicate

Move To Trash

Get Info

Menus must remain concise.

---

# Get Info Window

Every object supports information inspection.

Displays:

Name

Type

Location

Size

Permissions

Dates

Tags

Associated Application

Appearance should remain platform consistent.

---

# Quick Preview

Users can preview files without launching applications.

Capabilities:

Images

Documents

Audio

Video

Text

Preview must be fast.

Preview should not require full application startup.

---

# Keyboard Navigation

Required:

Arrow Keys

Enter

Delete

Tab

Shift Selection

Search

All functionality must remain accessible without a mouse.

---

# Accessibility

Required:

Screen Reader Support

Keyboard Navigation

Large Text Support

High Contrast Support

Accessible Labels

Accessibility support is mandatory.

---

# Finder Menus

Required menus:

Finder

File

Edit

View

Go

Window

Help

Menu structure should remain stable.

---

# Desktop Icons

Default icons:

Applications

Folders

Documents

Volumes

Trash

Icons must remain recognizable at:

16px

32px

64px

128px

256px

512px

---

# File Associations

Finder displays default applications.

Example:

txt
→ TextEdit

png
→ ImageViewer

jpg
→ ImageViewer

zip
→ Archive Utility

Users may override defaults.

---

# Performance Targets

Finder launch:

< 200ms

Directory open:

< 100ms

Search response:

< 100ms

Desktop redraw:

60 FPS minimum

120 FPS preferred

---

# Future RetroOS Compatibility

Finder must not depend on:

Linux paths

Linux mount semantics

Linux-specific APIs

Platform abstraction layers are required.

---

# Definition of Success

A user should be able to use RetroShell for an entire day without opening a terminal.

Users should think in terms of:

Applications

Documents

Folders

Volumes

Not:

Processes

Mount points

Executables

Configuration files

Finder succeeds when it becomes the natural center of the Retro desktop experience, providing a direct, approachable, and consistent way to interact with the entire system.
