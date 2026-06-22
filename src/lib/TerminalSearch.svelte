<script lang="ts">
  import type { SearchAddon } from '@xterm/addon-search';
  import type { Terminal } from '@xterm/xterm';
  import { t } from './i18n.svelte';

  let { addon, term, onClose }: { addon: SearchAddon | null; term: Terminal | null; onClose: () => void } = $props();

  let query = $state('');
  let caseSensitive = $state(false);
  let useRegex = $state(false);
  let matchIndex = $state(0);
  let matchTotal = $state(0);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);

  function getSearchOptions(): { regex: boolean; caseSensitive: boolean; incremental: boolean } | undefined {
    if (!query) return undefined;
    return { regex: useRegex, caseSensitive, incremental: true };
  }

  function countMatches(): number {
    if (!term || !query) return 0;
    try {
      const buffer = term.buffer.active;
      const text: string[] = [];
      for (let i = 0; i < buffer.length; i++) {
        const line = buffer.getLine(i);
        if (line) text.push(line.translateToString());
      }
      const content = text.join('\n');
      if (useRegex) {
        let flags = 'g';
        if (!caseSensitive) flags += 'i';
        try {
          const re = new RegExp(query, flags);
          return (content.match(re) || []).length;
        } catch (_) {
          return 0;
        }
      } else {
        const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const flags = caseSensitive ? 'g' : 'gi';
        return (content.match(new RegExp(escaped, flags)) || []).length;
      }
    } catch (_) {
      return 0;
    }
  }

  function doSearch() {
    if (!addon) return;
    if (query) {
      matchTotal = countMatches();
      matchIndex = matchTotal > 0 ? 1 : 0;
      addon.findNext(query, { regex: useRegex, caseSensitive, incremental: false });
    } else {
      matchIndex = 0;
      matchTotal = 0;
      addon.clearDecorations();
    }
  }

  function onInput() {
    if (!addon) return;
    if (query) {
      addon.findNext(query, { regex: useRegex, caseSensitive, incremental: true });
      matchTotal = countMatches();
      matchIndex = matchTotal > 0 ? 1 : 0;
    } else {
      matchIndex = 0;
      matchTotal = 0;
      addon.clearDecorations();
    }
  }

  function prev() {
    if (!addon || !query) return;
    addon.findPrevious(query, { regex: useRegex, caseSensitive });
    matchIndex = matchIndex > 1 ? matchIndex - 1 : matchTotal;
  }

  function next() {
    if (!addon || !query) return;
    addon.findNext(query, { regex: useRegex, caseSensitive });
    matchIndex = matchIndex < matchTotal ? matchIndex + 1 : 1;
  }

  function toggleCase() { caseSensitive = !caseSensitive; doSearch(); }
  function toggleRegex() { useRegex = !useRegex; doSearch(); }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (e.shiftKey) prev(); else next();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }

  $effect(() => {
    inputEl?.focus();
    return () => addon?.clearDecorations();
  });

  let countText = $derived(
    query ? (matchTotal > 0 ? `${matchIndex}/${matchTotal}` : '0') : ''
  );
</script>

<div class="ts-bar">
  <input
    bind:this={inputEl}
    type="text"
    class="ts-input"
    placeholder={t('search.placeholder')}
    bind:value={query}
    oninput={onInput}
    onkeydown={onKeydown}
  />
  <button
    class="ts-toggle"
    class:ts-active={caseSensitive}
    onclick={toggleCase}
    title="Match Case"
  >Aa</button>
  <button
    class="ts-toggle"
    class:ts-active={useRegex}
    onclick={toggleRegex}
    title="Use Regex"
  >.*</button>
  <span class="ts-count">{countText}</span>
  <button class="ts-btn" onclick={prev} title="Previous (Shift+Enter)">
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M7 3L3 7l4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
  </button>
  <button class="ts-btn" onclick={next} title="Next (Enter)">
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M3 11l4-4-4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
  </button>
  <button class="ts-btn ts-close" onclick={onClose} title="Close (Escape)">&times;</button>
</div>

<style>
  .ts-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 8px;
    background: #2d2d2d;
    border-bottom: 1px solid #555;
    flex-shrink: 0;
  }
  .ts-input {
    flex: 1;
    background: #1e1e1e;
    border: 1px solid #3c3c3c;
    color: #ccc;
    padding: 5px 10px;
    font-size: 13px;
    border-radius: 4px;
    outline: none;
    font-family: inherit;
    min-width: 120px;
  }
  .ts-input:focus { border-color: #007acc; }
  .ts-btn {
    background: none;
    border: 1px solid transparent;
    color: #ccc;
    cursor: pointer;
    font-size: 14px;
    padding: 4px 7px;
    border-radius: 3px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .ts-btn:hover { background: #444; border-color: #555; }
  .ts-close { font-size: 18px; padding: 2px 8px; }
  .ts-toggle {
    background: none;
    border: 1px solid #555;
    color: #888;
    cursor: pointer;
    font-size: 12px;
    padding: 3px 7px;
    border-radius: 3px;
    font-family: monospace;
    font-weight: 600;
    min-width: 28px;
  }
  .ts-toggle:hover { background: #444; color: #ccc; }
  .ts-toggle.ts-active {
    background: #007acc;
    border-color: #007acc;
    color: #fff;
  }
  .ts-count {
    font-size: 11px;
    color: #888;
    min-width: 36px;
    text-align: center;
    user-select: none;
    font-family: monospace;
  }
</style>
