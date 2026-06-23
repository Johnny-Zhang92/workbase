<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import { check } from '@tauri-apps/plugin-updater';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { SearchAddon } from '@xterm/addon-search';
  import '@xterm/xterm/css/xterm.css';

  import Sidebar from './lib/Sidebar.svelte';
  import TabBar from './lib/TabBar.svelte';
  import FilePanel from './lib/FilePanel.svelte';
  import TerminalSearch from './lib/TerminalSearch.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import QuickCommandPanel from './lib/QuickCommandPanel.svelte';
  import Settings from './lib/Settings.svelte';
  import { PtyBridge } from './lib/pty';
  import { appState } from './lib/stores.svelte';
  import { t, initLocale, locale } from './lib/i18n.svelte';
  import { applyTheme, getTheme, loadThemeName, type TerminalTheme } from './lib/theme.svelte';
  import { handleKeyEvent, loadKeybindings, eventToBinding, setBinding, recordingId } from './lib/keybindings.svelte';
  import type { Project, Session } from './lib/types';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  // Initialize i18n and keybindings on first mount
  let initDone = false;
  $effect(() => {
    if (!initDone) { initDone = true; initLocale(); loadKeybindings(); checkPreviousCrash(); loadClipboardSetting(); }
  });

  // ── Global error handling ──

  let globalError = $state<string | null>(null);
  let crashInfo = $state<string | null>(null);

  // ── Update checking ──

  let updateAvailable = $state(false);
  let updateVersion = $state('');
  let updateNotes = $state('');
  let updateDownloading = $state(false);

  async function checkUpdate() {
    try {
      const update = await check();
      if (update) {
        updateAvailable = true;
        updateVersion = update.version;
        updateNotes = update.body || '';
      }
    } catch (_) {
      // Silently ignore — GitHub may be unreachable
    }
  }

  async function doUpdate() {
    try {
      updateDownloading = true;
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
      }
    } catch (e) {
      console.error('Update failed:', e);
      updateDownloading = false;
    }
  }

  function skipUpdate() {
    updateAvailable = false;
  }

  // Check for updates 5 seconds after init (don't block startup)
  $effect(() => {
    if (initDone) {
      const timer = setTimeout(checkUpdate, 5000);
      return () => clearTimeout(timer);
    }
  });

  async function checkPreviousCrash() {
    try {
      const path = await invoke<string | null>('check_crash');
      if (path) {
        crashInfo = path;
      }
    } catch (_) {
      // Silently ignore — crash check is best-effort
    }
  }

  async function dismissCrash() {
    crashInfo = null;
    await invoke('clear_crashes').catch(() => {});
  }

  async function onGlobalError(msg: string | Event, source?: string, lineno?: number, colno?: number) {
    // Don't show error dialog for network errors from WebSocket connections etc.
    if (typeof msg === 'string' && (msg.includes('WebSocket') || msg.includes('ResizeObserver'))) {
      return;
    }
    const detail = [msg, source, `L${lineno}:${colno}`].filter(Boolean).join(' | ');
    globalError = detail;
    console.error('[Workbase Error]', detail);
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    const prevOnerror = window.onerror;
    const prevRejection = window.onunhandledrejection;

    window.onerror = (msg, source, lineno, colno, err) => {
      onGlobalError(String(msg), source, lineno, colno);
      if (prevOnerror) prevOnerror.call(window, msg, source, lineno, colno, err);
      return true; // Prevent default browser error dialog
    };

    window.onunhandledrejection = (event) => {
      onGlobalError(event.reason?.message || String(event.reason));
      if (prevRejection) prevRejection.call(window, event);
    };

    return () => {
      window.onerror = prevOnerror;
      window.onunhandledrejection = prevRejection;
    };
  });

  interface PaneInfo {
    ptyId: string;
    term: Terminal;
    bridge: PtyBridge;
    fitAddon: FitAddon;
    searchAddon: SearchAddon;
    currentCwd: string;
    cwdDebounceTimer: ReturnType<typeof setTimeout> | null;
  }

  interface TerminalTab {
    sessionId: string;
    sessionName: string;
    pane: PaneInfo;
    splitPane?: PaneInfo;
    splitMode?: 'vertical' | 'horizontal';
    splitRatio?: number;
    activePane?: 'primary' | 'secondary';
    unlistenResize: () => void;
  }

  let tabs = $state<TerminalTab[]>([]);
  let activeTabId = $state<string | null>(null);
  let containerRefs = $state<Record<string, HTMLDivElement>>({});
  let searchVisible = $state(false);
  let settingsVisible = $state(false);
  let quickCommandVisible = $state(false);
  let slashPanelBridge = $state<PtyBridge | null>(null);

  let activeSearchAddon = $derived(
    searchVisible ? (tabs.find(t => t.sessionId === activeTabId)?.pane.searchAddon ?? null) : null
  );
  let activeSearchTerm = $derived(
    searchVisible ? (tabs.find(t => t.sessionId === activeTabId)?.pane.term ?? null) : null
  );

  // ── Smart clipboard ──

  let selectToCopy = $state(false);
  let contextMenu = $state<{ x: number; y: number; tab: TerminalTab; pane: PaneInfo; hasSelection: boolean } | null>(null);

  async function loadClipboardSetting() {
    try {
      const v = await invoke<string | null>('get_setting', { key: 'selectToCopy' });
      selectToCopy = v === 'true';
    } catch (_) {}
  }

  async function saveClipboardSetting(val: boolean) {
    selectToCopy = val;
    await invoke('set_setting', { key: 'selectToCopy', value: String(val) }).catch(() => {});
  }

  function setupTerminalClipboard(term: Terminal, bridge: PtyBridge, atPrompt: () => boolean, onSlash: () => void) {
    term.attachCustomKeyEventHandler((e) => {
      // / key: open quick command panel when at shell prompt
      if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key === '/' && atPrompt()) {
        onSlash();
        return false; // we handled it, don't send to PTY
      }
      // Ctrl+Shift+C always copies (standard terminal shortcut)
      if (e.ctrlKey && e.shiftKey && (e.key === 'c' || e.key === 'C')) {
        const sel = term.getSelection();
        if (sel) { navigator.clipboard.writeText(sel).catch(() => {}); }
        return false; // we handled it, don't let xterm send to PTY
      }
      // Ctrl+Shift+V always pastes
      if (e.ctrlKey && e.shiftKey && (e.key === 'v' || e.key === 'V')) {
        navigator.clipboard.readText().then(t => bridge.write(t)).catch(() => {});
        return false;
      }
      // Ctrl+C: copy if selection exists, otherwise let xterm send to PTY
      if (e.ctrlKey && !e.shiftKey && (e.key === 'c' || e.key === 'C')) {
        const sel = term.getSelection();
        if (sel) {
          navigator.clipboard.writeText(sel).catch(() => {});
          term.clearSelection();
          return false; // we handled it, don't send to PTY
        }
        return true; // no selection — let xterm send Ctrl+C (SIGINT) to PTY
      }
      // Ctrl+V: always paste (don't send literal Ctrl+V to PTY)
      if (e.ctrlKey && !e.shiftKey && (e.key === 'v' || e.key === 'V')) {
        navigator.clipboard.readText().then(t => bridge.write(t)).catch(() => {});
        return false;
      }
      return true; // let xterm process all other keys normally
    });

    // Select-to-copy: auto-copy when mouse selection ends
    term.onSelectionChange(() => {
      if (selectToCopy) {
        const sel = term.getSelection();
        if (sel) {
          navigator.clipboard.writeText(sel).catch(() => {});
        }
      }
    });
  }

  function onTerminalContextMenu(e: MouseEvent, tab: TerminalTab, pane: PaneInfo) {
    e.preventDefault();
    const sel = pane.term.getSelection();
    contextMenu = { x: e.clientX, y: e.clientY, tab, pane, hasSelection: !!sel };
  }

  function closeContextMenu() { contextMenu = null; }

  async function ctxCopy(pane: PaneInfo) {
    const sel = pane.term.getSelection();
    if (sel) { await navigator.clipboard.writeText(sel); }
    closeContextMenu();
  }

  async function ctxPaste(pane: PaneInfo) {
    try {
      const text = await navigator.clipboard.readText();
      pane.bridge.write(text).catch(() => {});
    } catch (_) {}
    closeContextMenu();
  }

  function ctxSelectAll(pane: PaneInfo) {
    pane.term.selectAll();
    closeContextMenu();
  }

  function shellOsc7Init(shell: string): string | null {
    const lower = shell.toLowerCase();
    if (lower.endsWith('powershell.exe') || lower.endsWith('pwsh.exe')) {
      // Use concatenation to avoid double-quote escaping in interpolated strings
      return String.raw`function prompt { $p = (Get-Location).Path; $e = [char]27; $bel = [char]7; Write-Host -NoNewline ($e + ']7;file://' + $env:COMPUTERNAME + '/' + ($p -replace '\\', '/') + $bel); 'PS ' + $p + '> ' }` + '\r\n';
    }
    if (lower.endsWith('bash') || lower.endsWith('bash.exe')) {
      return `export PROMPT_COMMAND='printf "\\033]7;file://%s%s\\033\\\\" "$HOSTNAME" "$PWD"'\r\n`;
    }
    if (lower.endsWith('zsh') || lower.endsWith('zsh.exe')) {
      return `precmd() { printf "\\033]7;file://%s%s\\033\\\\" "$HOSTNAME" "$PWD"; }\r\n`;
    }
    return null;
  }

  function toggleSearch() {
    if (!activeTabId) return;
    searchVisible = !searchVisible;
  }

  function closeSearch() {
    activeSearchAddon?.clearDecorations();
    searchVisible = false;
  }

  function parseOsc7(buf: string): { cwd: string | null; remaining: string } {
    const osc7Regex = /\x1b\]7;file:\/\/[^\/]*(\/[^\x07\x1b]*)[\x07\x1b]/g;
    let match;
    let lastCwd: string | null = null;
    while ((match = osc7Regex.exec(buf)) !== null) {
      try {
        lastCwd = decodeURIComponent(match[1]);
      } catch (_) { lastCwd = match[1]; }
      // Strip leading / from Windows paths like /c:/Users
      if (lastCwd && /^\/[a-zA-Z]:/.test(lastCwd)) {
        lastCwd = lastCwd.slice(1);
      }
      if (lastCwd && lastCwd.endsWith('\\')) lastCwd = lastCwd.slice(0, -1);
    }
    let remaining = buf;
    if (remaining.length > 4096) remaining = remaining.slice(-2048);
    return { cwd: lastCwd, remaining };
  }

  $effect(() => {
    const sessionId = appState.activeSessionId;
    if (!sessionId) return;
    const existing = tabs.find(t => t.sessionId === sessionId);
    if (existing) {
      if (activeTabId !== sessionId) activateTab(sessionId);
    } else {
      openTab(sessionId);
    }
  });

  // Handle tab close signal from sidebar
  $effect(() => {
    const signal = appState.closeTabSignal;
    if (!signal) return;
    closeTabSilent(signal);
    appState.closeTabSignal = null;
  });

  // Handle close-all-tabs signal (project switch)
  $effect(() => {
    if (!appState.closeAllTabs) return;
    for (const tab of [...tabs]) {
      closeTabSilent(tab.sessionId);
    }
    activeTabId = null;
    appState.closeAllTabs = false;
  });

  // Load git status + start file watch when active project changes
  let watchedPath = '';
  $effect(() => {
    const projectId = appState.activeProjectId;
    if (!projectId) {
      appState.gitBranch = '';
      appState.gitFiles = {};
      if (watchedPath) { invoke('stop_watch').catch(() => {}); watchedPath = ''; }
      return;
    }
    const project = appState.projects.find(p => p.id === projectId);
    if (!project) return;
    loadGitStatus(project.root_path);

    // Start file watching for auto-refresh
    if (project.root_path !== watchedPath) {
      if (watchedPath) invoke('stop_watch').catch(() => {});
      invoke('start_watch', { path: project.root_path }).catch(() => {});
      watchedPath = project.root_path;
    }
  });

  // Listen for file change events (once on mount)
  let fileWatchFlash = $state(false);
  $effect(() => {
    let unlistenFn: (() => void) | undefined;
    (async () => {
      unlistenFn = await listen('file-changed', async () => {
        if (gitDebounce) clearTimeout(gitDebounce);
        gitDebounce = setTimeout(async () => {
          const projectId = appState.activeProjectId;
          if (projectId) {
            const project = appState.projects.find(p => p.id === projectId);
            if (project) await loadGitStatus(project.root_path);
          }
          appState.fileTreeVersion++;
          // Flash status bar to give visible confirmation
          fileWatchFlash = true;
          setTimeout(() => fileWatchFlash = false, 800);
        }, 600);
      });
    })();
    return () => { unlistenFn?.(); };
  });

  interface GitStatusResult {
    branch: string;
    files: { path: string; status: string }[];
  }

  let gitDebounce: ReturnType<typeof setTimeout> | null = null;

  async function loadGitStatus(rootPath: string) {
    try {
      const result = await invoke<GitStatusResult>('git_status', { rootPath });
      appState.gitBranch = result.branch;
      const files: Record<string, string> = {};
      for (const f of result.files) {
        files[f.path.replace(/\\/g, '/')] = f.status;
      }
      appState.gitFiles = files;
    } catch (_) {
      appState.gitBranch = '';
      appState.gitFiles = {};
    }
  }

  // Handle command palette actions
  $effect(() => {
    const action = appState.paletteAction;
    if (!action) return;
    if (action === 'next-tab') {
      const idx = tabs.findIndex(t => t.sessionId === activeTabId);
      const next = idx >= tabs.length - 1 ? 0 : idx + 1;
      if (tabs[next]) appState.activeSessionId = tabs[next].sessionId;
    } else if (action === 'prev-tab') {
      const idx = tabs.findIndex(t => t.sessionId === activeTabId);
      const prev = idx <= 0 ? tabs.length - 1 : idx - 1;
      if (tabs[prev]) appState.activeSessionId = tabs[prev].sessionId;
    } else if (action === 'add-project') {
      addProjectFromPalette();
    } else if (action === 'split-v' && activeTabId) {
      splitPane(activeTabId, 'vertical');
    } else if (action === 'split-h' && activeTabId) {
      splitPane(activeTabId, 'horizontal');
    } else if (action === 'close-split' && activeTabId) {
      closeSplitPane(activeTabId);
    }
    appState.paletteAction = null;
  });

  function closePalette() {
    appState.paletteVisible = false;
  }

  function closeSettings() {
    settingsVisible = false;
  }

  // ── Window state save/restore ──

  let windowStateRestored = false;
  let windowStateSaveEnabled = false;

  async function saveWindowState() {
    if (!windowStateSaveEnabled) return;
    const state = JSON.stringify({
      activeProjectId: appState.activeProjectId,
      activeSessionId: appState.activeSessionId,
      openSessionIds: tabs.map(t => t.sessionId),
    });
    invoke('set_setting', { key: 'windowState', value: state }).catch(() => {});
  }

  // Auto-save when tabs or active session change (immediate, no debounce)
  $effect(() => {
    const _tabIds = tabs.map(t => t.sessionId).join(',');
    const _active = appState.activeSessionId;
    const _project = appState.activeProjectId;
    if (windowStateSaveEnabled && (_project !== null || _tabIds !== '')) {
      saveWindowState();
    }
  });

  async function restoreWindowState() {
    if (windowStateRestored) return;
    windowStateRestored = true;

    try {
      const raw = await invoke<string | null>('get_setting', { key: 'windowState' });
      if (!raw) { windowStateSaveEnabled = true; return; }
      const state = JSON.parse(raw);
      if (!state.activeProjectId || !state.openSessionIds?.length) {
        windowStateSaveEnabled = true; return;
      }

      // Restore project (sidebar will load sessions for this project)
      appState.activeProjectId = state.activeProjectId;
      await new Promise(r => setTimeout(r, 100));

      const openIds = state.openSessionIds as string[];
      const lastId = state.activeSessionId as string | undefined;

      // Open tabs sequentially — set activeSessionId for each, the $effect opens them
      for (const sid of openIds) {
        if (sid === lastId) continue; // open last
        appState.activeSessionId = sid;
        // Wait for openTab to finish (initializing guard)
        let waited = 0;
        while (appState.tabOpening && waited < 3000) {
          await new Promise(r => setTimeout(r, 50));
          waited += 50;
        }
      }
      // Open the last tab (or the only tab) as active
      if (lastId && !tabs.find(t => t.sessionId === lastId)) {
        appState.activeSessionId = lastId;
      } else if (openIds.length > 0) {
        appState.activeSessionId = openIds[openIds.length - 1];
      }
    } catch (_) {}
    windowStateSaveEnabled = true;
  }

  // Attempt restore once projects are loaded
  $effect(() => {
    if (appState.projects.length > 0 && !windowStateRestored) {
      restoreWindowState();
    }
  });

  async function loadTerminalTheme(): Promise<{ fontFamily: string; fontSize: number; theme: TerminalTheme }> {
    const [ff, fs] = await Promise.all([
      invoke<string | null>('get_setting', { key: 'fontFamily' }).catch(() => null),
      invoke<string | null>('get_setting', { key: 'fontSize' }).catch(() => null),
    ]);
    const themeName = await loadThemeName();
    return {
      fontFamily: ff || 'Cascadia Code, Fira Code, JetBrains Mono, Consolas, monospace',
      fontSize: parseInt(fs || '14') || 14,
      theme: getTheme(themeName),
    };
  }

  async function applyThemeToTerminals() {
    const { fontFamily, fontSize, theme } = await loadTerminalTheme();
    for (const tab of tabs) {
      tab.pane.term.options.fontFamily = fontFamily;
      tab.pane.term.options.fontSize = fontSize;
      applyTheme(tab.pane.term, theme);
      if (tab.splitPane) {
        tab.splitPane.term.options.fontFamily = fontFamily;
        tab.splitPane.term.options.fontSize = fontSize;
        applyTheme(tab.splitPane.term, theme);
      }
    }
  }

  // Register apply callback for settings component
  (globalThis as any).__workbaseApplyTheme = applyThemeToTerminals;

  async function addProjectFromPalette() {
    closePalette();
    let selected: string | null = null;
    try {
      selected = await open({ directory: true, multiple: false, title: 'Select Project Folder' }) as string | null;
    } catch (_) { return; }
    if (!selected) return;
    try {
      const name = selected.split(/[/\\]/).pop() || selected;
      await invoke('create_project', { name, rootPath: selected });
      appState.projects = await invoke<Project[]>('list_projects');
    } catch (e) { console.error(e); }
  }

  function closePane(pane: PaneInfo) {
    if (pane.cwdDebounceTimer) clearTimeout(pane.cwdDebounceTimer);
    pane.bridge.destroy();
    try { pane.bridge.kill().catch(() => {}); } catch (_) {}
    try { pane.term.dispose(); } catch (_) {}
  }

  function closeTabSilent(sessionId: string) {
    const tab = tabs.find(t => t.sessionId === sessionId);
    if (!tab) return;
    closePane(tab.pane);
    if (tab.splitPane) {
      closePane(tab.splitPane);
      delete containerRefs[tab.splitPane.ptyId];
    }
    const wasActive = activeTabId === sessionId;
    tabs = tabs.filter(t => t.sessionId !== sessionId);
    delete containerRefs[sessionId];
    if (wasActive) {
      if (tabs.length > 0) {
        const newId = tabs[tabs.length - 1].sessionId;
        activeTabId = newId;
        appState.activeSessionId = newId;
      } else {
        activeTabId = null;
        appState.activeSessionId = null;
      }
    }
  }

  async function openTab(sessionId: string) {
    if (appState.tabOpening) return;
    appState.tabOpening = true;

    const sessionIdNum = parseInt(sessionId.replace('session_', ''));
    const session = appState.sessions.find(s => s.id === sessionIdNum);
    if (!session) { appState.tabOpening = false; return; }

    const project = appState.projects.find(p => p.id === session.project_id);
    if (!project) { appState.tabOpening = false; return; }

    try {
      const savedShell = await invoke<string | null>('get_setting', { key: 'shellPath' }).catch(() => null);
      const shell = savedShell || await invoke<string>('detect_shell');
      let cwd: string;
      if (session.cwd) {
        // OSC 7 produces absolute paths; session creation may store relative ones
        if (/^[a-zA-Z]:/.test(session.cwd) || session.cwd.startsWith('/')) {
          cwd = session.cwd;
        } else {
          cwd = `${project.root_path}/${session.cwd}`;
        }
      } else {
        cwd = project.root_path;
      }

      let currentCwd = cwd;
      let cwdDebounceTimer: ReturnType<typeof setTimeout> | null = null;
      let osc7Buf = '';

      const { fontFamily, fontSize, theme } = await loadTerminalTheme();
      const term = new Terminal({
        cursorBlink: true,
        fontSize,
        fontFamily,
        cols: 80,
        rows: 24,
      });
      applyTheme(term, theme);

      const fitAddon = new FitAddon();
      term.loadAddon(fitAddon);

      const searchAddon = new SearchAddon();
      term.loadAddon(searchAddon);

      const bridge = new PtyBridge(sessionId);

      // Track whether we're at a shell prompt for /-trigger
      let atPrompt = true;

      // Set up event handlers on bridge BEFORE spawn
      bridge.onData((data) => {
        term.write(data);
        if (data.includes('\n')) atPrompt = true;
        const result = parseOsc7(osc7Buf + data);
        osc7Buf = result.remaining;
        if (result.cwd && result.cwd !== currentCwd) {
          currentCwd = result.cwd;
          appState.statusText = `${session.name} — ${currentCwd}`;
          if (cwdDebounceTimer) clearTimeout(cwdDebounceTimer);
          cwdDebounceTimer = setTimeout(() => {
            invoke('update_session_cwd', { id: sessionIdNum, cwd: currentCwd }).catch(() => {});
          }, 2000);
        }
      });

      bridge.onExit((code) => {
        term.write(`\r\n[Process exited with code ${code}]\r\n`);
        appState.statusText = `${session.name} — ${t('status.exited', { code })}`;
      });

      term.onData((data) => {
        atPrompt = false;
        bridge.write(data).catch(() => {});
      });

      setupTerminalClipboard(term, bridge,
        () => atPrompt,
        () => {
          slashPanelBridge = bridge;
          activeTabId = sessionId;
          quickCommandVisible = true;
        },
      );

      // Add tab to trigger DOM render BEFORE term.open()
      const pane: PaneInfo = {
        ptyId: sessionId,
        term,
        bridge,
        fitAddon,
        searchAddon,
        currentCwd,
        cwdDebounceTimer: null,
      };
      const tab: TerminalTab = {
        sessionId,
        sessionName: session.name,
        pane,
        unlistenResize: () => { try { fitAddon.fit(); } catch (_) {} },
      };
      tabs = [...tabs, tab];
      activeTabId = sessionId;

      // Wait for DOM to render so containerRefs[sessionId] is populated
      await new Promise(r => setTimeout(r, 0));

      const container = containerRefs[sessionId];
      if (container) {
        term.open(container);
        fitAddon.fit();
      }

      // Resize handler after terminal is opened
      term.onResize(({ cols, rows }) => {
        bridge.resize(cols, rows).catch(() => {});
      });

      // Now spawn the PTY — output goes to the visible xterm
      const { cols, rows } = term;
      await bridge.spawn(shell, cwd, cols, rows);

      // Inject OSC 7 shell integration for cwd tracking
      const initCmd = shellOsc7Init(shell);
      if (initCmd) bridge.write(initCmd).catch(() => {});

      // Auto-execute launch command after shell is ready
      if (session.launch_command) {
        setTimeout(() => {
          bridge.write(`${session.launch_command}\r`).catch(() => {});
        }, 500);
      }

      tab.pane.currentCwd = currentCwd;
      tab.pane.cwdDebounceTimer = cwdDebounceTimer;
      appState.statusText = `${session.name} — ${cwd}`;
      term.focus();
    } catch (e) {
      console.error('Failed to init terminal:', e);
      appState.statusText = `Error: ${e}`;
    } finally {
      appState.tabOpening = false;
    }
  }

  function activateTab(sessionId: string) {
    activeTabId = sessionId;
    const tab = tabs.find(t => t.sessionId === sessionId);
    if (!tab) return;
    const pane = tab.activePane === 'secondary' && tab.splitPane ? tab.splitPane : tab.pane;
    setTimeout(() => {
      try { pane.fitAddon.fit(); } catch (_) {}
      pane.term.focus();
    }, 0);
    const sessionIdNum = parseInt(sessionId.replace('session_', ''));
    const session = appState.sessions.find(s => s.id === sessionIdNum);
    if (session) {
      appState.statusText = `${session.name} — ${pane.currentCwd}`;
    }
  }

  async function closeTab(sessionId: string) {
    // Just close the tab — keep the session in DB/sidebar for re-opening
    closeTabSilent(sessionId);
  }

  async function newTab() {
    // Delegate to sidebar's addSession — it will call selectSession → triggers $effect → openTab
    if (!appState.activeProjectId) return;
    try {
      const name = t('sidebar.default_terminal', { n: appState.sessions.length + 1 });
      await invoke('create_session', { projectId: appState.activeProjectId, name, cwd: null });
      const sessions = await invoke<Session[]>('list_sessions', { projectId: appState.activeProjectId });
      appState.sessions = sessions;
      const s = sessions[sessions.length - 1];
      if (s) appState.activeSessionId = `session_${s.id}`;
    } catch (e) { console.error(e); }
  }

  function handleResize() {
    for (const tab of tabs) {
      if (tab.sessionId === activeTabId) {
        tab.unlistenResize();
        if (tab.splitPane) {
          try { tab.splitPane.fitAddon.fit(); } catch (_) {}
        }
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Keybinding recording mode (triggered from Settings)
    if (recordingId.value) {
      const binding = eventToBinding(e);
      if (binding) {
        e.preventDefault();
        setBinding(recordingId.value, binding);
        recordingId.value = null;
      }
      return;
    }

    // When modals are open, only handle Escape
    if (settingsVisible || appState.paletteVisible || quickCommandVisible) {
      if (e.key === 'Escape') { e.preventDefault(); settingsVisible = false; appState.paletteVisible = false; quickCommandVisible = false; }
      return;
    }

    // Ctrl+K: Quick Command Panel
    if (e.ctrlKey && !e.shiftKey && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      if (quickCommandVisible) {
        quickCommandVisible = false;
      } else if (activeTabId) {
        quickCommandVisible = true;
      }
      return;
    }

    const action = handleKeyEvent(e);
    if (!action) return;
    e.preventDefault();

    switch (action) {
      case 'toggle-sidebar':
        appState.sidebarVisible = !appState.sidebarVisible;
        break;
      case 'command-palette':
        appState.paletteVisible = true;
        break;
      case 'file-panel':
        appState.filePanelVisible = !appState.filePanelVisible;
        break;
      case 'terminal-search':
        toggleSearch();
        break;
      case 'split-pane':
        if (activeTabId) {
          const tab = tabs.find(t => t.sessionId === activeTabId);
          if (tab?.splitPane) closeSplitPane(activeTabId);
          else splitPane(activeTabId, 'vertical');
        }
        break;
      case 'next-tab':
        if (tabs.length > 1) {
          const idx = tabs.findIndex(t => t.sessionId === activeTabId);
          const next = idx >= tabs.length - 1 ? 0 : idx + 1;
          appState.activeSessionId = tabs[next].sessionId;
        }
        break;
      case 'prev-tab':
        if (tabs.length > 1) {
          const idx = tabs.findIndex(t => t.sessionId === activeTabId);
          const prev = idx <= 0 ? tabs.length - 1 : idx - 1;
          appState.activeSessionId = tabs[prev].sessionId;
        }
        break;
      case 'settings':
        settingsVisible = !settingsVisible;
        break;
    }
  }

  function focusPane(tabId: string, target: 'primary' | 'secondary') {
    const tab = tabs.find(t => t.sessionId === tabId);
    if (!tab || !tab.splitPane) return;
    tab.activePane = target;
    const pane = target === 'primary' ? tab.pane : tab.splitPane;
    setTimeout(() => pane.term.focus(), 0);
  }

  async function splitPane(sessionId: string, mode: 'vertical' | 'horizontal' = 'horizontal') {
    const tab = tabs.find(t => t.sessionId === sessionId);
    if (!tab || tab.splitPane) return;

    const sessionIdNum = parseInt(sessionId.replace('session_', ''));
    const session = appState.sessions.find(s => s.id === sessionIdNum);
    if (!session) return;

    const project = appState.projects.find(p => p.id === session.project_id);
    if (!project) return;

    try {
      // Create a real DB session for the split pane
      const savedShell = await invoke<string | null>('get_setting', { key: 'shellPath' }).catch(() => null);
      const shell = savedShell || await invoke<string>('detect_shell');
      const cwd = tab.pane.currentCwd;

      // Compute relative cwd from project root for DB storage
      let relCwd: string | null = null;
      if (cwd.startsWith(project.root_path)) {
        relCwd = cwd.slice(project.root_path.length).replace(/^[/\\]/, '') || null;
      }

      const newSessionName = `${session.name} (split)`;
      const newSession = await invoke<{ id: number }>('create_session', {
        projectId: session.project_id,
        name: newSessionName,
        cwd: relCwd,
      });
      const newSessionId = `session_${newSession.id}`;

      // Refresh sidebar sessions
      appState.sessions = await invoke<Session[]>('list_sessions', { projectId: session.project_id });

      const { fontFamily, fontSize, theme } = await loadTerminalTheme();
      const term = new Terminal({
        cursorBlink: true,
        fontSize,
        fontFamily,
        cols: 80,
        rows: 24,
      });
      applyTheme(term, theme);

      const fitAddon = new FitAddon();
      term.loadAddon(fitAddon);
      const searchAddon = new SearchAddon();
      term.loadAddon(searchAddon);

      const bridge = new PtyBridge(newSessionId);

      // Track cwd changes for the split pane
      let splitCwd = cwd;
      let splitOsc7Buf = '';
      let splitAtPrompt = true;
      bridge.onData((data) => {
        term.write(data);
        if (data.includes('\n')) splitAtPrompt = true;
        const result = parseOsc7(splitOsc7Buf + data);
        splitOsc7Buf = result.remaining;
        if (result.cwd && result.cwd !== splitCwd) {
          splitCwd = result.cwd;
          invoke('update_session_cwd', { id: newSession.id, cwd: splitCwd }).catch(() => {});
        }
      });
      bridge.onExit((code) => { term.write(`\r\n[Process exited with code ${code}]\r\n`); });
      term.onData((data) => { splitAtPrompt = false; bridge.write(data).catch(() => {}); });

      setupTerminalClipboard(term, bridge,
        () => splitAtPrompt,
        () => {
          slashPanelBridge = bridge;
          activeTabId = sessionId;
          quickCommandVisible = true;
        },
      );

      const splitPaneInfo: PaneInfo = {
        ptyId: newSessionId,
        term,
        bridge,
        fitAddon,
        searchAddon,
        currentCwd: splitCwd,
        cwdDebounceTimer: null,
      };

      tab.splitPane = splitPaneInfo;
      tab.splitMode = mode;
      tab.splitRatio = 50;
      tab.activePane = 'secondary';

      // Wait for DOM to render the new split pane container
      await new Promise(r => setTimeout(r, 0));

      // Primary pane DOM element stays the same — just re-fit
      try { tab.pane.fitAddon.fit(); } catch (_) {}

      // Attach split terminal to its new container
      const splitContainer = containerRefs[newSessionId];
      if (splitContainer) {
        term.open(splitContainer);
        fitAddon.fit();
      }

      term.onResize(({ cols, rows }) => { bridge.resize(cols, rows).catch(() => {}); });
      const { cols, rows } = term;
      await bridge.spawn(shell, cwd, cols, rows);

      const initCmd2 = shellOsc7Init(shell);
      if (initCmd2) bridge.write(initCmd2).catch(() => {});

      term.focus();
    } catch (e) {
      console.error('Failed to split pane:', e);
    }
  }

  function closeSplitPane(sessionId: string) {
    const tab = tabs.find(t => t.sessionId === sessionId);
    if (!tab || !tab.splitPane) return;
    const splitPaneInfo = tab.splitPane;

    // Kill the split pane PTY (session stays in DB/sidebar for re-opening)
    closePane(splitPaneInfo);
    delete containerRefs[splitPaneInfo.ptyId];

    // Remove split from current tab
    tab.splitPane = undefined;
    tab.splitMode = undefined;
    tab.splitRatio = undefined;
    tab.activePane = undefined;

    // Primary pane element stays in DOM — just re-fit and focus
    setTimeout(() => {
      try { tab.pane.fitAddon.fit(); } catch (_) {}
      tab.pane.term.focus();
    }, 0);
  }

  let dividerDragState = $state<{ tabId: string; startX: number; startY: number; startRatio: number } | null>(null);

  function onDividerDrag(e: MouseEvent, tabId: string) {
    const tab = tabs.find(t => t.sessionId === tabId);
    if (!tab || !tab.splitPane) return;
    e.preventDefault();
    dividerDragState = { tabId, startX: e.clientX, startY: e.clientY, startRatio: tab.splitRatio ?? 50 };

    function onMove(ev: MouseEvent) {
      if (!dividerDragState) return;
      const t = tabs.find(x => x.sessionId === dividerDragState!.tabId);
      if (!t || !t.splitPane) return;
      const container = containerRefs[t.pane.ptyId]?.parentElement;
      if (!container) return;
      const rect = container.getBoundingClientRect();
      let newRatio: number;
      if (t.splitMode === 'vertical') {
        newRatio = ((ev.clientX - rect.left) / rect.width) * 100;
      } else {
        newRatio = ((ev.clientY - rect.top) / rect.height) * 100;
      }
      t.splitRatio = Math.max(15, Math.min(85, newRatio));
    }

    function onUp() {
      dividerDragState = null;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function selectTab(id: string) {
    const tab = tabs.find(t => t.sessionId === id);
    if (!tab) return;
    appState.activeSessionId = id;
    // Sync sidebar
    const sid = parseInt(id.replace('session_', ''));
    const session = appState.sessions.find(s => s.id === sid);
    if (session) appState.activeProjectId = session.project_id;
  }
</script>

<svelte:window on:resize={handleResize} on:keydown={handleKeydown} on:beforeunload={() => saveWindowState()} />

<div class="app-container" style="padding-left: {appState.sidebarVisible ? appState.sidebarWidth : 0}px; padding-right: {appState.filePanelVisible && appState.activeProjectId ? appState.filePanelWidth : 0}px">
  {#if tabs.length > 0}
    <div class="terminal-area">
      {#if searchVisible}
        <TerminalSearch addon={activeSearchAddon} term={activeSearchTerm} onClose={closeSearch} />
      {/if}
      <TabBar
        tabs={tabs.map(t => ({ sessionId: t.sessionId, sessionName: t.sessionName }))}
        {activeTabId}
        onSelectTab={selectTab}
        onCloseTab={closeTab}
        onNewTab={newTab}
        onReorder={(from, to) => {
          const reordered = [...tabs];
          const [moved] = reordered.splice(from, 1);
          reordered.splice(to, 0, moved);
          tabs = reordered;
        }}
      />
      <div class="terminal-panels">
        {#each tabs as tab (tab.sessionId)}
          <div
            class="terminal-container"
            class:active={tab.sessionId === activeTabId}
            class:split-layout={!!tab.splitPane}
            class:split-v={tab.splitMode === 'vertical'}
            class:split-h={tab.splitMode === 'horizontal'}
          >
            <div
              bind:this={containerRefs[tab.pane.ptyId]}
              class="pane"
              class:active-pane={!tab.splitPane || (tab.activePane ?? 'primary') === 'primary'}
              style={tab.splitPane ? `flex: ${tab.splitRatio ?? 50}%` : ''}
              onclick={() => tab.splitPane && focusPane(tab.sessionId, 'primary')}
              oncontextmenu={(e) => onTerminalContextMenu(e, tab, tab.pane)}
            ></div>
            {#if tab.splitPane}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="pane-divider"
                onmousedown={(e) => onDividerDrag(e, tab.sessionId)}
                role="separator"
              ></div>
              <div
                bind:this={containerRefs[tab.splitPane.ptyId]}
                class="pane"
                class:active-pane={tab.activePane === 'secondary'}
                style="flex: {(100 - (tab.splitRatio ?? 50))}%"
                onclick={() => focusPane(tab.sessionId, 'secondary')}
                oncontextmenu={(e) => tab.splitPane && onTerminalContextMenu(e, tab, tab.splitPane)}
              ></div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="welcome">
      <div class="welcome-content">
        <h1>{t('welcome.title')}</h1>
        <p>{t('welcome.subtitle')}</p>
        <p class="hint">{t('welcome.hint1')}</p>
        <p class="hint">{@html t('welcome.hint2')}</p>
      </div>
    </div>
  {/if}
</div>

<div class="statusbar" class:watch-flash={fileWatchFlash}>
  <span class="status-left">{appState.statusText}{#if fileWatchFlash} — {t('status.file_changed')}{/if}</span>
  <span class="status-right">
    {#if appState.gitBranch}
      {@const changed = Object.keys(appState.gitFiles).length}
      <span class="git-branch" title={`Branch: ${appState.gitBranch}`}>
        <span class="git-icon">⎇</span>
        {appState.gitBranch}
        {#if changed > 0}
          <span class="git-changed">({changed})</span>
        {/if}
      </span>
      <span class="status-sep"></span>
    {/if}
    {appState.activeSessionId ? t('status.tab_of', { n: tabs.findIndex(t => t.sessionId === activeTabId) + 1, total: tabs.length }) : t('status.no_terminal')}
  </span>
</div>

<Sidebar />
<FilePanel />
{#if appState.paletteVisible}
  <CommandPalette onClose={closePalette} onNewTab={newTab} {toggleSearch} onOpenSettings={() => settingsVisible = true} />
{/if}
{#if quickCommandVisible && activeTabId}
  {@const activeTab = tabs.find(t => t.sessionId === activeTabId)}
  {#if activeTab}
    <QuickCommandPanel
      sessionId={activeTabId}
      bridge={activeTab.pane.bridge}
      onClose={() => {
        if (slashPanelBridge) {
          slashPanelBridge.write('/').catch(() => {});
          slashPanelBridge = null;
        }
        quickCommandVisible = false;
      }}
      onAction={() => { slashPanelBridge = null; }}
    />
  {/if}
{/if}
{#if settingsVisible}
  <Settings onClose={closeSettings} />
{/if}

<!-- Crash recovery notice -->
{#if crashInfo}
  <div class="crash-overlay" role="alertdialog">
    <div class="crash-dialog">
      <h3>{t('error.title')}</h3>
      <p>{t('error.crash_detected')}</p>
      <div class="crash-actions">
        <button class="crash-btn secondary" onclick={dismissCrash}>{t('error.crash_dismiss')}</button>
      </div>
    </div>
  </div>
{/if}

<!-- Update notification -->
{#if updateAvailable}
  <div class="update-banner">
    <span class="update-msg">Workbase {updateVersion} is available. Current: v0.1.0</span>
    <button class="update-btn" onclick={doUpdate} disabled={updateDownloading}>
      {updateDownloading ? 'Downloading...' : 'Update'}
    </button>
    <button class="update-skip" onclick={skipUpdate}>Skip</button>
  </div>
{/if}

<!-- Terminal context menu -->
{#if contextMenu}
  {@const ctx = contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="ctx-overlay" onclick={closeContextMenu} oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }} role="presentation">
    <div class="ctx-menu" style="left: {ctx.x}px; top: {ctx.y}px;" role="menu">
      {#if ctx.hasSelection}
        <button class="ctx-item" onclick={() => ctxCopy(ctx.pane)} role="menuitem">{t('clipboard.copy')}</button>
      {/if}
      <button class="ctx-item" onclick={() => ctxPaste(ctx.pane)} role="menuitem">{t('clipboard.paste')}</button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => ctxSelectAll(ctx.pane)} role="menuitem">{t('clipboard.select_all')}</button>
    </div>
  </div>
{/if}

<!-- Global error dialog -->
{#if globalError}
  <div class="crash-overlay" role="alertdialog">
    <div class="crash-dialog">
      <h3>{t('error.title')}</h3>
      <p class="crash-detail">{globalError}</p>
      <div class="crash-actions">
        <button class="crash-btn secondary" onclick={() => globalError = null}>{t('error.crash_dismiss')}</button>
        <button class="crash-btn" onclick={() => location.reload()}>{t('error.reload')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .app-container {
    height: calc(100% - 28px);
    width: 100%;
    background: #1e1e1e;
    overflow: hidden;
    transition: padding-left 0.15s ease;
  }
  .terminal-area {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .terminal-panels {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
  .terminal-container {
    position: absolute;
    inset: 0;
    padding: 4px;
    display: none;
  }
  .terminal-container.active {
    display: block;
  }
  .split-layout.active {
    display: flex;
  }
  .terminal-container :global(.xterm) { height: 100%; width: 100%; }
  .terminal-container :global(.xterm-viewport) { overflow-y: auto; }
  .split-layout { display: flex; }
  .split-layout.split-v { flex-direction: row; }
  .split-layout.split-h { flex-direction: column; }
  .pane {
    overflow: hidden; position: relative; padding: 4px;
    height: 100%;
    border: 1px solid transparent;
  }
  .split-layout .pane {
    height: auto;
    flex: 1;
  }
  .split-layout .pane.active-pane {
    border-color: #007acc;
  }
  .pane-divider {
    flex-shrink: 0;
    background: #454545;
    transition: background 0.15s;
  }
  .pane-divider:hover { background: #007acc; }
  .split-v .pane-divider {
    width: 4px; cursor: col-resize;
  }
  .split-h .pane-divider {
    height: 4px; cursor: row-resize;
  }
  .welcome {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .welcome-content { text-align: center; color: #888; }
  .welcome-content h1 { font-size: 32px; font-weight: 600; color: #ccc; margin-bottom: 12px; }
  .welcome-content p { font-size: 14px; margin-bottom: 8px; color: #888; }
  .welcome-content .hint { font-size: 12px; color: #666; }
  .welcome-content kbd {
    background: #3c3c3c; border: 1px solid #555;
    border-radius: 3px; padding: 1px 6px; font-size: 12px; color: #ccc;
  }
  .statusbar {
    position: fixed; bottom: 0; left: 0; right: 0; height: 28px;
    background: #007acc; color: #fff; display: flex;
    align-items: center; padding: 0 12px; font-size: 12px; z-index: 20;
  }
  .status-left { flex: 1; }
  .status-right { display: flex; align-items: center; gap: 8px; color: rgba(255, 255, 255, 0.8); }
  .git-branch { display: flex; align-items: center; gap: 4px; color: rgba(255, 255, 255, 0.85); }
  .git-icon { font-size: 13px; }
  .git-changed { color: rgba(255, 255, 255, 0.6); font-size: 10px; }
  .status-sep { width: 1px; height: 14px; background: rgba(255, 255, 255, 0.3); }
  .status-right { color: rgba(255, 255, 255, 0.8); }
  .watch-flash { background: #16825d; transition: background 0.15s; }

  /* Crash / error overlay */
  .crash-overlay {
    position: fixed; inset: 0; z-index: 9999;
    background: rgba(0, 0, 0, 0.6);
    display: flex; align-items: center; justify-content: center;
  }
  .crash-dialog {
    background: #252526; border: 1px solid #e88;
    border-radius: 8px; padding: 24px; max-width: 440px; width: 90%;
    box-shadow: 0 8px 32px rgba(0,0,0,0.6);
  }
  .crash-dialog h3 {
    font-size: 16px; font-weight: 600; color: #e88;
    margin: 0 0 8px;
  }
  .crash-dialog p {
    font-size: 13px; color: #ccc; margin: 0 0 16px;
    line-height: 1.5;
  }
  .crash-detail {
    font-size: 12px; color: #999;
    background: #1e1e1e; padding: 8px; border-radius: 4px;
    word-break: break-all; max-height: 100px; overflow-y: auto;
  }
  .crash-actions { display: flex; gap: 8px; justify-content: flex-end; }
  .crash-btn {
    padding: 6px 16px; border: none; border-radius: 4px;
    cursor: pointer; font-size: 13px;
    background: #007acc; color: #fff;
  }
  .crash-btn:hover { background: #005fa3; }
  .crash-btn.secondary {
    background: #3c3c3c; color: #ccc;
  }
  .crash-btn.secondary:hover { background: #555; }

  /* Update notification banner */
  .update-banner {
    position: fixed; top: 0; left: 0; right: 0; z-index: 9998;
    background: #007acc; color: #fff;
    display: flex; align-items: center; justify-content: center; gap: 12px;
    padding: 8px 16px; font-size: 13px;
  }
  .update-msg { flex: 1; text-align: center; }
  .update-btn {
    padding: 4px 14px; background: #fff; color: #007acc;
    border: none; border-radius: 3px; font-size: 12px; font-weight: 600;
    cursor: pointer;
  }
  .update-btn:hover { background: #e0e0e0; }
  .update-btn:disabled { opacity: 0.6; cursor: default; }
  .update-skip {
    padding: 4px 10px; background: transparent; color: rgba(255,255,255,0.7);
    border: 1px solid rgba(255,255,255,0.3); border-radius: 3px;
    font-size: 12px; cursor: pointer;
  }
  .update-skip:hover { background: rgba(255,255,255,0.1); }

  /* Terminal context menu */
  .ctx-overlay {
    position: fixed; inset: 0; z-index: 9998;
  }
  .ctx-menu {
    position: fixed;
    background: #2d2d2d; border: 1px solid #555;
    border-radius: 6px; box-shadow: 0 4px 16px rgba(0,0,0,0.5);
    padding: 4px; min-width: 140px; z-index: 9999;
  }
  .ctx-item {
    display: block; width: 100%; padding: 6px 12px;
    background: none; border: none; color: #ccc;
    font-size: 13px; text-align: left; cursor: pointer;
    border-radius: 3px;
  }
  .ctx-item:hover { background: #094771; color: #fff; }
  .ctx-sep {
    height: 1px; background: #555; margin: 3px 8px;
  }
</style>
