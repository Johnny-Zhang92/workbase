<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { appState } from './stores.svelte';
  import { t } from './i18n.svelte';
  import type { Project, Session } from './types';

  let dragStart = $state(0);
  let dragging = $state(false);
  let dragRaf: number | null = null;
  let pendingWidth: number | null = null;
  let expandedProjects = $state<number[]>([]);
  let hasAutoRestored = $state(false);
  let editingType = $state<'project' | 'session' | null>(null);
  let editingId = $state<number>(0);
  let editingName = $state('');

  // Session launcher
  let launcherVisible = $state(false);
  let launcherX = $state(0);
  let launcherY = $state(0);
  let launcherBtn = $state<HTMLButtonElement | undefined>(undefined);
  let hasClaude = $state(false);
  let hasCodex = $state(false);

  // Template editing
  let templateEditorVisible = $state(false);
  let templateEditorId = $state<number | null>(null);
  let templateEditorName = $state('');
  let templateEditorCommand = $state('');
  let templateEditorIcon = $state('');

  // Context menu
  let ctxVisible = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);
  let ctxTarget = $state<{ type: 'project' | 'session'; id: number; name: string; path: string } | null>(null);

  // Project drag-to-reorder state
  let projDragIdx = $state<number | null>(null);
  let projDragOverIdx = $state<number | null>(null);

  function onHandleDown(e: MouseEvent, idx: number) {
    e.preventDefault();
    e.stopPropagation();
    projDragIdx = idx;
  }

  function onProjDragOver(e: MouseEvent, idx: number) {
    if (projDragIdx === null) return;
    e.preventDefault();
    projDragOverIdx = idx;
  }

  function onMouseUp() {
    if (projDragIdx === null || projDragOverIdx === null || projDragIdx === projDragOverIdx) {
      projDragIdx = null;
      projDragOverIdx = null;
      return;
    }
    const projects = [...appState.projects];
    const [moved] = projects.splice(projDragIdx, 1);
    projects.splice(projDragOverIdx, 0, moved);
    appState.projects = projects;
    const ids = projects.map(p => p.id);
    invoke('reorder_projects', { ids }).catch(() => {});
    projDragIdx = null;
    projDragOverIdx = null;
  }

  function isExpanded(id: number) { return expandedProjects.includes(id); }
  function expand(id: number) { expandedProjects = [...new Set([...expandedProjects, id])]; }
  function collapse(id: number) { expandedProjects = expandedProjects.filter(p => p !== id); }

  async function addProject() {
    appState.statusText = t('sidebar.status.selecting');
    let selected: string | null = null;
    try {
      selected = await open({
        directory: true, multiple: false, title: 'Select Project Folder',
      }) as string | null;
    } catch (e) {
      appState.statusText = t('sidebar.status.cancelled');
      return;
    }
    if (!selected) { appState.statusText = t('status.ready'); return; }

    try {
      const name = selected.split(/[/\\]/).pop() || selected;
      appState.statusText = t('sidebar.status.adding');
      await invoke('create_project', { name, rootPath: selected });
      await loadProjects(true);
      const newProject = appState.projects.find(p => p.root_path === selected);
      if (newProject) await selectProject(newProject.id);
      appState.statusText = t('sidebar.status.added', { name });
    } catch (e) {
      appState.statusText = t('status.error', { msg: String(e) });
    }
  }

  async function removeProject(id: number) {
    try {
      await invoke('delete_project', { id });
      if (appState.activeProjectId === id) {
        appState.closeAllTabs = true;
        appState.activeProjectId = null;
        appState.sessions = [];
      }
      collapse(id);
      await loadProjects(true);
    } catch (e) { console.error(e); }
  }

  async function selectProject(id: number) {
    expandedProjects = [id];
    const isNewProject = appState.activeProjectId !== id;
    if (isNewProject) appState.closeAllTabs = true;
    appState.activeProjectId = id;
    await loadSessions(id);
    if (appState.sessions.length === 0) await addSession();
    if (appState.sessions.length > 0 && isNewProject) {
      // Open all sessions as tabs
      for (const s of appState.sessions) {
        appState.activeSessionId = `session_${s.id}`;
        // Wait for openTab to finish before opening next
        let waited = 0;
        while (appState.tabOpening && waited < 5000) {
          await new Promise(r => setTimeout(r, 50));
          waited += 50;
        }
      }
    }
  }

  async function addSession() {
    if (!appState.activeProjectId) return;
    try {
      const name = t('sidebar.default_terminal', { n: appState.sessions.length + 1 });
      await invoke('create_session', { projectId: appState.activeProjectId, name, cwd: null, launchCommand: '', launchType: 'shell' });
      await loadSessions(appState.activeProjectId);
      const s = appState.sessions[appState.sessions.length - 1];
      if (s) selectSession(s);
    } catch (e) { console.error(e); }
  }

  async function launchSession(label: string, launchCommand: string, launchType: string) {
    if (!appState.activeProjectId) return;
    launcherVisible = false;
    try {
      // Auto-name: count existing sessions with same base name
      const existing = appState.sessions.filter(s => s.name === label || s.name.startsWith(label + ' ('));
      const name = existing.length > 0 ? `${label} (${existing.length + 1})` : label;
      await invoke('create_session', { projectId: appState.activeProjectId, name, cwd: null, launchCommand, launchType });
      await loadSessions(appState.activeProjectId);
      const s = appState.sessions[appState.sessions.length - 1];
      if (s) selectSession(s);
    } catch (e) { console.error(e); }
  }

  async function loadTemplates() {
    try {
      appState.templates = await invoke<any[]>('list_templates');
    } catch (_) { appState.templates = []; }
  }

  async function saveTemplate() {
    const name = templateEditorName.trim();
    const cmd = templateEditorCommand.trim();
    if (!name || !cmd) return;
    try {
      if (templateEditorId) {
        await invoke('update_template', { id: templateEditorId, name, launchCommand: cmd, icon: templateEditorIcon || '' });
      } else {
        await invoke('create_template', { name, launchCommand: cmd, icon: templateEditorIcon || '' });
      }
      await loadTemplates();
    } catch (e) { console.error(e); }
    closeTemplateEditor();
  }

  function closeTemplateEditor() {
    templateEditorVisible = false;
    templateEditorId = null;
    templateEditorName = '';
    templateEditorCommand = '';
    templateEditorIcon = '';
  }

  async function deleteTemplate(id: number) {
    try {
      await invoke('delete_template', { id });
      await loadTemplates();
    } catch (e) { console.error(e); }
  }

  function openTemplateEditor(id?: number) {
    if (id) {
      const tpl = appState.templates.find(t => t.id === id);
      if (tpl) {
        templateEditorId = tpl.id;
        templateEditorName = tpl.name;
        templateEditorCommand = tpl.launch_command;
        templateEditorIcon = tpl.icon;
      }
    } else {
      templateEditorId = null;
      templateEditorName = '';
      templateEditorCommand = '';
      templateEditorIcon = '';
    }
    templateEditorVisible = true;
  }

  function openLauncher() {
    if (launcherVisible) { launcherVisible = false; return; }
    if (launcherBtn) {
      const rect = launcherBtn.getBoundingClientRect();
      launcherX = rect.left;
      launcherY = rect.bottom + 2;
    }
    launcherVisible = true;
    loadTemplates();
  }

  async function removeSession(sessionId: number, e: MouseEvent) {
    e.stopPropagation();
    try {
      const sid = `session_${sessionId}`;
      // Signal App to close the tab first, which handles PTY cleanup
      appState.closeTabSignal = sid;
      await invoke('delete_session_cmd', { id: sessionId, ptyId: sid });
      if (appState.activeSessionId === sid) appState.activeSessionId = null;
      if (appState.activeProjectId) await loadSessions(appState.activeProjectId);
    } catch (err) { console.error(err); }
  }

  function selectSession(session: Session) {
    appState.activeSessionId = `session_${session.id}`;
    appState.activeProjectId = session.project_id;
    appState.statusText = `${session.name} — ${t('status.active')}`;
  }

  async function loadProjects(skipAutoRestore = false) {
    try {
      const list = await invoke<Project[]>('list_projects');
      appState.projects = list;
      if (!skipAutoRestore && !hasAutoRestored && list.length > 0) {
        hasAutoRestored = true;
        for (const project of list) {
          const ss = await invoke<Session[]>('list_sessions', { projectId: project.id });
          if (ss.length > 0) {
            appState.sessions = ss;
            appState.activeProjectId = project.id;
            expandedProjects = [project.id];
            selectSession(ss[0]);
            return;
          }
        }
      }
    } catch (e) { console.error(e); }
  }

  async function loadSessions(projectId: number) {
    try {
      appState.sessions = await invoke<Session[]>('list_sessions', { projectId });
    } catch (e) { console.error(e); }
  }

  function startRenameProject(id: number, name: string) {
    editingType = 'project';
    editingId = id;
    editingName = name;
  }

  function startRenameSession(id: number, name: string) {
    editingType = 'session';
    editingId = id;
    editingName = name;
  }

  async function saveRename() {
    const name = editingName.trim();
    if (!name) { cancelRename(); return; }
    try {
      if (editingType === 'project') {
        await invoke('rename_project', { id: editingId, name });
        await loadProjects(true);
        appState.statusText = t('sidebar.status.renamed_project', { name });
      } else if (editingType === 'session') {
        await invoke('rename_session', { id: editingId, name });
        if (appState.activeProjectId) await loadSessions(appState.activeProjectId);
        appState.statusText = t('sidebar.status.renamed_session', { name });
      }
    } catch (e) { appState.statusText = t('status.error', { msg: String(e) }); }
    cancelRename();
  }

  function cancelRename() {
    editingType = null;
    editingId = 0;
    editingName = '';
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') saveRename();
    if (e.key === 'Escape') cancelRename();
  }

  function showProjectCtx(e: MouseEvent, project: Project) {
    e.preventDefault();
    ctxTarget = { type: 'project', id: project.id, name: project.name, path: project.root_path };
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxVisible = true;
  }

  function showSessionCtx(e: MouseEvent, session: Session, projectId: number) {
    e.preventDefault();
    const project = appState.projects.find(p => p.id === projectId);
    const cwd = session.cwd ? `${project?.root_path ?? ''}/${session.cwd}` : (project?.root_path ?? '');
    ctxTarget = { type: 'session', id: session.id, name: session.name, path: cwd };
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxVisible = true;
  }

  function closeCtx() { ctxVisible = false; ctxTarget = null; }

  function sessionIcon(launchType: string): string {
    switch (launchType) {
      case 'claude': return '🤖';
      case 'codex': return '🤖';
      case 'custom': return '🔧';
      default: return '💻';
    }
  }

  async function ctxOpenInExplorer() {
    if (!ctxTarget) return;
    await invoke('open_in_explorer', { path: ctxTarget.path });
    closeCtx();
  }

  function ctxRename() {
    if (!ctxTarget) return;
    if (ctxTarget.type === 'project') startRenameProject(ctxTarget.id, ctxTarget.name);
    else startRenameSession(ctxTarget.id, ctxTarget.name);
    closeCtx();
  }

  async function ctxRemove() {
    if (!ctxTarget) return;
    if (ctxTarget.type === 'project') await removeProject(ctxTarget.id);
    else {
      const sid = `session_${ctxTarget.id}`;
      appState.closeTabSignal = sid;
      await invoke('delete_session_cmd', { id: ctxTarget.id, ptyId: sid });
      if (appState.activeSessionId === sid) appState.activeSessionId = null;
      if (appState.activeProjectId) await loadSessions(appState.activeProjectId);
    }
    closeCtx();
  }

  async function ctxCopyCwd() {
    if (!ctxTarget) return;
    try {
      await navigator.clipboard.writeText(ctxTarget.path);
      appState.statusText = t('sidebar.status.copied', { path: ctxTarget.path });
    } catch (_) {
      appState.statusText = t('sidebar.status.copy_failed');
    }
    closeCtx();
  }

  async function toggleProject(id: number) {
    try {
      if (isExpanded(id)) collapse(id);
      else expand(id);
      if (!isExpanded(id)) return;
      if (appState.activeProjectId !== id) await selectProject(id);
    } catch (e) { console.error('toggleProject error:', e); }
  }

  function onDragStart(e: MouseEvent) {
    dragging = true; dragStart = e.clientX;
    window.addEventListener('mousemove', onDragMove);
    window.addEventListener('mouseup', onDragEnd);
  }
  function onDragMove(e: MouseEvent) {
    const diff = e.clientX - dragStart;
    pendingWidth = Math.max(180, Math.min(500, appState.sidebarWidth + diff));
    dragStart = e.clientX;
    if (dragRaf === null) {
      dragRaf = requestAnimationFrame(() => {
        if (pendingWidth !== null) appState.sidebarWidth = pendingWidth;
        dragRaf = null;
      });
    }
  }
  function onDragEnd() {
    dragging = false;
    if (pendingWidth !== null) appState.sidebarWidth = pendingWidth;
    pendingWidth = null;
    window.removeEventListener('mousemove', onDragMove);
    window.removeEventListener('mouseup', onDragEnd);
    onMouseUp();
  }

  async function detectTools() {
    // Always show the options — the user knows if they have these installed.
    // If they pick one without the CLI installed, the terminal will just show "command not found".
    hasClaude = true;
    hasCodex = true;
  }

  $effect(() => { loadProjects(); detectTools(); });
</script>

{#if appState.sidebarVisible}
  <aside class="sidebar" style="width: {appState.sidebarWidth}px">
    <div class="sidebar-header">
      <span class="title">{t('sidebar.title')}</span>
      <button class="btn icon" onclick={addProject} title={t('sidebar.add_project')}>+</button>
      <button class="btn icon" onclick={() => (appState.sidebarVisible = false)}>&times;</button>
    </div>
    <div class="project-list">
      {#each appState.projects as project, idx (project.id)}
        <div
          class="project-item"
          class:active={appState.activeProjectId === project.id}
          class:drag-over={projDragOverIdx === idx && projDragIdx !== idx}
          onmouseover={() => { if (projDragIdx !== null) projDragOverIdx = idx; }}
        >
          <div class="project-header" oncontextmenu={(e) => showProjectCtx(e, project)}>

            <span
              class="drag-handle"
              onmousedown={(e) => onHandleDown(e, idx)}
              title={t('sidebar.drag_hint')}
            >⋮⋮</span>

            <button class="expand-btn" onclick={() => toggleProject(project.id)} aria-label="Toggle">
              {isExpanded(project.id) ? '▾' : '▸'}
            </button>
            {#if editingType === 'project' && editingId === project.id}
              <input class="rename-input" type="text" bind:value={editingName}
                onkeydown={handleRenameKeydown} onblur={saveRename} autofocus />
            {:else}
              <button class="project-name-btn" onclick={() => selectProject(project.id)}
                ondblclick={() => startRenameProject(project.id, project.name)}>
                {project.name}
              </button>
            {/if}
            <button class="btn icon delete" onclick={() => removeProject(project.id)} title={t('sidebar.ctx.remove_project')}>&times;</button>
          </div>
          {#if isExpanded(project.id)}
            <div class="session-list">
              {#each appState.sessions as session (session.id)}
                <div class="session-item" class:active={appState.activeSessionId === `session_${session.id}`}
                  onclick={() => selectSession(session)} onkeydown={(e) => { if (e.key === 'Enter') selectSession(session); }}
                  oncontextmenu={(e) => showSessionCtx(e, session, project.id)} role="button" tabindex="0">
                  <span class="session-icon">{sessionIcon(session.launch_type)}</span>
                  {#if editingType === 'session' && editingId === session.id}
                    <input class="rename-input" type="text" bind:value={editingName}
                      onkeydown={handleRenameKeydown} onblur={saveRename} autofocus />
                  {:else}
                    <span class="session-name" role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'F2') startRenameSession(session.id, session.name); }} ondblclick={() => startRenameSession(session.id, session.name)}>{session.name}</span>
                  {/if}
                  <button class="btn icon delete" onclick={(e) => removeSession(session.id, e)} title={t('sidebar.ctx.close_session')}>&times;</button>
                </div>
              {/each}
              <div class="launcher-wrapper">
                <button bind:this={launcherBtn} class="btn new-session" onclick={openLauncher}>{t('sidebar.new_terminal')}</button>
                {#if launcherVisible}
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="launcher-overlay" onclick={() => launcherVisible = false} role="presentation"></div>
                  <div class="launcher-dropdown" style="left: {launcherX}px; top: {launcherY}px;">
                    <button class="launcher-item" onclick={() => launchSession('Shell', '', 'shell')}>
                      <span class="launcher-icon">💻</span> {t('launcher.plain_shell')}
                    </button>
                    {#if hasClaude}
                      <button class="launcher-item" onclick={() => launchSession('Claude Code', 'claude', 'claude')}>
                        <span class="launcher-icon">🤖</span> {t('launcher.claude_code')}
                      </button>
                    {/if}
                    {#if hasCodex}
                      <button class="launcher-item" onclick={() => launchSession('Codex', 'codex', 'codex')}>
                        <span class="launcher-icon">🤖</span> {t('launcher.codex')}
                      </button>
                    {/if}
                    {#if appState.templates.length > 0}
                      <div class="launcher-sep"></div>
                      {#each appState.templates as tpl (tpl.id)}
                        <button class="launcher-item" onclick={() => launchSession(tpl.name, tpl.launch_command, 'custom')}>
                          <span class="launcher-icon">{tpl.icon || '🔧'}</span> {tpl.name}
                        </button>
                      {/each}
                    {/if}
                    <div class="launcher-sep"></div>
                    <button class="launcher-item" onclick={() => { launcherVisible = false; openTemplateEditor(); }}>
                      <span class="launcher-icon">✏️</span> {t('launcher.custom_template')}
                    </button>
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      {/each}
      {#if appState.projects.length === 0}
        <div class="empty-state">
          <p>{t('sidebar.no_projects')}</p>
          <button class="btn primary" onclick={addProject}>{t('sidebar.add_project')}</button>
        </div>
      {/if}
    </div>
  </aside>
  <div class="resizer" role="separator" onmousedown={onDragStart} style="left: {appState.sidebarWidth}px"></div>

  {#if templateEditorVisible}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="tpl-overlay" onclick={closeTemplateEditor} role="presentation"></div>
    <div class="tpl-dialog" role="dialog">
      <h3>{templateEditorId ? t('templates.edit') : t('templates.new')}</h3>
      <label class="tpl-label">{t('templates.name')}</label>
      <input class="tpl-input" type="text" bind:value={templateEditorName} placeholder={t('templates.name_placeholder')} />
      <label class="tpl-label">{t('templates.command')}</label>
      <input class="tpl-input" type="text" bind:value={templateEditorCommand} placeholder={t('templates.command_placeholder')} />
      <label class="tpl-label">{t('templates.icon')}</label>
      <input class="tpl-input" type="text" bind:value={templateEditorIcon} placeholder={t('templates.icon_placeholder')} maxlength="4" />
      <div class="tpl-actions">
        {#if templateEditorId}
          <button class="tpl-btn danger" onclick={() => { deleteTemplate(templateEditorId!); closeTemplateEditor(); }}>{t('templates.delete')}</button>
        {/if}
        <div class="tpl-spacer"></div>
        <button class="tpl-btn secondary" onclick={closeTemplateEditor}>{t('templates.cancel')}</button>
        <button class="tpl-btn primary" onclick={saveTemplate} disabled={!templateEditorName.trim() || !templateEditorCommand.trim()}>{t('templates.save')}</button>
      </div>
    </div>
  {/if}

  {#if ctxVisible && ctxTarget}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="ctx-overlay" onclick={closeCtx} oncontextmenu={(e) => { e.preventDefault(); closeCtx(); }}
      onkeydown={(e) => { if (e.key === 'Escape') closeCtx(); }} role="presentation"></div>
    <div class="ctx-menu" style="left: {ctxX}px; top: {ctxY}px">
      <button class="ctx-item" onclick={ctxOpenInExplorer}>
        <span class="ctx-icon">📂</span> {t('sidebar.ctx.open_explorer')}
      </button>
      <button class="ctx-item" onclick={ctxRename}>
        <span class="ctx-icon">✏️</span> {ctxTarget.type === 'project' ? t('sidebar.ctx.rename_project') : t('sidebar.ctx.rename_session')}
      </button>
      {#if ctxTarget.type === 'session'}
        <button class="ctx-item" onclick={ctxCopyCwd}>
          <span class="ctx-icon">📋</span> {t('sidebar.ctx.copy_cwd')}
        </button>
      {/if}
      <div class="ctx-separator"></div>
      <button class="ctx-item ctx-danger" onclick={ctxRemove}>
        <span class="ctx-icon">✕</span> {ctxTarget.type === 'project' ? t('sidebar.ctx.remove_project') : t('sidebar.ctx.close_session')}
      </button>
    </div>
  {/if}
{/if}

<style>
  .sidebar { position: fixed; top: 0; left: 0; bottom: 28px; background: #252526; border-right: 1px solid #3c3c3c; display: flex; flex-direction: column; overflow: hidden; z-index: 10; user-select: none; }
  .sidebar-header { display: flex; align-items: center; padding: 8px 12px; border-bottom: 1px solid #3c3c3c; gap: 8px; }
  .title { font-size: 13px; font-weight: 600; color: #ccc; flex: 1; }
  .project-list { flex: 1; overflow-y: auto; padding: 4px 0; }
  .project-header { display: flex; align-items: center; padding: 4px 8px; gap: 4px; }
  .project-header:hover { background: #2a2a2b; }
  .project-item.active > .project-header { background: #37373d; }
  .project-item.drag-over { border-top: 2px solid #007acc; }
  .drag-handle { cursor: grab; color: #555; font-size: 10px; padding: 0 2px; flex-shrink: 0; letter-spacing: -1px; user-select: none; }
  .drag-handle:active { cursor: grabbing; color: #888; }
  .expand-btn { background: none; border: none; color: #888; cursor: pointer; font-size: 12px; padding: 0; width: 16px; flex-shrink: 0; }
  .project-name-btn { flex: 1; background: none; border: none; color: #ccc; cursor: pointer; font-size: 13px; text-align: left; padding: 2px 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; border-radius: 3px; }
  .project-name-btn:hover { background: #3c3c3c; }
  .session-list { padding-left: 20px; padding-bottom: 4px; }
  .session-item { display: flex; align-items: center; width: 100%; padding: 3px 8px; gap: 4px; cursor: pointer; font-size: 12px; color: #999; background: none; border: none; text-align: left; }
  .session-item:hover { background: #2a2a2b; }
  .session-item.active { background: #094771; color: #fff; }
  .session-icon { font-size: 13px; width: 18px; flex-shrink: 0; text-align: center; }
  .session-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .btn { background: none; border: none; color: #888; cursor: pointer; font-size: 14px; padding: 2px 6px; border-radius: 3px; flex-shrink: 0; }
  .btn:hover { background: #3c3c3c; color: #fff; }
  .btn.primary { background: #0e639c; color: #fff; padding: 6px 16px; font-size: 13px; }
  .btn.primary:hover { background: #1177bb; }
  .btn.delete:hover { background: #5a1d1d; color: #f44747; }
  .btn.new-session { display: block; width: 100%; text-align: left; padding: 4px 8px; font-size: 12px; color: #888; margin-top: 2px; }
  .btn.icon { font-size: 16px; padding: 0 4px; line-height: 1; }
  .rename-input { flex: 1; background: #3c3c3c; border: 1px solid #0e639c; color: #fff; font-size: 13px; padding: 2px 6px; border-radius: 3px; outline: none; min-width: 0; }
  .session-item .rename-input { font-size: 12px; padding: 1px 4px; }
  .empty-state { padding: 24px 12px; text-align: center; color: #888; }
  .empty-state p { margin-bottom: 12px; font-size: 13px; }
  .resizer { position: fixed; top: 0; bottom: 28px; width: 4px; background: transparent; cursor: col-resize; z-index: 11; }
  .resizer:hover { background: #0e639c; }

  .ctx-overlay { position: fixed; inset: 0; z-index: 99; }
  .ctx-menu { position: fixed; z-index: 100; background: #2d2d30; border: 1px solid #454545; border-radius: 6px; padding: 4px; min-width: 180px; box-shadow: 0 4px 16px rgba(0,0,0,0.4); }
  .ctx-item { display: flex; align-items: center; gap: 8px; width: 100%; padding: 6px 10px; background: none; border: none; color: #ccc; font-size: 13px; cursor: pointer; border-radius: 4px; text-align: left; }
  .ctx-item:hover { background: #094771; color: #fff; }
  .ctx-icon { width: 16px; text-align: center; flex-shrink: 0; font-size: 12px; }
  .ctx-separator { height: 1px; background: #454545; margin: 4px 8px; }
  .ctx-danger:hover { background: #5a1d1d !important; color: #f44747 !important; }

  .launcher-wrapper { position: relative; }
  .launcher-overlay { position: fixed; inset: 0; z-index: 98; }
  .launcher-dropdown {
    position: fixed; z-index: 99;
    background: #2d2d30; border: 1px solid #454545; border-radius: 6px;
    padding: 4px; box-shadow: 0 4px 16px rgba(0,0,0,0.5);
    min-width: 180px;
  }
  .launcher-item {
    display: flex; align-items: center; gap: 8px;
    width: 100%; padding: 6px 10px;
    background: none; border: none; color: #ccc;
    font-size: 12px; cursor: pointer; border-radius: 4px;
    text-align: left;
  }
  .launcher-item:hover { background: #094771; color: #fff; }
  .launcher-icon { width: 18px; text-align: center; flex-shrink: 0; font-size: 14px; }
  .launcher-sep { height: 1px; background: #454545; margin: 3px 8px; }

  .tpl-overlay { position: fixed; inset: 0; z-index: 301; background: rgba(0,0,0,0.5); }
  .tpl-dialog {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
    z-index: 302; background: #252526; border: 1px solid #454545;
    border-radius: 8px; padding: 20px 24px; width: 380px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.6);
  }
  .tpl-dialog h3 { font-size: 15px; font-weight: 600; color: #ccc; margin: 0 0 14px 0; }
  .tpl-label { display: block; font-size: 11px; color: #999; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px; }
  .tpl-input {
    width: 100%; padding: 6px 8px; background: #1e1e1e; border: 1px solid #3c3c3c;
    color: #ccc; border-radius: 4px; font-size: 13px; outline: none; font-family: inherit;
    box-sizing: border-box; margin-bottom: 10px;
  }
  .tpl-input:focus { border-color: #007acc; }
  .tpl-actions { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
  .tpl-spacer { flex: 1; }
  .tpl-btn {
    padding: 5px 14px; border: none; border-radius: 4px;
    cursor: pointer; font-size: 12px; font-weight: 500;
  }
  .tpl-btn.primary { background: #007acc; color: #fff; }
  .tpl-btn.primary:hover { background: #005fa3; }
  .tpl-btn.primary:disabled { opacity: 0.4; cursor: default; }
  .tpl-btn.secondary { background: #3c3c3c; color: #ccc; }
  .tpl-btn.secondary:hover { background: #555; }
  .tpl-btn.danger { background: transparent; color: #e88; border: 1px solid #5a1d1d; }
  .tpl-btn.danger:hover { background: #5a1d1d; }
</style>
