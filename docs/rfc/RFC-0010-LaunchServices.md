# RFC-0010-LaunchServices.md

Status: Accepted
Version: 1.0

## Abstract

LaunchServices owns application discovery, bundle registration, application launching, file associations, and default applications.

## Search Locations

/Applications and /User/Applications.

## Bundle Registration

Reads App.toml and stores bundle ID, version, capabilities, and file types.

## Launch Flow

User action → LaunchServices → application process → window registration.

## File Associations

txt → TextEdit, png → ImageViewer, zip → Archive Utility.

## Open With

Supported, user-overridable.
