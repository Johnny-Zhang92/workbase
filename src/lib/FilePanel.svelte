<script lang="ts">
  import FileTree from './FileTree.svelte';
  import { appState } from './stores.svelte';
  import { t } from './i18n.svelte';

  let rootPath = $derived(
    appState.activeProjectId
      ? (appState.projects.find(p => p.id === appState.activeProjectId)?.root_path ?? '')
      : ''
  );

  let dragStart = $state(0);
  let dragging = $state(false);

  function onDragStart(e: MouseEvent) { dragging = true; dragStart = e.clientX; }
  function onDragMove(e: MouseEvent) {
    if (!dragging) return;
    const diff = dragStart - e.clientX;
    appState.filePanelWidth = Math.max(180, Math.min(500, appState.filePanelWidth + diff));
    dragStart = e.clientX;
  }
  function onDragEnd() { dragging = false; }
</script>

<svelte:window on:mousemove={onDragMove} on:mouseup={onDragEnd} />

{#if rootPath && appState.filePanelVisible}
  <div class="fp-resizer" role="separator" onmousedown={onDragStart}
    style="right: {appState.filePanelWidth}px"></div>
  <aside class="file-panel" style="width: {appState.filePanelWidth}px">
    <div class="fp-header">
      <span class="fp-title">{t('files.title')}</span>
      <button class="fp-close" onclick={() => (appState.filePanelVisible = false)} title="Close">&times;</button>
    </div>
    <div class="fp-body">
      <FileTree {rootPath} />
    </div>
  </aside>
{/if}

<style>
  .fp-resizer {
    position: fixed;
    top: 0;
    bottom: 28px;
    width: 4px;
    background: transparent;
    cursor: col-resize;
    z-index: 11;
  }
  .fp-resizer:hover { background: #0e639c; }
  .file-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 28px;
    background: #252526;
    border-left: 1px solid #3c3c3c;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    z-index: 10;
    user-select: none;
  }
  .fp-header {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid #3c3c3c;
    gap: 8px;
    flex-shrink: 0;
  }
  .fp-title {
    font-size: 12px;
    font-weight: 600;
    color: #ccc;
    flex: 1;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .fp-close {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
    border-radius: 3px;
  }
  .fp-close:hover { background: #3c3c3c; color: #fff; }
  .fp-body {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
