# RFC-0006-RetroBus.md

Status: Accepted
Version: 1.0

## Abstract

RetroBus is the system communication layer. It provides service discovery, message routing, event delivery, notifications, window coordination, and menu synchronization. Initial transport may be D-Bus; future transport is native.

## Goals

Applications should never directly manipulate shell state or other applications. All communication occurs through RetroBus APIs.

## Architecture

Application → RetroBus → RetroShell Services

## Message Types

Commands: LaunchApplication, OpenDocument, ShowPreferences.
Events: WindowFocused, ThemeChanged, VolumeMounted.
Queries: GetRunningApplications, GetTheme, GetWorkspaceState.

## Service Registry

Services register as com.retro.finder, com.retro.launchservices, com.retro.settings, com.retro.windowmanager.

## Design Rule

Transport, platform, and language independent.
