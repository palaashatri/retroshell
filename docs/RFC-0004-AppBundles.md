# RFC-0004-AppBundles.md

Status: Accepted

Version: 1.0

Authors: RetroShell Architecture Team

---

# Abstract

Retro applications are distributed as self-contained application bundles.

Applications are represented as:

Application.app

An application bundle appears as a single object to users.

Internally, the bundle contains:

* Executable code
* Resources
* Assets
* Metadata
* Localization data
* Optional embedded frameworks

This model is inspired by:

* Classic Mac OS
* NeXTSTEP
* macOS

The bundle model is the primary software distribution mechanism for the Retro platform.

---

# Goals

Provide:

* Simple installation
* Simple removal
* Portable applications
* Predictable application structure
* User-friendly software distribution
* Reduced dependency management complexity

---

# Non-Goals

This RFC does not define:

* Package repositories
* Package managers
* Linux package formats
* System package installation

Those are intentionally excluded from the primary user experience.

---

# Design Principles

## Principle 1

Applications are objects.

Users should think:

"I have an application."

Not:

"I installed a package."

---

## Principle 2

Applications are self-contained.

Where practical, applications should carry their own resources.

---

## Principle 3

Installation should be obvious.

Preferred installation:

Drag application to Applications folder.

Done.

---

## Principle 4

Removal should be obvious.

Preferred removal:

Move application to Trash.

Done.

---

## Principle 5

Application internals should be hidden.

Users interact with applications.

Not implementation details.

---

# Bundle Extension

Applications use:

.app

Examples:

Finder.app

TextEdit.app

Terminal.app

Music.app

ImageViewer.app

---

# User View

Applications appear as:

Single icon

Single name

Single object

Users do not see internal bundle contents by default.

---

# Bundle Structure

Example:

TextEdit.app/

```
App.toml

Executable/

    TextEdit

Resources/

Assets/

Localization/

Frameworks/

Plugins/

Updates/
```

Directory layout may evolve.

Required top-level components are defined below.

---

# Required Components

Every bundle must contain:

App.toml

Executable/

Resources/

Optional components may be omitted.

---

# App.toml

App.toml defines application metadata.

Example:

name = "TextEdit"

bundle_id = "com.retro.textedit"

version = "1.0.0"

author = "Retro Project"

minimum_platform = "1.0"

entrypoint = "Executable/TextEdit"

---

# Bundle Identifier

Every application must have a globally unique identifier.

Examples:

com.retro.finder

com.retro.textedit

com.retro.terminal

Identifiers must never change after public release.

---

# Versioning

Semantic versioning is required.

Example:

1.0.0

1.1.0

2.0.0

Version format:

MAJOR.MINOR.PATCH

---

# Executable Directory

Contains application binaries.

Example:

Executable/

```
TextEdit
```

Users should never interact with this directory directly.

---

# Resources Directory

Contains:

Icons

UI definitions

Themes

Fonts

Translations

Images

Audio assets

Resources are bundled with the application.

---

# Assets Directory

Contains:

Graphics

Animations

Media

Documentation

Application-specific assets

---

# Localization Directory

Contains language resources.

Example:

en_US

en_GB

fr_FR

de_DE

ja_JP

Localization is encouraged.

---

# Frameworks Directory

Optional.

Contains application-specific frameworks.

Example:

Frameworks/

```
Markdown.framework

Charts.framework
```

Applications may bundle dependencies.

---

# Shared Framework Model

System frameworks reside in:

/System/Frameworks

Examples:

RetroKit.framework

MediaKit.framework

AudioKit.framework

NetworkKit.framework

Applications should prefer system frameworks.

---

# Framework Loading

Load order:

1. Application Frameworks
2. System Frameworks

Application framework versions take precedence.

This enables application portability.

---

# Plugins Directory

Optional.

Contains:

Application extensions

Importers

Exporters

Viewers

Plugin loading must be sandboxed.

---

# Application Installation

Preferred workflow:

Download Application.app

Drag into:

/Applications

Application becomes available.

No package manager required.

---

# Application Removal

Preferred workflow:

Move Application.app to Trash

Application removed.

No uninstallers required.

No registry cleanup required.

---

# Application Discovery

LaunchServices scans:

/Applications

/User/Applications

Registered bundles become launchable applications.

---

# Application Updates

Applications may support self-update.

Workflow:

Check for update

Download replacement bundle

Replace application

Update process must preserve:

User data

Preferences

Documents

---

# System Updates

System updates are separate from application updates.

Applications are not updated through operating system updates.

System frameworks may be updated independently.

---

# Portable Applications

Applications should remain portable.

Example:

USB Drive

```
PixelPaint.app
```

User copies bundle.

Application launches.

Installation is optional.

---

# User Data Separation

Applications must not store user data inside bundles.

User data belongs in:

User/

```
Library/

    Application Support/

    Preferences/

    Cache/
```

This enables safe updates and removal.

---

# Preferences

Applications store settings through platform APIs.

Applications should not write arbitrary configuration files.

Preferred storage:

Preferences Service

---

# Application Permissions

Permissions are declared in:

App.toml

Example:

permissions = [

```
"documents",

"network",

"camera"
```

]

Permissions are descriptive.

Future RetroOS versions may enforce them.

---

# File Associations

Declared in App.toml.

Example:

file_types = [

```
"txt",

"md",

"rtf"
```

]

LaunchServices uses these declarations.

---

# Document Types

Applications may define custom document types.

Example:

retrodoc

retroproject

retroimage

Document ownership should be explicit.

---

# Icons

Every application requires:

16x16

32x32

64x64

128x128

256x256

512x512

1024x1024

Icons must support:

Platinum

Graphite

OLED Graphite

where appropriate.

---

# Security

Applications may not:

Modify other application bundles

Modify system bundles

Modify RetroShell

Application isolation is required.

---

# Bundle Inspection

Advanced users may inspect bundles.

Context Menu:

Show Package Contents

This capability is intended for developers and advanced users.

---

# Finder Integration

Finder treats bundles as applications.

Default actions:

Double-click → Launch

Rename → Rename bundle

Move → Move bundle

Copy → Copy bundle

Internal structure remains hidden.

---

# Backup Behavior

Bundles should be self-contained.

Copying a bundle should preserve:

Executable

Resources

Metadata

Frameworks

Applications should remain functional after copying.

---

# RetroOS Compatibility

Bundle format must remain platform-independent.

Bundle structure must not depend on:

Linux

Wayland

D-Bus

POSIX-specific paths

This enables future migration to RetroOS.

---

# Performance Targets

Bundle discovery:

< 100ms per application

Launch preparation:

< 50ms

Metadata loading:

Instantaneous for common operations

---

# Future Extensions

Potential future features:

Application signing

Application notarization

Bundle compression

Delta updates

Enterprise deployment

These extensions must not break existing bundles.

---

# Definition of Success

A user should be able to:

Download an application

Drag it into Applications

Launch it

Update it

Remove it

without ever interacting with:

* package managers
* repositories
* dependency installers
* system registries
* command-line tools

Applications should feel like self-contained objects that users can understand, move, copy, back up, and manage directly.
