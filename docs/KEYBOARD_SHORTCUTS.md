# RetroShell Keyboard Shortcuts

This document covers all keyboard shortcuts across the shell and first-party applications.
On Linux the Meta key is the Super/Windows key. On Mac it is the Command key.

---

## Shell (Global)

These shortcuts are active whenever the RetroShell desktop has focus.

| Shortcut              | Action                                        |
|-----------------------|-----------------------------------------------|
| Cmd+N                 | New Finder window                             |
| Cmd+O                 | Open Finder application                       |
| Cmd+W                 | Close front window                            |
| Cmd+S                 | Save (document-aware apps only)               |
| Cmd+P                 | Print (not yet connected to print service)    |
| Cmd+Z                 | Undo                                          |
| Cmd+Shift+Z           | Redo                                          |
| Cmd+X                 | Cut                                           |
| Cmd+C                 | Copy                                          |
| Cmd+V                 | Paste                                         |
| Cmd+A                 | Select All                                    |
| Cmd+F                 | Enter/Exit Fullscreen                         |
| Cmd+Tab               | Cycle focus through open windows (same workspace) |
| Cmd+Q                 | Quit RetroShell                               |
| Cmd+Shift+Q           | Log Out                                       |
| Cmd+L (Ctrl+Cmd+L)    | Lock Screen                                   |
| Cmd+Alt+Escape        | Force Quit dialog                             |

---

## Window Management

| Shortcut              | Action                                        |
|-----------------------|-----------------------------------------------|
| Click titlebar        | Bring window to front                         |
| Drag titlebar         | Move window                                   |
| Drag resize handle    | Resize window (bottom-right corner)           |
| Click close box       | Close window (left titlebar button)           |
| Click minimize box    | Minimize to titlebar tab                      |
| Click zoom box        | Zoom/unzoom window (right titlebar button)    |

---

## Workspace Switching

| Shortcut                  | Action                                    |
|---------------------------|-------------------------------------------|
| Window > Previous Desktop | Switch to previous workspace              |
| Window > Next Desktop     | Switch to next workspace                  |
| Window > Desktop 1..8     | Switch directly to workspace 1..8 (Ctrl+Alt+1..8) |
| Meta+] / Meta+[           | Next / previous workspace                 |
| Meta+L                    | Lock Screen                               |
| Meta+Shift+Q              | Log Out                                   |

Workspace shortcuts are accessible through the Window menu in both the shell menu bar
and in any active first-party SDK application menu.

---

## Finder (File Manager)

Finder runs as a separate process. These shortcuts apply when Finder has focus.

| Shortcut              | Action                                        |
|-----------------------|-----------------------------------------------|
| Cmd+N                 | New Folder in current directory               |
| Cmd+I                 | Get Info (selected item or current folder)    |
| Cmd+Delete            | Move to Trash                                 |
| Double-click folder   | Open folder in new window                     |
| Drag icon to folder   | Move file/folder (internal drag-to-folder)    |

---

## TextEdit

| Shortcut              | Action                                        |
|-----------------------|-----------------------------------------------|
| Cmd+N                 | New document                                  |
| Cmd+O                 | Open file                                     |
| Cmd+S                 | Save                                          |
| Cmd+Shift+S           | Save As                                       |
| Cmd+W                 | Close window                                  |
| Cmd+Z                 | Undo                                          |
| Cmd+Shift+Z           | Redo                                          |
| Cmd+X                 | Cut                                           |
| Cmd+C                 | Copy                                          |
| Cmd+V                 | Paste                                         |
| Cmd+A                 | Select All                                    |

---

## Terminal

| Shortcut              | Action                                        |
|-----------------------|-----------------------------------------------|
| Cmd+N                 | New Terminal window                           |
| Cmd+T                 | New tab                                       |
| Cmd+Shift+W           | Close current tab                             |
| Cmd+W                 | Close window                                  |
| Cmd+C                 | Copy selection                                |
| Cmd+V                 | Paste                                         |
| Cmd+A                 | Select All                                    |
| Cmd+[1-9]             | Switch to tab by number (planned)             |

Terminal tabs support full PTY with VT100/VT220 compatibility including 256-color SGR,
true-color (24-bit) sequences, erase-in-line, and scroll margin regions.

---

## App Store

| Shortcut              | Action                                        |
|-----------------------|-----------------------------------------------|
| Cmd+F                 | Focus search field                            |
| Return                | Execute search                                |

---

## Settings

Settings uses mouse-driven controls. There are no keyboard shortcuts beyond standard
window management.

---

## Notes

- "Cmd" in this document refers to the Meta/Super/Windows key on Linux and the Command
  key on macOS hosts.
- Shortcuts that open status windows (Undo, Redo, etc.) will show a message explaining
  the action requires a document-aware app or a future compositor feature.
- All menu shortcuts are defined in `crates/retro-shell/src/menu_server.rs` and each
  application's `src/main.rs`.
