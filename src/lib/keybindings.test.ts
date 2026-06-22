import { describe, it, expect } from 'vitest';
import {
  KEY_ACTIONS,
  eventToBinding,
  bindingToDisplay,
  matchesBinding,
} from './keybindings.svelte';

function makeKeyEvent(opts: {
  code: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  metaKey?: boolean;
}): KeyboardEvent {
  return new KeyboardEvent('keydown', {
    code: opts.code,
    ctrlKey: opts.ctrlKey ?? false,
    shiftKey: opts.shiftKey ?? false,
    altKey: opts.altKey ?? false,
    metaKey: opts.metaKey ?? false,
    bubbles: true,
  });
}

// ── KEY_ACTIONS ──

describe('KEY_ACTIONS', () => {
  it('has all expected action ids', () => {
    const ids = KEY_ACTIONS.map(a => a.id);
    expect(ids).toContain('toggle-sidebar');
    expect(ids).toContain('command-palette');
    expect(ids).toContain('file-panel');
    expect(ids).toContain('terminal-search');
    expect(ids).toContain('split-pane');
    expect(ids).toContain('next-tab');
    expect(ids).toContain('prev-tab');
    expect(ids).toContain('settings');
  });

  it('every action has a defaultBinding in code:Mod format', () => {
    for (const action of KEY_ACTIONS) {
      expect(action.defaultBinding).toMatch(/^\w+:(\w+(\+\w+)*)?$/);
    }
  });
});

// ── eventToBinding ──

describe('eventToBinding', () => {
  it('converts simple Ctrl+Key shortcut', () => {
    const e = makeKeyEvent({ code: 'KeyB', ctrlKey: true });
    expect(eventToBinding(e)).toBe('KeyB:Ctrl');
  });

  it('converts Ctrl+Shift+Key shortcut', () => {
    const e = makeKeyEvent({ code: 'KeyP', ctrlKey: true, shiftKey: true });
    expect(eventToBinding(e)).toBe('KeyP:Ctrl+Shift');
  });

  it('converts key with no modifiers', () => {
    const e = makeKeyEvent({ code: 'F1' });
    expect(eventToBinding(e)).toBe('F1:');
  });

  it('converts Alt modifier', () => {
    const e = makeKeyEvent({ code: 'KeyX', altKey: true });
    expect(eventToBinding(e)).toBe('KeyX:Alt');
  });

  it('converts Ctrl+Alt combination', () => {
    const e = makeKeyEvent({ code: 'KeyD', ctrlKey: true, altKey: true });
    expect(eventToBinding(e)).toBe('KeyD:Ctrl+Alt');
  });

  it('treats Meta key as Ctrl', () => {
    const e = makeKeyEvent({ code: 'KeyB', metaKey: true });
    expect(eventToBinding(e)).toBe('KeyB:Ctrl');
  });

  it('returns null for modifier-only keypress (ControlLeft)', () => {
    const e = makeKeyEvent({ code: 'ControlLeft', ctrlKey: true });
    expect(eventToBinding(e)).toBeNull();
  });

  it('returns null for modifier-only keypress (ShiftLeft)', () => {
    const e = makeKeyEvent({ code: 'ShiftLeft', shiftKey: true });
    expect(eventToBinding(e)).toBeNull();
  });

  it('returns null for AltLeft', () => {
    const e = makeKeyEvent({ code: 'AltLeft', altKey: true });
    expect(eventToBinding(e)).toBeNull();
  });

  it('returns null for MetaLeft', () => {
    const e = makeKeyEvent({ code: 'MetaLeft', metaKey: true });
    expect(eventToBinding(e)).toBeNull();
  });

  it('uses code not key (layout-independent)', () => {
    const e = makeKeyEvent({ code: 'Backslash', ctrlKey: true });
    expect(eventToBinding(e)).toBe('Backslash:Ctrl');
  });
});

// ── matchesBinding ──

describe('matchesBinding', () => {
  it('matches exact same binding', () => {
    const e = makeKeyEvent({ code: 'KeyB', ctrlKey: true });
    expect(matchesBinding('KeyB:Ctrl', e)).toBe(true);
  });

  it('does not match different key', () => {
    const e = makeKeyEvent({ code: 'KeyB', ctrlKey: true });
    expect(matchesBinding('KeyP:Ctrl', e)).toBe(false);
  });

  it('does not match different modifiers', () => {
    const e = makeKeyEvent({ code: 'KeyB', ctrlKey: true });
    expect(matchesBinding('KeyB:Ctrl+Shift', e)).toBe(false);
  });

  it('matches with Meta treated as Ctrl', () => {
    const e = makeKeyEvent({ code: 'KeyP', metaKey: true });
    expect(matchesBinding('KeyP:Ctrl', e)).toBe(true);
  });

  it('matches Ctrl+Shift combo', () => {
    const e = makeKeyEvent({ code: 'Tab', ctrlKey: true, shiftKey: true });
    expect(matchesBinding('Tab:Ctrl+Shift', e)).toBe(true);
  });
});

// ── bindingToDisplay ──

describe('bindingToDisplay', () => {
  it('formats "KeyB:Ctrl" as "Ctrl+B"', () => {
    expect(bindingToDisplay('KeyB:Ctrl')).toBe('Ctrl+B');
  });

  it('formats "KeyP:Ctrl+Shift" as "Ctrl+Shift+P"', () => {
    expect(bindingToDisplay('KeyP:Ctrl+Shift')).toBe('Ctrl+Shift+P');
  });

  it('formats "Backslash:Ctrl" as "Ctrl+\\"', () => {
    expect(bindingToDisplay('Backslash:Ctrl')).toBe('Ctrl+\\');
  });

  it('formats "Comma:Ctrl" as "Ctrl+,"', () => {
    expect(bindingToDisplay('Comma:Ctrl')).toBe('Ctrl+,');
  });

  it('formats "Tab:Ctrl" as "Ctrl+Tab"', () => {
    expect(bindingToDisplay('Tab:Ctrl')).toBe('Ctrl+Tab');
  });

  it('formats "Slash:Ctrl" as "Ctrl+/"', () => {
    expect(bindingToDisplay('Slash:Ctrl')).toBe('Ctrl+/');
  });

  it('formats "Period:Ctrl" as "Ctrl+."', () => {
    expect(bindingToDisplay('Period:Ctrl')).toBe('Ctrl+.');
  });

  it('formats arrow keys with display symbols', () => {
    expect(bindingToDisplay('ArrowUp:Ctrl')).toBe('Ctrl+↑');
    expect(bindingToDisplay('ArrowDown:Ctrl')).toBe('Ctrl+↓');
    expect(bindingToDisplay('ArrowLeft:Ctrl')).toBe('Ctrl+←');
    expect(bindingToDisplay('ArrowRight:Ctrl')).toBe('Ctrl+→');
  });

  it('handles binding with no colon separator', () => {
    expect(bindingToDisplay('F1')).toBe('F1');
  });

  it('strips "Key" prefix from letter keys', () => {
    expect(bindingToDisplay('KeyA:')).toBe('A');
  });

  it('strips "Digit" prefix from number keys', () => {
    expect(bindingToDisplay('Digit1:Ctrl')).toBe('Ctrl+1');
  });

  it('displays F-keys as-is', () => {
    expect(bindingToDisplay('F11:Ctrl')).toBe('Ctrl+F11');
  });

  it('displays Space', () => {
    expect(bindingToDisplay('Space:Ctrl')).toBe('Ctrl+Space');
  });

  it('displays Escape as Esc', () => {
    expect(bindingToDisplay('Escape:Ctrl')).toBe('Ctrl+Esc');
  });
});
