# RFC-0004-AppBundles.md

Status: Accepted
Version: 1.0

## Abstract

Retro applications are distributed as self-contained application bundles with `.app` extension. Users experience each app as a single object.

## Goals

Simple installation, simple removal, portable applications, predictable structure, user-friendly distribution, reduced dependency complexity.

## Non-Goals

No package repositories, package managers, Linux package formats, or system package installation as the primary user experience.

## Bundle Extension

`.app`

## Bundle Structure

Application.app/
    App.toml
    Executable/
    Resources/
    Assets/
    Localization/
    Frameworks/
    Plugins/
    Updates/

Required: App.toml, Executable/, Resources/

## App.toml

Contains name, bundle_id, version, author, minimum_platform, entrypoint, permissions, supported file types, and menu declarations.

## Bundle Identifier

Must be globally unique and stable after release.

## Versioning

Semantic versioning: MAJOR.MINOR.PATCH

## Frameworks

System frameworks live in /System/Frameworks. Applications may bundle their own frameworks. Load order: app frameworks then system frameworks.

## Installation and Removal

Preferred: drag to Applications. Removal: move to Trash.

## User Data

User data does not belong inside bundles. Store preferences and app support in user library locations.

## Security

Applications may not modify other bundles or system bundles.

## Finder Integration

Bundles are launched by double-clicking and treated as applications, not directories.

## Backup and Portability

Copying a bundle should preserve functionality.

## Success

Users should install, launch, update, and remove apps without package managers or registries.
