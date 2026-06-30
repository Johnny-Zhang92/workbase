<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from './stores.svelte';
  import { t } from './i18n.svelte';
  import FileTreeNode from './FileTreeNode.svelte';
  import type { TreeNode } from './FileTreeNode.svelte';

  interface DirEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  let { rootPath }: { rootPath: string } = $props();
  let nodes = $state<TreeNode[]>([]);
  let loading = $state(false);
  let loadedPath = $state('');
  let lastFullRefresh = 0;

  async function loadRoot() {
    if (!rootPath) return;
    if (nodes.length === 0) loading = true;
    try {
      const entries = await invoke<DirEntry[]>('list_dir', { path: rootPath });
      // Preserve expanded/loaded state of existing nodes on refresh
      const existingMap = new Map(nodes.map(n => [n.name, n]));
      nodes = entries.map(e => ({
        ...e,
        loaded: existingMap.get(e.name)?.loaded ?? false,
        expanded: existingMap.get(e.name)?.expanded ?? false,
        gitStatus: appState.gitFiles[e.path.replace(/\\/g, '/')] || undefined,
      }));
      loadedPath = rootPath;
      lastFullRefresh = Date.now();
    } catch (e) {
      console.error(e);
      nodes = [];
    }
    loading = false;
  }

  // Update git status on existing nodes only — no IPC, no node replacement
  function applyGitStatus() {
    for (const node of nodes) {
      node.gitStatus = appState.gitFiles[node.path.replace(/\\/g, '/')] || undefined;
    }
  }

  $effect(() => {
    if (rootPath) loadRoot();
  });

  // Auto-refresh when files change on disk
  $effect(() => {
    const _version = appState.fileTreeVersion;
    if (!rootPath || _version === 0 || !appState.filePanelVisible) return;
    // Always apply git status (instant, from reactive store)
    applyGitStatus();
    // Full reload (list_dir IPC) throttled to once per 10s
    if (Date.now() - lastFullRefresh > 10_000) {
      loadRoot();
    }
  });
</script>

<div class="file-tree">
  {#if loading}
    <div class="ft-loading">{t('files.loading')}</div>
  {:else if nodes.length === 0}
    <div class="ft-empty">{t('files.empty')}</div>
  {:else}
    {#each nodes as node (node.path)}
      <FileTreeNode {node} depth={0} {rootPath} />
    {/each}
  {/if}
</div>

<style>
  .file-tree {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2px 0;
  }
  .ft-loading, .ft-empty {
    padding: 8px 12px;
    font-size: 12px;
    color: #666;
    font-style: italic;
  }
</style>
