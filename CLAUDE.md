# Workbase — 项目级 AI CLI 终端工作台

## 产品定位

以"项目"为一级公民的桌面终端应用。用户在侧栏管理项目，每个项目下有多
个终端会话。关闭重开后项目路径和会话自动恢复。

**不内置任何 AI 能力** — 用户在终端里跑自己装的 CLI AI 工具。

竞品对比：
- Windows Terminal：纯终端，无项目管理
- Warp：有 AI 但无项目层级
- Tabby：有项目管理但较重（Electron）
- VS Code Terminal：依附于编辑器

## 技术栈

| 层 | 选型 | 说明 |
|---|---|---|
| 桌面壳 | Tauri v2 | 系统 WebView2，不打 Chromium，~5MB |
| 终端渲染 | xterm.js 5.5 + Canvas 渲染器 | VS Code 同款引擎 |
| PTY | portable-pty 0.8 (Rust) | 跨平台 PTY，Windows 用 ConPTY |
| 持久化 | rusqlite 0.31 (bundled SQLite) | 零依赖嵌入式数据库 |
| 前端框架 | Svelte 5 (runes) | 编译时消失，运行时零依赖 |
| 文件系统 | walkdir + ignore (Rust) | 原生性能目录遍历 |
| Git | 调 git CLI | 解析 --porcelain 输出 |

## 项目结构

```
src/                      # 前端 (Svelte 5)
├── App.svelte            # 主组件：终端、标签页、状态栏、全局键盘/错误处理 (~1100行)
├── lib/
│   ├── Sidebar.svelte    # 侧栏：项目列表、会话列表、拖拽排序
│   ├── TabBar.svelte     # 顶部标签栏，支持拖拽重排
│   ├── FilePanel.svelte  # 右侧文件树面板，支持拖拽调整宽度
│   ├── FileTree.svelte   # 文件树组件，懒加载子目录
│   ├── FileTreeNode.svelte
│   ├── CommandPalette.svelte    # Ctrl+Shift+P 命令面板
│   ├── QuickCommandPanel.svelte # Ctrl+K 快捷命令面板
│   ├── TerminalSearch.svelte    # Ctrl+Shift+F 终端搜索
│   ├── Settings.svelte          # 设置页：字体、主题、Shell、快捷键
│   ├── pty.ts             # PtyBridge：前端 PTY 通信封装（含 2ms 写缓冲）
│   ├── stores.svelte.ts   # appState：全局响应式状态（$state rune）
│   ├── i18n.svelte.ts     # 中英文国际化
│   ├── keybindings.svelte.ts  # 快捷键管理
│   ├── theme.svelte.ts    # 主题系统（7 个预设 + ANSI 调色板）
│   └── types.ts           # TypeScript 类型定义

src-tauri/                 # Rust 后端
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── lib.rs             # Tauri 命令注册、AppState
│   ├── terminal_engine.rs # PTY 引擎：spawn/write/resize/kill + reader loop
│   ├── db.rs              # SQLite：projects/sessions/settings/templates CRUD
│   ├── git.rs             # Git 状态解析（git --porcelain）
│   ├── watcher.rs         # 文件监视（notify crate，500ms 防抖）
│   └── crash.rs           # Panic hook + 崩溃恢复
```

## 数据流：终端按键到屏幕回显

```
用户按键
  → xterm.onData('a')
    → PtyBridge.write('a')         # 2ms 批量缓冲
      → invoke('pty_write', ...)   # IPC 到 Rust
        → TerminalEngine.write()   # 写入 PTY
          → 操作系统 PTY → Shell 处理
            → Shell 回显 'a' → PTY 输出
              → reader_loop 读到数据
                → WouldBlock 时立即 flush   # ~0ms 延迟
                  → app.emit('pty:data:...')  # IPC 回前端
                    → bridge.onData callback
                      → 小数据立即 term.write()    # 交互模式
                      → 大数据 requestAnimationFrame 批量  # TUI 模式
                        → xterm.js Canvas 渲染器 → 屏幕显示
```

## 核心数据结构

```typescript
// App.svelte
interface PaneInfo {
  ptyId: string; term: Terminal; bridge: PtyBridge;
  fitAddon: FitAddon; searchAddon: SearchAddon;
  currentCwd: string; cwdDebounceTimer: ReturnType<typeof setTimeout> | null;
}
interface TerminalTab {
  sessionId: string; sessionName: string;
  pane: PaneInfo;
  splitPane?: PaneInfo; splitMode?: 'vertical'|'horizontal';
  splitRatio?: number; activePane?: 'primary'|'secondary';
  unlistenResize: () => void;
}

// stores.svelte.ts — 全局状态（单例 $state 对象）
appState: { projects, sessions, templates, activeProjectId, activeSessionId,
            sidebarVisible, sidebarWidth, filePanelVisible, filePanelWidth,
            statusText, gitBranch, gitFiles, paletteVisible, paletteAction,
            closeTabSignal, closeAllTabs, tabOpening, fileTreeVersion, perfStats }
```

```sql
-- 核心表
projects(id INTEGER PRIMARY KEY, name TEXT, root_path TEXT, sort_order INTEGER, created_at TEXT)
sessions(id INTEGER PRIMARY KEY, project_id INTEGER, name TEXT, cwd TEXT, launch_command TEXT, sort_order INTEGER, last_active_at TEXT)
settings(key TEXT PRIMARY KEY, value TEXT)
session_templates(id INTEGER PRIMARY KEY, name TEXT, command TEXT, icon TEXT, sort_order INTEGER)
```

## 当前性能状态（2026-06-25）

### 已确认有效的优化
1. **Canvas 渲染器** — xterm.js Canvas addon，单 canvas 替代 1920 个 DOM 节点
2. **Rust reader loop WouldBlock 刷新** — 交互回声 ~0ms 延迟（不再用固定定时器）
3. **前端自适应渲染** — 小数据（<1KB，间隔>20ms）立即写 xterm；大数据/TUI 用 rAF 节流
4. **前端写缓冲** — PtyBridge.write() 2ms 批量聚集按键再 invoke
5. **OSC7 快速路径** — 仅当数据含 ESC 字符时才跑 OSC7 正则

### 已知问题
- **输入延迟仍存在**：按键到回显的端到端延迟尚未达到 Windows Terminal 水平。怀疑瓶颈可能在 ConPTY 层面或 xterm.js Canvas 渲染器在 WebView2 上的性能。
- **删除字符时更明显**：可能是 PowerShell 对 Backspace 的处理产生了更多转义序列。
- **窗口拖动偶尔掉帧**：Sidebar/FilePanel 的 `<svelte:window on:mousemove>` 在组件顶层注册，面板隐藏时也触发（Svelte 不允许把 svelte:window 放在 {#if} 内）。

### 可尝试的优化方向
1. 用原生 Canvas API 替代 xterm.js（类似 Phase 2 但更精简的实现）
2. 试用 `@xterm/addon-webgl` 替代 Canvas（需处理 WebGL 上下文丢失）
3. 在 Windows 上用 winpty 替代 ConPTY
4. 把高频更新的状态字段从 `$state` 大对象中拆出来减少响应式图规模
5. 限制 reader loop 的 emit 频率（当多个 session 同时输出时）
6. 启用 Tauri 的 `devtools: true`（已在 tauri.conf.json 中设置）用 Performance 面板做 CPU profile

## 常用命令

```bash
npm run tauri dev    # 开发模式（前端 HMR + Rust debug 编译）
npm run build        # 仅前端编译
npx tauri build      # 完整生产构建（MSI + NSIS 安装包）

# 前端检查
npm run check        # svelte-check + tsc

# 测试
npm test             # vitest（前端）
cargo test           # Rust 测试
```

## 版本

当前 0.1.1，Windows 安装包位于：
`src-tauri/target/release/bundle/msi/Workbase_0.1.1_x64_en-US.msi`
`src-tauri/target/release/bundle/nsis/Workbase_0.1.1_x64-setup.exe`
