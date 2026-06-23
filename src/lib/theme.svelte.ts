import { invoke } from '@tauri-apps/api/core';
import type { Terminal } from '@xterm/xterm';

export interface TerminalTheme {
  name: string;
  label: string;
  background: string;
  foreground: string;
  cursor: string;
  cursorAccent: string;
  selectionBackground: string;
  ansi: string[];
}

export const PRESETS: TerminalTheme[] = [
  {
    name: 'workbase-dark',
    label: 'Workbase Dark',
    background: '#1e1e1e',
    foreground: '#d4d4d4',
    cursor: '#ffffff',
    cursorAccent: '#1e1e1e',
    selectionBackground: '#264f78',
    ansi: [
      '#000000', '#cd3131', '#0dbc79', '#e5e510',
      '#2472c8', '#bc3fbc', '#11a8cd', '#e5e5e5',
      '#666666', '#f14c4c', '#23d18b', '#f5f543',
      '#3b8eea', '#d670d6', '#29b8db', '#ffffff',
    ],
  },
  {
    name: 'dracula',
    label: 'Dracula',
    background: '#282a36',
    foreground: '#f8f8f2',
    cursor: '#f8f8f2',
    cursorAccent: '#282a36',
    selectionBackground: '#44475a',
    ansi: [
      '#21222c', '#ff5555', '#50fa7b', '#f1fa8c',
      '#bd93f9', '#ff79c6', '#8be9fd', '#f8f8f2',
      '#6272a4', '#ff6e6e', '#69ff94', '#ffffa5',
      '#d6acff', '#ff92df', '#a4ffff', '#ffffff',
    ],
  },
  {
    name: 'nord',
    label: 'Nord',
    background: '#2e3440',
    foreground: '#d8dee9',
    cursor: '#d8dee9',
    cursorAccent: '#2e3440',
    selectionBackground: '#434c5e',
    ansi: [
      '#3b4252', '#bf616a', '#a3be8c', '#ebcb8b',
      '#81a1c1', '#b48ead', '#88c0d0', '#e5e9f0',
      '#4c566a', '#bf616a', '#a3be8c', '#ebcb8b',
      '#81a1c1', '#b48ead', '#8fbcbb', '#eceff4',
    ],
  },
  {
    name: 'monokai',
    label: 'Monokai',
    background: '#272822',
    foreground: '#f8f8f2',
    cursor: '#f8f8f0',
    cursorAccent: '#272822',
    selectionBackground: '#49483e',
    ansi: [
      '#272822', '#f92672', '#a6e22e', '#f4bf75',
      '#66d9ef', '#ae81ff', '#a1efe4', '#f8f8f2',
      '#75715e', '#f92672', '#a6e22e', '#f4bf75',
      '#66d9ef', '#ae81ff', '#a1efe4', '#f9f8f5',
    ],
  },
  {
    name: 'one-dark',
    label: 'One Dark',
    background: '#282c34',
    foreground: '#abb2bf',
    cursor: '#528bff',
    cursorAccent: '#282c34',
    selectionBackground: '#3e4451',
    ansi: [
      '#282c34', '#e06c75', '#98c379', '#e5c07b',
      '#61afef', '#c678dd', '#56b6c2', '#abb2bf',
      '#545862', '#e06c75', '#98c379', '#e5c07b',
      '#61afef', '#c678dd', '#56b6c2', '#c8ccd4',
    ],
  },
  {
    name: 'solarized-dark',
    label: 'Solarized Dark',
    background: '#002b36',
    foreground: '#839496',
    cursor: '#839496',
    cursorAccent: '#002b36',
    selectionBackground: '#073642',
    ansi: [
      '#073642', '#dc322f', '#859900', '#b58900',
      '#268bd2', '#d33682', '#2aa198', '#eee8d5',
      '#002b36', '#cb4b16', '#586e75', '#657b83',
      '#839496', '#6c71c4', '#93a1a1', '#fdf6e3',
    ],
  },
  {
    name: 'github-light',
    label: 'GitHub Light',
    background: '#ffffff',
    foreground: '#24292e',
    cursor: '#24292e',
    cursorAccent: '#ffffff',
    selectionBackground: '#0366d625',
    ansi: [
      '#24292e', '#d73a49', '#28a745', '#dbab09',
      '#0366d6', '#5a32a3', '#0598bc', '#6a737d',
      '#959da5', '#d73a49', '#28a745', '#dbab09',
      '#0366d6', '#5a32a3', '#0598bc', '#d1d5da',
    ],
  },
];

const DEFAULT_THEME = 'workbase-dark';

export function getTheme(name: string): TerminalTheme {
  return PRESETS.find(t => t.name === name) ?? PRESETS[0];
}

export function applyTheme(term: Terminal, theme: TerminalTheme) {
  term.options.theme = {
    background: theme.background,
    foreground: theme.foreground,
    cursor: theme.cursor,
    cursorAccent: theme.cursorAccent,
    selectionBackground: theme.selectionBackground,
    black: theme.ansi[0],
    red: theme.ansi[1],
    green: theme.ansi[2],
    yellow: theme.ansi[3],
    blue: theme.ansi[4],
    magenta: theme.ansi[5],
    cyan: theme.ansi[6],
    white: theme.ansi[7],
    brightBlack: theme.ansi[8],
    brightRed: theme.ansi[9],
    brightGreen: theme.ansi[10],
    brightYellow: theme.ansi[11],
    brightBlue: theme.ansi[12],
    brightMagenta: theme.ansi[13],
    brightCyan: theme.ansi[14],
    brightWhite: theme.ansi[15],
  };
}

export async function loadThemeName(): Promise<string> {
  try {
    const name = await invoke<string | null>('get_setting', { key: 'theme' });
    return name && PRESETS.some(t => t.name === name) ? name : DEFAULT_THEME;
  } catch (_) {
    return DEFAULT_THEME;
  }
}

export async function saveThemeName(name: string) {
  await invoke('set_setting', { key: 'theme', value: name }).catch(() => {});
}
