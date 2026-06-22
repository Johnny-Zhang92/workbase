import type { Project, Session, SessionTemplate } from './types';

export const appState = $state({
  projects: [] as Project[],
  activeProjectId: null as number | null,
  sessions: [] as Session[],
  templates: [] as SessionTemplate[],
  tabOpening: false,
  activeSessionId: null as string | null,
  sidebarVisible: true,
  sidebarWidth: 280,
  statusText: 'DevTerm v0.1.0 — Ready',
  closeTabSignal: null as string | null,
  closeAllTabs: false,
  filePanelVisible: true,
  filePanelWidth: 240,
  gitBranch: '' as string,
  gitFiles: {} as Record<string, string>,
  paletteVisible: false,
  paletteAction: null as string | null,
  fileTreeVersion: 0,
});
