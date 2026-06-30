<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { t } from './i18n.svelte';

  interface Props {
    sessionId: string;
    bridge: { write: (data: string) => void };
    onClose: () => void;
    onAction?: () => void;
  }

  let { sessionId, bridge, onClose, onAction }: Props = $props();

  let selectedIndex = $state(0);
  let mode = $state<'menu' | 'remember' | 'help'>('menu');
  let resumeInput = $state('');
  let resumeCommands = $state<Record<string, string>>({});
  let message = $state<string | null>(null);
  let inputRef = $state<HTMLInputElement | null>(null);

  let menuItems = $derived.by(() => {
    const saved = resumeCommands[sessionId];
    const hasAny = Object.keys(resumeCommands).length > 0;
    return [
      { id: 'claude',     label: t('quickcmd.start_claude'), hint: t('quickcmd.start_claude_hint'), disabled: false },
      { id: 'codex',      label: t('quickcmd.start_codex'),  hint: t('quickcmd.start_codex_hint'),  disabled: false },
      { id: 'resume',     label: t('quickcmd.resume'),        hint: saved || '',                       disabled: !saved },
      { id: 'remember',   label: t('quickcmd.remember'),      hint: t('quickcmd.remember_hint'),      disabled: false },
      { id: 'clear',      label: t('quickcmd.clear'),         hint: '',                                 disabled: !hasAny },
      { id: 'help',       label: t('quickcmd.help'),          hint: '',                                 disabled: false },
    ];
  });

  // Load resume commands from settings on mount
  $effect(() => {
    (async () => {
      try {
        const raw = await invoke<string | null>('get_setting', { key: 'resume_commands' });
        if (raw) {
          resumeCommands = JSON.parse(raw);
        }
      } catch (_) {}
    })();
  });

  async function saveResumeCommands() {
    await invoke('set_setting', { key: 'resume_commands', value: JSON.stringify(resumeCommands) }).catch(() => {});
  }

  function executeAction(id: string) {
    onAction?.();
    switch (id) {
      case 'claude':
        bridge.write('claude\r');
        onClose();
        break;
      case 'codex':
        bridge.write('codex\r');
        onClose();
        break;
      case 'resume': {
        const cmd = resumeCommands[sessionId];
        if (cmd) {
          bridge.write(cmd + '\r');
        }
        onClose();
        break;
      }
      case 'remember':
        mode = 'remember';
        resumeInput = '';
        selectedIndex = -1;
        // Focus input after DOM update
        setTimeout(() => inputRef?.focus(), 0);
        break;
      case 'clear':
        resumeCommands = {};
        saveResumeCommands();
        message = t('quickcmd.remember_cleared');
        setTimeout(() => { message = null; }, 2000);
        break;
      case 'help':
        mode = 'help';
        selectedIndex = -1;
        break;
    }
  }

  function saveResume() {
    const cmd = resumeInput.trim();
    if (!cmd) return;
    resumeCommands = { ...resumeCommands, [sessionId]: cmd };
    saveResumeCommands();
    message = t('quickcmd.remember_saved');
    mode = 'menu';
    selectedIndex = 3; // highlight remember item
    setTimeout(() => { message = null; }, 2000);
  }

  function cancelRemember() {
    mode = 'menu';
    resumeInput = '';
    selectedIndex = 3;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (mode === 'remember') {
      if (e.key === 'Escape') {
        e.preventDefault();
        cancelRemember();
      } else if (e.key === 'Enter') {
        e.preventDefault();
        saveResume();
      }
      return;
    }

    if (mode === 'help') {
      if (e.key === 'Escape' || e.key === 'Enter') {
        e.preventDefault();
        mode = 'menu';
        selectedIndex = 5;
      }
      return;
    }

    // menu mode
    const items = menuItems.filter(it => !it.disabled);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % items.length;
      // Map back to real index
      for (let i = 0, j = 0; i < menuItems.length; i++) {
        if (!menuItems[i].disabled) {
          if (j === selectedIndex) { selectedIndex = i; break; }
          j++;
        }
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      let newIdx = selectedIndex - 1;
      if (newIdx < 0) newIdx = items.length - 1;
      // Map back to real index
      for (let i = menuItems.length - 1, j = items.length - 1; i >= 0; i--) {
        if (!menuItems[i].disabled) {
          if (j === newIdx) { selectedIndex = i; break; }
          j--;
        }
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = menuItems[selectedIndex];
      if (item && !item.disabled) {
        executeAction(item.id);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="qcmd-overlay" onclick={onClose} role="dialog">
  <div class="qcmd-panel" onclick={(e: MouseEvent) => e.stopPropagation()} role="presentation">
    {#if mode === 'help'}
      <div class="qcmd-header">{t('quickcmd.help')}</div>
      <div class="qcmd-help-text">{t('quickcmd.help_text')}</div>
      <div class="qcmd-footer-hint">Enter / Esc — {t('launcher.cancel')}</div>
    {:else if mode === 'remember'}
      <div class="qcmd-header">{t('quickcmd.remember')}</div>
      <div class="qcmd-remember-row">
        <span class="qcmd-remember-label">{t('quickcmd.remember_prompt')}</span>
        <input
          bind:this={inputRef}
          type="text"
          class="qcmd-input"
          bind:value={resumeInput}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Escape') { e.preventDefault(); cancelRemember(); } else if (e.key === 'Enter') { e.preventDefault(); saveResume(); } }}
          placeholder="claude -r abc123"
        />
      </div>
      <div class="qcmd-footer-hint">Enter — {t('templates.save')} | Esc — {t('launcher.cancel')}</div>
    {:else}
      <div class="qcmd-header">{t('quickcmd.title')}</div>
      <div class="qcmd-items">
        {#each menuItems as item, i}
          <button
            class="qcmd-item"
            class:selected={selectedIndex === i}
            class:disabled={item.disabled}
            disabled={item.disabled}
            onclick={() => executeAction(item.id)}
            onmouseenter={() => { if (!item.disabled) selectedIndex = i; }}
          >
            <span class="qcmd-item-label">{item.label}</span>
            {#if item.hint}
              <span class="qcmd-item-hint">{item.hint}</span>
            {/if}
          </button>
        {/each}
      </div>
      <div class="qcmd-footer-hint">↑↓ Navigate | Enter — OK | Esc — Cancel</div>
    {/if}

    {#if message}
      <div class="qcmd-message">{message}</div>
    {/if}
  </div>
</div>

<style>
  .qcmd-overlay {
    position: fixed;
    inset: 0;
    z-index: 9990;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding-bottom: 40px;
    background: rgba(0, 0, 0, 0.3);
  }
  .qcmd-panel {
    background: #1e1e1e;
    border: 1px solid #454545;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
    min-width: 420px;
    max-width: 560px;
    overflow: hidden;
  }
  .qcmd-header {
    padding: 8px 14px;
    font-size: 12px;
    font-weight: 600;
    color: #888;
    border-bottom: 1px solid #333;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .qcmd-items {
    padding: 4px;
  }
  .qcmd-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 12px;
    background: none;
    border: none;
    color: #ccc;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
    gap: 16px;
  }
  .qcmd-item:hover,
  .qcmd-item.selected {
    background: #094771;
    color: #fff;
  }
  .qcmd-item.disabled {
    color: #555;
    cursor: default;
  }
  .qcmd-item.disabled:hover,
  .qcmd-item.disabled.selected {
    background: none;
    color: #555;
  }
  .qcmd-item-label {
    white-space: nowrap;
  }
  .qcmd-item-hint {
    font-size: 11px;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 240px;
  }
  .qcmd-item:hover .qcmd-item-hint,
  .qcmd-item.selected .qcmd-item-hint {
    color: #999;
  }
  .qcmd-item.disabled .qcmd-item-hint {
    color: #444;
  }
  .qcmd-footer-hint {
    padding: 4px 14px 6px;
    font-size: 10px;
    color: #555;
    border-top: 1px solid #2a2a2a;
  }
  .qcmd-help-text {
    padding: 12px 14px;
    font-size: 12px;
    color: #aaa;
    line-height: 1.6;
    white-space: pre-line;
  }
  .qcmd-remember-row {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .qcmd-remember-label {
    font-size: 12px;
    color: #aaa;
  }
  .qcmd-input {
    width: 100%;
    padding: 6px 10px;
    background: #2d2d30;
    border: 1px solid #454545;
    border-radius: 4px;
    color: #fff;
    font-size: 13px;
    font-family: inherit;
    outline: none;
  }
  .qcmd-input:focus {
    border-color: #007acc;
  }
  .qcmd-message {
    padding: 6px 14px;
    font-size: 11px;
    color: #4ec9b0;
    border-top: 1px solid #333;
  }
</style>
