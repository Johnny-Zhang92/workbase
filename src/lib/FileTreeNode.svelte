<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import FileTreeNode from './FileTreeNode.svelte';
  import { appState } from './stores.svelte';
  import { t } from './i18n.svelte';

  interface DirEntry {
    name: string;
    path: string;
    is_dir: boolean;
  }

  export interface TreeNode extends DirEntry {
    children?: TreeNode[];
    loaded: boolean;
    expanded: boolean;
    gitStatus?: string;
  }

  let { node, depth = 0, onSelect, rootPath = '' }: {
    node: TreeNode;
    depth?: number;
    onSelect?: (path: string) => void;
    rootPath?: string;
  } = $props();

  let gitStatus = $derived(node.gitStatus ?? '');
  const gitClass: Record<string, string> = {
    M: 'git-m', A: 'git-a', D: 'git-d', '?': 'git-u', U: 'git-u',
    R: 'git-r', C: 'git-r',
  };
  let gitCls = $derived(gitClass[gitStatus] ?? '');

  let ctxVisible = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);

  async function loadChildren() {
    if (node.loaded || !node.is_dir) return;
    try {
      const entries = await invoke<DirEntry[]>('list_dir', { path: node.path });
      const gitMap: Record<string, string> = {};
      for (const [k, v] of Object.entries(appState.gitFiles)) {
        gitMap[k] = v;
      }
      node.children = entries.map(e => ({
        ...e,
        loaded: false,
        expanded: false,
        gitStatus: gitMap[e.path.replace(/\\/g, '/')] || undefined,
      }));
      node.loaded = true;
    } catch (e) { console.error(e); }
  }

  function toggle() {
    if (!node.is_dir) { onSelect?.(node.path); return; }
    node.expanded = !node.expanded;
    if (node.expanded && !node.loaded) loadChildren();
  }

  async function dblClick() {
    if (node.is_dir) return;
    try { await invoke('open_file', { path: node.path }); } catch (_) {}
  }

  function showCtx(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxVisible = true;
  }

  function closeCtx() { ctxVisible = false; }

  function relPath() {
    if (!rootPath) return node.path;
    const rp = rootPath.replace(/\\/g, '/');
    const np = node.path.replace(/\\/g, '/');
    return np.startsWith(rp) ? np.slice(rp.length).replace(/^\//, '') : node.path;
  }

  async function copyPath() {
    try { await navigator.clipboard.writeText(node.path); } catch (_) {}
    closeCtx();
  }

  async function copyRelPath() {
    try { await navigator.clipboard.writeText(relPath()); } catch (_) {}
    closeCtx();
  }

  async function openInExplorer() {
    try { await invoke('open_in_explorer', { path: node.path }); } catch (_) {}
    closeCtx();
  }

  async function openFile() {
    try { await invoke('open_file', { path: node.path }); } catch (_) {}
    closeCtx();
  }
</script>

<div class="ft-row" style="padding-left: {depth * 14 + 6}px"
  onclick={toggle}
  ondblclick={dblClick}
  oncontextmenu={showCtx}
  onkeydown={(e) => { if (e.key === 'Enter') toggle(); }}
  role="treeitem" aria-selected={false} tabindex="0">
  <span class="ft-arrow">{node.is_dir ? (node.expanded ? '▾' : '▸') : ''}</span>
  <span class="ft-icon">{node.is_dir ? '📁' : '📄'}</span>
  <span class="ft-name">{node.name}</span>
  {#if gitStatus}
    <span class="ft-git {gitCls}">{gitStatus}</span>
  {/if}
</div>

{#if ctxVisible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="ctx-overlay" onclick={closeCtx} oncontextmenu={(e) => { e.preventDefault(); closeCtx(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeCtx(); }} role="presentation"></div>
  <div class="ctx-menu" style="left: {ctxX}px; top: {ctxY}px">
    {#if !node.is_dir}
      <button class="ctx-item" onclick={openFile}>
        <span class="ctx-icon">🔗</span> {t('files.ctx.open')}
      </button>
      <div class="ctx-separator"></div>
    {/if}
    <button class="ctx-item" onclick={openInExplorer}>
      <span class="ctx-icon">📂</span> {node.is_dir ? t('files.ctx.open_in_explorer') : t('files.ctx.reveal_in_explorer')}
    </button>
    <button class="ctx-item" onclick={copyPath}>
      <span class="ctx-icon">📋</span> {t('files.ctx.copy_path')}
    </button>
    {#if !node.is_dir}
      <button class="ctx-item" onclick={copyRelPath}>
        <span class="ctx-icon">📋</span> {t('files.ctx.copy_rel_path')}
      </button>
    {/if}
  </div>
{/if}

{#if node.is_dir && node.expanded && node.children}
  {#each node.children as child (child.path)}
    <FileTreeNode node={child} depth={depth + 1} {onSelect} {rootPath} />
  {/each}
{/if}

<style>
  .ft-row {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 12px;
    color: #ccc;
    white-space: nowrap;
    user-select: none;
  }
  .ft-row:hover { background: #2a2a2b; }
  .ft-arrow { width: 14px; flex-shrink: 0; color: #666; font-size: 10px; text-align: center; }
  .ft-icon { width: 16px; flex-shrink: 0; text-align: center; font-size: 12px; }
  .ft-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ft-git { font-size: 10px; font-weight: 700; flex-shrink: 0; margin-left: 4px; min-width: 14px; text-align: center; }
  .git-m { color: #e5c07b; } /* modified */
  .git-a { color: #98c379; } /* added */
  .git-d { color: #e06c75; } /* deleted */
  .git-u { color: #56b6c2; } /* untracked/conflict */
  .git-r { color: #c678dd; } /* renamed/copied */
  .ctx-overlay { position: fixed; inset: 0; z-index: 199; }
  .ctx-menu {
    position: fixed; z-index: 200; background: #2d2d30; border: 1px solid #454545;
    border-radius: 6px; padding: 4px; min-width: 180px; box-shadow: 0 4px 16px rgba(0,0,0,0.4);
    font-size: 13px;
  }
  .ctx-item {
    display: flex; align-items: center; gap: 8px; width: 100%;
    padding: 6px 10px; background: none; border: none; color: #ccc;
    cursor: pointer; border-radius: 4px; text-align: left; font-size: 13px;
  }
  .ctx-item:hover { background: #094771; color: #fff; }
  .ctx-icon { width: 16px; text-align: center; flex-shrink: 0; }
  .ctx-separator { height: 1px; background: #454545; margin: 4px 8px; }
</style>
