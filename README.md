# Workbase

A lightweight, project-centric terminal workspace for developers who use CLI AI tools like Claude Code and Codex.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-Source%20Available-orange)

## What is Workbase?

Workbase organizes your terminal around **projects**, not just loose tabs. Each project remembers its sessions — plain shells, Claude Code chats, Codex sessions — and restores them when you reopen the app.

- **Project-first** — Sidebar shows projects → sessions. Everything persists across restarts.
- **Multi-tab terminals** — One tab per session, with split panes.
- **CLI AI ready** — Quick launch Claude Code / Codex from the sidebar or with `/` in terminal.
- **Session resume** — Save resume commands (`claude -r abc123`) and replay them with one click.
- **Built-in file tree** — Right-side panel with git status indicators.
- **Lightweight** — ~10MB installer, ~40MB RAM baseline. Built on Tauri v2 + system WebView.

## Download

### Windows

Download the `.msi` installer from [Releases](https://github.com/ZhangLiHua/workbase/releases).

### macOS / Linux

Coming soon.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+Shift+E` | Toggle file panel |
| `Ctrl+Shift+P` | Command palette |
| `Ctrl+Shift+F` | Find in terminal |
| `Ctrl+K` | Quick commands (launch AI tools) |
| `/` | Quick commands (when at shell prompt) |
| `Ctrl+Shift+V` | Paste |
| `Ctrl+Shift+C` | Copy selection |

## Tech Stack

| Layer | Choice |
|-------|--------|
| Desktop shell | Tauri v2 |
| Frontend | Svelte 5 |
| Terminal | xterm.js 5.x |
| PTY | portable-pty (Rust) |
| Storage | SQLite (rusqlite) |
| Git | git CLI |

## License

This software is **source available**. You may use it for free, but redistribution, modification, and commercial use are restricted. See [LICENSE](./LICENSE) for details.

© 2026 ZhangLiHua
