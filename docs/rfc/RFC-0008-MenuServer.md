# RFC-0008-MenuServer.md

Status: Accepted
Version: 1.0

## Abstract

MenuServer implements the global menu bar. Only one menu bar exists and it is always at the top of the primary display.

## Ownership

Applications define menu content. RetroShell renders menus.

## Focus Switching

When focus changes, MenuServer updates the active menus.

## Keyboard Shortcuts

MenuServer owns standard shortcuts (Cmd+Q, Cmd+W, Cmd+N, Cmd+S) and routes application-declared shortcuts.

## Status Area

Clock, network, audio, battery, and notifications live on the right side and are managed by MenuServer.
