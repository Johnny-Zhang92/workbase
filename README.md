# Workbase

A lightweight, project-centric terminal workspace for developers who use CLI AI tools like Claude Code and Codex.

![Version](https://img.shields.io/badge/version-0.1.2-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-Proprietary-red)

> [!IMPORTANT]
> This is a **public releases and documentation repository only**. The source code of Workbase is not included. Workbase is proprietary software — you may download and use official binary releases under the [EULA](./EULA.md). See [LICENSE](./LICENSE) for details.

## What is Workbase?

Workbase is a desktop terminal emulator and developer workspace that organizes your terminal around **projects**, not just loose tabs. Each project remembers its sessions — plain shells, Claude Code chats, Codex sessions — and restores them when you reopen the app.

- **Project-first** — Sidebar shows projects → sessions. Everything persists across restarts.
- **Multi-tab terminals** — One tab per session, with split panes.
- **CLI AI ready** — Quick launch Claude Code / Codex from the sidebar or with `/` in terminal.
- **Session resume** — Save resume commands (`claude -r abc123`) and replay them with one click.
- **Built-in file tree** — Right-side panel with git status indicators.
- **Lightweight** — ~10MB installer, ~40MB RAM baseline. Built on Tauri v2 + system WebView.

## Download

### Windows

Download the `.msi` or `.exe` installer from [Releases](https://github.com/Johnny-Zhang92/workbase/releases).

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

## 中文介绍

Workbase 是一款轻量级的桌面终端模拟器，以"项目"为核心组织终端会话。适合使用 Claude Code、Codex 等 CLI AI 工具的开发者。

- **项目管理**：侧栏按项目管理终端会话，关闭重开自动恢复
- **多标签终端**：支持多 Tab 和分屏面板
- **CLI AI 集成**：侧栏或 `/` 快捷键一键启动 Claude Code / Codex
- **文件树**：右侧面板内置文件浏览，支持 Git 状态标注
- **轻量高效**：基于 Tauri v2 + 系统 WebView，安装包约 10MB，内存占用约 40MB

## License

Workbase is **proprietary software**. All rights reserved.

This repository exists for public releases, documentation, issue tracking, and downloads only. The source code is not included and is not licensed under any open source or source-available license.

Download and use of official binary releases is governed by the [End User License Agreement](./EULA.md). See [LICENSE](./LICENSE) and [PRIVACY.md](./PRIVACY.md) for additional details.

© 2026 ZhangLiHua
