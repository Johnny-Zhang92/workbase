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

  async function loadRoot() {
    if (!rootPath) return;
    // Only show loading indicator on first load — keep old tree visible during refresh
    if (nodes.length === 0) loading = true;
    try {
      const [entries, gitResult] = await Promise.all([
        invoke<DirEntry[]>('list_dir', { path: rootPath }),
        invoke<{ branch: string; files: { path: string; status: string }[] }>('git_status', { rootPath }).catch(() => null),
      ]);
      const gitMap: Record<string, string> = {};
      if (gitResult) {
        for (const f of gitResult.files) {
          gitMap[f.path.replace(/\\/g, '/')] = f.status;
        }
      }
      nodes = entries.map(e => ({
        ...e,
        loaded: false,
        expanded: false,
        gitStatus: gitMap[e.path.replace(/\\/g, '/')] || undefined,
      }));
      loadedPath = rootPath;
    } catch (e) {
      console.error(e);
      nodes = [];
    }
    loading = false;
  }

  $effect(() => {
    if (rootPath) loadRoot();
  });

  // Auto-refresh when files change on disk
  $effect(() => {
    const _version = appState.fileTreeVersion;
    if (rootPath && _version > 0) loadRoot();
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
