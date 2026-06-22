<script lang="ts">
  import { t } from './i18n.svelte';

  interface Tab {
    sessionId: string;
    sessionName: string;
  }

  let { tabs, activeTabId, onSelectTab, onCloseTab, onNewTab, onReorder }: {
    tabs: Tab[];
    activeTabId: string | null;
    onSelectTab: (id: string) => void;
    onCloseTab: (id: string) => void;
    onNewTab: () => void;
    onReorder: (fromIndex: number, toIndex: number) => void;
  } = $props();

  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let dropBefore = $state(false);

  function onDragStart(e: DragEvent, idx: number) {
    dragIndex = idx;
    e.dataTransfer!.effectAllowed = 'move';
    e.dataTransfer!.setData('text/plain', String(idx));
  }

  function onDragOver(e: DragEvent, idx: number) {
    e.preventDefault();
    if (dragIndex === null || dragIndex === idx) {
      dragOverIndex = null;
      return;
    }
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dropBefore = e.clientX < rect.left + rect.width / 2;
    dragOverIndex = idx;
  }

  function onDrop(e: DragEvent, idx: number) {
    e.preventDefault();
    if (dragIndex === null || dragIndex === idx) return;
    const from = dragIndex;
    const to = dropBefore ? (idx > dragIndex! ? idx - 1 : idx) : (idx > dragIndex! ? idx : idx + 1);
    onReorder(from, to);
    dragIndex = null;
    dragOverIndex = null;
  }

  function onDragEnd() {
    dragIndex = null;
    dragOverIndex = null;
  }

</script>

{#if tabs.length > 0}
  <div class="tab-bar">
    <div class="tab-list" ondragleave={() => { dragOverIndex = null; }}>
      {#each tabs as tab, idx (tab.sessionId)}
        <button
          class="tab"
          class:active={tab.sessionId === activeTabId}
          class:dragging={dragIndex === idx}
          class:drag-over-before={dragOverIndex === idx && dropBefore}
          class:drag-over-after={dragOverIndex === idx && !dropBefore}
          draggable="true"
          onclick={() => onSelectTab(tab.sessionId)}
          ondragstart={(e) => onDragStart(e, idx)}
          ondragover={(e) => onDragOver(e, idx)}
          ondrop={(e) => onDrop(e, idx)}
          ondragend={onDragEnd}
          title={tab.sessionName}
        >
          <span class="tab-label">{tab.sessionName}</span>
          <span
            class="tab-close"
            onclick={(e) => { e.stopPropagation(); onCloseTab(tab.sessionId); }}
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); onCloseTab(tab.sessionId); } }}
          >&times;</span>
        </button>
      {/each}
    </div>
    <button class="tab-new" onclick={onNewTab} title={t('tab.new_hint')}>+</button>
  </div>
{/if}

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    background: #252526;
    border-bottom: 1px solid #3c3c3c;
    height: 35px;
    flex-shrink: 0;
  }
  .tab-list {
    display: flex;
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    height: 100%;
  }
  .tab-list::-webkit-scrollbar { height: 2px; }
  .tab-list::-webkit-scrollbar-thumb { background: #424242; }
  .tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    height: 100%;
    background: #2d2d2d;
    border: none;
    border-right: 1px solid #252526;
    color: #999;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    min-width: 80px;
    max-width: 160px;
    position: relative;
  }
  .tab:hover { background: #37373d; }
  .tab.active {
    background: #1e1e1e;
    color: #fff;
    border-bottom: 2px solid #007acc;
  }
  .tab.dragging {
    opacity: 0.4;
  }
  .tab.drag-over-before::before {
    content: '';
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: #007acc;
    z-index: 1;
  }
  .tab.drag-over-after::after {
    content: '';
    position: absolute;
    right: 0;
    top: 4px;
    bottom: 4px;
    width: 2px;
    background: #007acc;
    z-index: 1;
  }
  .tab-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tab-close {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    font-size: 14px;
    visibility: hidden;
    flex-shrink: 0;
  }
  .tab:hover .tab-close,
  .tab.active .tab-close { visibility: visible; }
  .tab-close:hover { background: #5a1d1d; color: #f44747; }
  .tab-new {
    background: none;
    border: none;
    color: #888;
    font-size: 18px;
    cursor: pointer;
    padding: 0 10px;
    height: 100%;
    flex-shrink: 0;
  }
  .tab-new:hover { color: #fff; background: #37373d; }
</style>
