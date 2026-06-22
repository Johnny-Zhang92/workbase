import { invoke } from '@tauri-apps/api/core';

type Locale = 'en' | 'zh';

const dictionaries: Record<Locale, Record<string, string>> = {
  en: {
    // Sidebar
    'sidebar.title': 'DevTerm',
    'sidebar.add_project': 'Add Project',
    'sidebar.drag_hint': 'Drag to reorder',
    'sidebar.new_terminal': '+ New Terminal',
    'sidebar.no_projects': 'No projects yet',
    'sidebar.ctx.open_explorer': 'Open in Explorer',
    'sidebar.ctx.rename_project': 'Rename Project',
    'sidebar.ctx.rename_session': 'Rename Session',
    'sidebar.ctx.copy_cwd': 'Copy Working Directory',
    'sidebar.ctx.remove_project': 'Remove Project',
    'sidebar.ctx.close_session': 'Close Session',
    'sidebar.status.selecting': 'Selecting project folder...',
    'sidebar.status.cancelled': 'Dialog cancelled',
    'sidebar.status.adding': 'Adding project...',
    'sidebar.status.added': 'Project "{name}" added',
    'sidebar.status.copied': 'Copied: {path}',
    'sidebar.status.copy_failed': 'Failed to copy',
    'sidebar.status.renamed_project': 'Project renamed to "{name}"',
    'sidebar.status.renamed_session': 'Session renamed to "{name}"',
    'sidebar.default_terminal': 'Terminal {n}',

    // Welcome screen
    'welcome.title': 'DevTerm',
    'welcome.subtitle': 'A project-centric terminal workspace',
    'welcome.hint1': 'Add a project, then create a terminal session.',
    'welcome.hint2': 'Press <kbd>Ctrl+B</kbd> to toggle sidebar, <kbd>Ctrl+Shift+P</kbd> for commands, <kbd>Ctrl+Shift+E</kbd> for files, <kbd>Ctrl+Shift+F</kbd> to search.',

    // Status bar
    'status.ready': 'DevTerm v0.1.0 — Ready',
    'status.no_terminal': 'No terminal',
    'status.tab_of': 'Tab {n} of {total}',
    'status.file_changed': 'file changed',
    'status.exited': 'exited (code {code})',
    'status.active': 'active',
    'status.error': 'Error: {msg}',

    // Command palette
    'cmd.toggle_sidebar': 'Toggle Sidebar',
    'cmd.toggle_files': 'Toggle File Panel',
    'cmd.find_in_terminal': 'Find in Terminal',
    'cmd.new_tab': 'New Terminal Tab',
    'cmd.close_tab': 'Close Active Tab',
    'cmd.next_tab': 'Next Tab',
    'cmd.prev_tab': 'Previous Tab',
    'cmd.split_v': 'Split Terminal Vertically',
    'cmd.split_h': 'Split Terminal Horizontally',
    'cmd.close_split': 'Close Split Pane',
    'cmd.add_project': 'Add Project',
    'cmd.open_settings': 'Open Settings',
    'cmd.category.view': 'View',
    'cmd.category.terminal': 'Terminal',
    'cmd.category.project': 'Project',
    'cmd.category.preferences': 'Preferences',
    'cmd.placeholder': 'Type a command...',
    'cmd.no_match': 'No matching commands',

    // Settings
    'settings.title': 'Settings',
    'settings.font_family': 'Font Family',
    'settings.font_family_hint': 'e.g. Cascadia Code, Fira Code, monospace',
    'settings.font_size': 'Font Size',
    'settings.background': 'Background',
    'settings.foreground': 'Foreground',
    'settings.shell_path': 'Shell Path',
    'settings.shell_hint': 'Leave empty to auto-detect',
    'settings.language': 'Language',
    'settings.language_hint': 'Restart to apply',

    // Keybindings
    'keybind.section_title': 'Keyboard Shortcuts',
    'keybind.command_palette': 'Command Palette',
    'keybind.split_pane': 'Split/Unsplit Pane',
    'keybind.press_keys': 'Press keys...',
    'keybind.reset': 'Reset',

    // Terminal search
    'search.placeholder': 'Find in terminal...',

    // File tree
    'files.title': 'Files',
    'files.loading': 'Loading...',
    'files.empty': 'Empty directory',
    'files.ctx.open': 'Open',
    'files.ctx.open_in_explorer': 'Open in Explorer',
    'files.ctx.reveal_in_explorer': 'Reveal in Explorer',
    'files.ctx.copy_path': 'Copy Path',
    'files.ctx.copy_rel_path': 'Copy Relative Path',

    // Tab bar
    'tab.new_hint': 'New Terminal',

    // Error recovery
    'error.title': 'Something went wrong',
    'error.unknown': 'An unexpected error occurred',
    'error.retry': 'Retry',
    'error.reload': 'Reload Application',
    'error.crash_detected': 'DevTerm closed unexpectedly last session.',
    'error.crash_view': 'View Details',
    'error.crash_dismiss': 'Dismiss',

    // Clipboard / Context menu
    'clipboard.copy': 'Copy',
    'clipboard.paste': 'Paste',
    'clipboard.select_all': 'Select All',
    'settings.select_to_copy': 'Copy on select',
    'settings.select_to_copy_hint': 'Automatically copy selected text to clipboard',
    'settings.theme': 'Theme',
    'settings.theme_hint': 'Select a color theme for the terminal',

    // Session launcher
    'launcher.new_session': 'New Session',
    'launcher.plain_shell': 'Plain Shell',
    'launcher.claude_code': 'Claude Code',
    'launcher.codex': 'Codex',
    'launcher.custom_template': 'Custom...',

    // Session templates
    'templates.title': 'Startup Templates',
    'templates.hint': 'Create reusable session launchers with pre-configured commands',
    'templates.new': 'New Template',
    'templates.name': 'Name',
    'templates.command': 'Command',
    'templates.icon': 'Icon',
    'templates.name_placeholder': 'e.g. Dev Server',
    'templates.command_placeholder': 'e.g. npm run dev',
    'templates.icon_placeholder': 'e.g. 🔧',
    'templates.save': 'Save',
    'templates.cancel': 'Cancel',
    'templates.edit': 'Edit',
    'templates.delete': 'Delete',

    // Quick command panel (Ctrl+K)
    'quickcmd.title': 'Quick Commands',
    'quickcmd.start_claude': 'Start Claude Code',
    'quickcmd.start_claude_hint': 'types: claude',
    'quickcmd.start_codex': 'Start Codex',
    'quickcmd.start_codex_hint': 'types: codex',
    'quickcmd.resume': 'Resume saved session',
    'quickcmd.resume_hint': 'types saved resume command',
    'quickcmd.remember': 'Remember current session',
    'quickcmd.remember_hint': 'save a resume command for this terminal',
    'quickcmd.clear': 'Clear all saved sessions',
    'quickcmd.help': 'Help',
    'quickcmd.remember_prompt': 'Paste the resume command (e.g. claude -r abc123):',
    'quickcmd.remember_saved': 'Resume command saved for this session.',
    'quickcmd.remember_cleared': 'All saved sessions cleared.',

    // Help text
    'quickcmd.help_text': 'Quick Commands let you start CLI AI tools and resume past sessions.\n\n• Start Claude Code / Codex — types the command in the terminal\n• Remember — save a resume command (e.g. "claude -r abc123") for this terminal\n• Resume — replay the saved resume command\n• Use ↑↓ to navigate, Enter to select, Esc to close',
  },
  zh: {
    // Sidebar
    'sidebar.title': 'DevTerm',
    'sidebar.add_project': '添加项目',
    'sidebar.drag_hint': '拖拽排序',
    'sidebar.new_terminal': '+ 新建终端',
    'sidebar.no_projects': '暂无项目',
    'sidebar.ctx.open_explorer': '在资源管理器中打开',
    'sidebar.ctx.rename_project': '重命名项目',
    'sidebar.ctx.rename_session': '重命名会话',
    'sidebar.ctx.copy_cwd': '复制工作目录',
    'sidebar.ctx.remove_project': '移除项目',
    'sidebar.ctx.close_session': '关闭会话',
    'sidebar.status.selecting': '正在选择项目文件夹...',
    'sidebar.status.cancelled': '已取消',
    'sidebar.status.adding': '正在添加项目...',
    'sidebar.status.added': '项目 "{name}" 已添加',
    'sidebar.status.copied': '已复制: {path}',
    'sidebar.status.copy_failed': '复制失败',
    'sidebar.status.renamed_project': '项目已重命名为 "{name}"',
    'sidebar.status.renamed_session': '会话已重命名为 "{name}"',
    'sidebar.default_terminal': '终端 {n}',

    // Welcome screen
    'welcome.title': 'DevTerm',
    'welcome.subtitle': '以项目为中心的终端工作台',
    'welcome.hint1': '添加一个项目，然后创建终端会话。',
    'welcome.hint2': '<kbd>Ctrl+B</kbd> 切换侧栏、<kbd>Ctrl+Shift+P</kbd> 命令面板、<kbd>Ctrl+Shift+E</kbd> 文件面板、<kbd>Ctrl+Shift+F</kbd> 搜索。',

    // Status bar
    'status.ready': 'DevTerm v0.1.0 — 就绪',
    'status.no_terminal': '无终端',
    'status.tab_of': '第 {n} 个标签（共 {total} 个）',
    'status.file_changed': '文件已变更',
    'status.exited': '已退出（代码 {code}）',
    'status.active': '已激活',
    'status.error': '错误: {msg}',

    // Command palette
    'cmd.toggle_sidebar': '切换侧栏',
    'cmd.toggle_files': '切换文件面板',
    'cmd.find_in_terminal': '在终端中查找',
    'cmd.new_tab': '新建终端标签',
    'cmd.close_tab': '关闭当前标签',
    'cmd.next_tab': '下一个标签',
    'cmd.prev_tab': '上一个标签',
    'cmd.split_v': '垂直分割终端',
    'cmd.split_h': '水平分割终端',
    'cmd.close_split': '关闭分割窗格',
    'cmd.add_project': '添加项目',
    'cmd.open_settings': '打开设置',
    'cmd.category.view': '视图',
    'cmd.category.terminal': '终端',
    'cmd.category.project': '项目',
    'cmd.category.preferences': '偏好设置',
    'cmd.placeholder': '输入命令...',
    'cmd.no_match': '无匹配命令',

    // Settings
    'settings.title': '设置',
    'settings.font_family': '字体',
    'settings.font_family_hint': '例如：Cascadia Code, Fira Code, monospace',
    'settings.font_size': '字号',
    'settings.background': '背景色',
    'settings.foreground': '前景色',
    'settings.shell_path': 'Shell 路径',
    'settings.shell_hint': '留空则自动检测',
    'settings.language': '语言',
    'settings.language_hint': '重启后生效',

    // Keybindings
    'keybind.section_title': '快捷键',
    'keybind.command_palette': '命令面板',
    'keybind.split_pane': '分屏/取消分屏',
    'keybind.press_keys': '按下按键...',
    'keybind.reset': '恢复默认',

    // Terminal search
    'search.placeholder': '在终端中查找...',

    // File tree
    'files.title': '文件',
    'files.loading': '加载中...',
    'files.empty': '空目录',
    'files.ctx.open': '打开',
    'files.ctx.open_in_explorer': '在资源管理器中打开',
    'files.ctx.reveal_in_explorer': '在资源管理器中显示',
    'files.ctx.copy_path': '复制路径',
    'files.ctx.copy_rel_path': '复制相对路径',

    // Tab bar
    'tab.new_hint': '新建终端',

    // Error recovery
    'error.title': '出了点问题',
    'error.unknown': '发生了未知错误',
    'error.retry': '重试',
    'error.reload': '重新加载应用',
    'error.crash_detected': 'DevTerm 上次意外关闭了。',
    'error.crash_view': '查看详情',
    'error.crash_dismiss': '忽略',

    // Clipboard / Context menu
    'clipboard.copy': '复制',
    'clipboard.paste': '粘贴',
    'clipboard.select_all': '全选',
    'settings.select_to_copy': '选中即复制',
    'settings.select_to_copy_hint': '选中文本时自动复制到剪贴板',
    'settings.theme': '主题',
    'settings.theme_hint': '选择终端配色主题',

    // Session launcher
    'launcher.new_session': '新建会话',
    'launcher.plain_shell': '普通 Shell',
    'launcher.claude_code': 'Claude Code',
    'launcher.codex': 'Codex',
    'launcher.custom_template': '自定义...',

    // Session templates
    'templates.title': '启动模板',
    'templates.hint': '创建可复用的会话启动器，预配置启动命令',
    'templates.new': '新建模板',
    'templates.name': '名称',
    'templates.command': '命令',
    'templates.icon': '图标',
    'templates.name_placeholder': '例如：Dev Server',
    'templates.command_placeholder': '例如：npm run dev',
    'templates.icon_placeholder': '例如：🔧',
    'templates.save': '保存',
    'templates.cancel': '取消',
    'templates.edit': '编辑',
    'templates.delete': '删除',

    // Quick command panel (Ctrl+K)
    'quickcmd.title': '快速命令',
    'quickcmd.start_claude': '启动 Claude Code',
    'quickcmd.start_claude_hint': '输入: claude',
    'quickcmd.start_codex': '启动 Codex',
    'quickcmd.start_codex_hint': '输入: codex',
    'quickcmd.resume': '恢复已保存的会话',
    'quickcmd.resume_hint': '输入已保存的恢复命令',
    'quickcmd.remember': '记住当前会话',
    'quickcmd.remember_hint': '为此终端保存恢复命令',
    'quickcmd.clear': '清除所有已保存的会话',
    'quickcmd.help': '帮助',
    'quickcmd.remember_prompt': '粘贴恢复命令 (例如: claude -r abc123):',
    'quickcmd.remember_saved': '此会话的恢复命令已保存。',
    'quickcmd.remember_cleared': '所有已保存的会话已清除。',

    // Help text
    'quickcmd.help_text': '快速命令让你可以启动 CLI AI 工具并恢复之前的会话。\n\n• 启动 Claude Code / Codex — 在终端中输入对应命令\n• 记住 — 为此终端保存恢复命令 (如 "claude -r abc123")\n• 恢复 — 重放已保存的恢复命令\n• 使用 ↑↓ 导航，Enter 选择，Esc 关闭',
  },
};

let _locale: Locale = 'en';
let _ready = false;
let _pending: Array<() => void> = [];

export const locale = $state({ value: 'en' as Locale });

export function t(key: string, params?: Record<string, string | number>): string {
  // Reading locale.value to establish reactivity
  const _ = locale.value;
  const dict = dictionaries[_] ?? dictionaries.en;
  let text = dict[key] ?? dictionaries.en[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, String(v));
    }
  }
  return text;
}

export function setLocale(loc: Locale) {
  locale.value = loc;
  invoke('set_setting', { key: 'locale', value: loc }).catch(() => {});
}

export async function initLocale(): Promise<void> {
  try {
    const saved = await invoke<string | null>('get_setting', { key: 'locale' });
    if (saved === 'zh' || saved === 'en') {
      locale.value = saved;
    }
  } catch (_) {}
  _ready = true;
  for (const cb of _pending) cb();
  _pending = [];
}
