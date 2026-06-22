import { invoke } from '@tauri-apps/api/core';

export interface KeyAction {
  id: string;
  labelKey: string; // i18n key for display name
  defaultBinding: string; // internal format: "code:Ctrl+Shift"
}

// All bindable actions
export const KEY_ACTIONS: KeyAction[] = [
  { id: 'toggle-sidebar', labelKey: 'cmd.toggle_sidebar', defaultBinding: 'KeyB:Ctrl' },
  { id: 'command-palette', labelKey: 'keybind.command_palette', defaultBinding: 'KeyP:Ctrl+Shift' },
  { id: 'file-panel', labelKey: 'cmd.toggle_files', defaultBinding: 'KeyE:Ctrl+Shift' },
  { id: 'terminal-search', labelKey: 'cmd.find_in_terminal', defaultBinding: 'KeyF:Ctrl+Shift' },
  { id: 'split-pane', labelKey: 'keybind.split_pane', defaultBinding: 'Backslash:Ctrl' },
  { id: 'next-tab', labelKey: 'cmd.next_tab', defaultBinding: 'Tab:Ctrl' },
  { id: 'prev-tab', labelKey: 'cmd.prev_tab', defaultBinding: 'Tab:Ctrl+Shift' },
  { id: 'settings', labelKey: 'cmd.open_settings', defaultBinding: 'Comma:Ctrl' },
];

type BindingMap = Record<string, string>; // actionId → internal binding string

let _bindingsLoaded = false;
let _pendingLoad: Array<() => void> = [];
let _bindings: BindingMap = {};

export const keybindings = $state({ map: _bindings });
export const recordingId = $state({ value: null as string | null });

function bindingKey(actionId: string): string {
  return `keybinding.${actionId}`;
}

export async function loadKeybindings(): Promise<void> {
  if (_bindingsLoaded) return;
  _bindingsLoaded = true;
  for (const action of KEY_ACTIONS) {
    try {
      const saved = await invoke<string | null>('get_setting', { key: bindingKey(action.id) });
      _bindings[action.id] = saved || action.defaultBinding;
    } catch (_) {
      _bindings[action.id] = action.defaultBinding;
    }
  }
  keybindings.map = { ..._bindings };
  for (const cb of _pendingLoad) cb();
  _pendingLoad = [];
}

function onBindingsLoaded(cb: () => void) {
  if (_bindingsLoaded) cb();
  else _pendingLoad.push(cb);
}

export function getBinding(actionId: string): string {
  return keybindings.map[actionId] || KEY_ACTIONS.find(a => a.id === actionId)?.defaultBinding || '';
}

export async function setBinding(actionId: string, binding: string): Promise<void> {
  _bindings[actionId] = binding;
  keybindings.map = { ..._bindings };
  await invoke('set_setting', { key: bindingKey(actionId), value: binding }).catch(() => {});
}

export async function resetBinding(actionId: string): Promise<void> {
  const def = KEY_ACTIONS.find(a => a.id === actionId)?.defaultBinding || '';
  await setBinding(actionId, def);
}

// ── KeyboardEvent ↔ internal format ──

/** Convert a KeyboardEvent to internal binding string */
export function eventToBinding(e: KeyboardEvent): string | null {
  // Ignore modifier-only presses
  if (['ControlLeft', 'ControlRight', 'ShiftLeft', 'ShiftRight',
       'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight',
       'OSLeft', 'OSRight'].includes(e.code)) {
    return null;
  }
  const mods: string[] = [];
  if (e.ctrlKey || e.metaKey) mods.push('Ctrl');
  if (e.shiftKey) mods.push('Shift');
  if (e.altKey) mods.push('Alt');
  return `${e.code}:${mods.join('+')}`;
}

/** Check if a KeyboardEvent matches a stored binding */
export function matchesBinding(binding: string, e: KeyboardEvent): boolean {
  return eventToBinding(e) === binding;
}

// ── Display ──

const CODE_DISPLAY: Record<string, string> = {
  'Backslash': '\\', 'Slash': '/', 'Comma': ',', 'Period': '.',
  'Backquote': '`', 'Semicolon': ';', 'Quote': "'",
  'BracketLeft': '[', 'BracketRight': ']', 'Minus': '-', 'Equal': '=',
  'Space': 'Space', 'Tab': 'Tab', 'Escape': 'Esc', 'Enter': 'Enter',
  'Backspace': 'Backspace', 'Delete': 'Delete',
  'ArrowUp': '↑', 'ArrowDown': '↓', 'ArrowLeft': '←', 'ArrowRight': '→',
  'Home': 'Home', 'End': 'End', 'PageUp': 'PgUp', 'PageDown': 'PgDn',
  'Insert': 'Ins', 'CapsLock': 'Caps', 'NumLock': 'NumLk',
};

function codeToDisplay(code: string): string {
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  return CODE_DISPLAY[code] ?? code;
}

/** Convert internal binding "code:Mod1+Mod2" to display string "Mod1+Mod2+Key" */
export function bindingToDisplay(binding: string): string {
  const colon = binding.lastIndexOf(':');
  if (colon < 0) return binding;
  const code = binding.slice(0, colon);
  const mods = binding.slice(colon + 1);
  const key = codeToDisplay(code);
  return mods ? `${mods}+${key}` : key;
}

// ── Handler ──

/** Handle a keyboard event using registered bindings. Returns the matched action id or null. */
export function handleKeyEvent(e: KeyboardEvent): string | null {
  for (const action of KEY_ACTIONS) {
    const b = getBinding(action.id);
    if (b && matchesBinding(b, e)) {
      return action.id;
    }
  }
  return null;
}
