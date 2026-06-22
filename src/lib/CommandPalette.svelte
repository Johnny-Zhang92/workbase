<script lang="ts">
  import { appState } from './stores.svelte';
  import { t } from './i18n.svelte';

  interface Command {
    id: string;
    label: string;
    category: string;
    action: () => void;
  }

  let { onClose, onNewTab, toggleSearch, onOpenSettings }: {
    onClose: () => void;
    onNewTab: () => void;
    toggleSearch: () => void;
    onOpenSettings: () => void;
  } = $props();

  let query = $state('');
  let selectedIdx = $state(0);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);

  const allActions: Array<{ id: string; labelKey: string; catKey: string; action: () => void }> = [
    { id: 'toggle-sidebar', labelKey: 'cmd.toggle_sidebar', catKey: 'cmd.category.view', action: () => { appState.sidebarVisible = !appState.sidebarVisible; onClose(); } },
    { id: 'toggle-files', labelKey: 'cmd.toggle_files', catKey: 'cmd.category.view', action: () => { appState.filePanelVisible = !appState.filePanelVisible; onClose(); } },
    { id: 'terminal-search', labelKey: 'cmd.find_in_terminal', catKey: 'cmd.category.view', action: () => { onClose(); toggleSearch(); } },
    { id: 'new-tab', labelKey: 'cmd.new_tab', catKey: 'cmd.category.terminal', action: () => { onClose(); onNewTab(); } },
    { id: 'close-tab', labelKey: 'cmd.close_tab', catKey: 'cmd.category.terminal', action: () => { appState.closeTabSignal = appState.activeSessionId || null; onClose(); } },
    { id: 'next-tab', labelKey: 'cmd.next_tab', catKey: 'cmd.category.terminal', action: () => { appState.paletteAction = 'next-tab'; onClose(); } },
    { id: 'prev-tab', labelKey: 'cmd.prev_tab', catKey: 'cmd.category.terminal', action: () => { appState.paletteAction = 'prev-tab'; onClose(); } },
    { id: 'split-v', labelKey: 'cmd.split_v', catKey: 'cmd.category.terminal', action: () => { appState.paletteAction = 'split-v'; onClose(); } },
    { id: 'split-h', labelKey: 'cmd.split_h', catKey: 'cmd.category.terminal', action: () => { appState.paletteAction = 'split-h'; onClose(); } },
    { id: 'close-split', labelKey: 'cmd.close_split', catKey: 'cmd.category.terminal', action: () => { appState.paletteAction = 'close-split'; onClose(); } },
    { id: 'add-project', labelKey: 'cmd.add_project', catKey: 'cmd.category.project', action: () => { appState.paletteAction = 'add-project'; onClose(); } },
    { id: 'open-settings', labelKey: 'cmd.open_settings', catKey: 'cmd.category.preferences', action: () => { onClose(); onOpenSettings(); } },
  ];

  let allCommands = $derived(allActions.map(a => ({
    id: a.id,
    label: t(a.labelKey),
    category: t(a.catKey),
    action: a.action,
  })));

  let filtered = $derived.by(() => {
    if (!query.trim()) return allCommands;
    const q = query.toLowerCase();
    return allCommands.filter(c =>
      c.label.toLowerCase().includes(q) || c.category.toLowerCase().includes(q)
    );
  });

  $effect(() => {
    selectedIdx = 0;
  });

  $effect(() => {
    inputEl?.focus();
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIdx = Math.min(selectedIdx + 1, filtered.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIdx = Math.max(selectedIdx - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      filtered[selectedIdx]?.action();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }

  function execute(idx: number) {
    filtered[idx]?.action();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="cp-overlay" onclick={onClose} onkeydown={onKeydown} role="presentation">
  <div class="cp-dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown} role="dialog">
    <input
      bind:this={inputEl}
      type="text"
      class="cp-input"
      placeholder={t('cmd.placeholder')}
      bind:value={query}
      onkeydown={onKeydown}
    />
    <div class="cp-list">
      {#each filtered as cmd, idx (cmd.id)}
        <button
          class="cp-item"
          class:selected={idx === selectedIdx}
          onclick={() => execute(idx)}
        >
          <span class="cp-label">{cmd.label}</span>
          <span class="cp-cat">{cmd.category}</span>
        </button>
      {/each}
      {#if filtered.length === 0}
        <div class="cp-empty">{t('cmd.no_match')}</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .cp-overlay {
    position: fixed; inset: 0; z-index: 300;
    background: rgba(0, 0, 0, 0.5);
    display: flex; justify-content: center; padding-top: 15vh;
  }
  .cp-dialog {
    width: 480px; max-height: 50vh;
    background: #252526; border: 1px solid #454545;
    border-radius: 8px; box-shadow: 0 8px 32px rgba(0,0,0,0.6);
    display: flex; flex-direction: column; overflow: hidden;
  }
  .cp-input {
    background: #1e1e1e; border: none; border-bottom: 1px solid #3c3c3c;
    color: #ccc; padding: 12px 16px; font-size: 14px;
    outline: none; font-family: inherit;
  }
  .cp-list { flex: 1; overflow-y: auto; padding: 4px; }
  .cp-item {
    display: flex; align-items: center; gap: 8px;
    width: 100%; padding: 6px 12px;
    background: none; border: none; color: #ccc;
    cursor: pointer; border-radius: 4px;
    font-size: 13px; text-align: left;
  }
  .cp-item:hover, .cp-item.selected { background: #094771; color: #fff; }
  .cp-label { flex: 1; }
  .cp-cat { font-size: 10px; color: #888; text-transform: uppercase; }
  .cp-item.selected .cp-cat { color: rgba(255, 255, 255, 0.7); }
  .cp-empty { padding: 12px 16px; color: #666; font-style: italic; font-size: 13px; }
</style>
