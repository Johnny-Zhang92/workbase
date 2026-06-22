<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { t, locale, setLocale } from './i18n.svelte';
  import { KEY_ACTIONS, loadKeybindings, recordingId, getBinding, resetBinding, bindingToDisplay, type KeyAction } from './keybindings.svelte';
  import { PRESETS, loadThemeName, saveThemeName } from './theme.svelte';
  import type { SessionTemplate } from './types';
  import { appState } from './stores.svelte';

  let { onClose }: { onClose: () => void } = $props();

  let fontFamily = $state('Cascadia Code, Fira Code, JetBrains Mono, Consolas, monospace');
  let fontSize = $state(14);
  let bgColor = $state('#1e1e1e');
  let fgColor = $state('#d4d4d4');
  let shellPath = $state('');
  let selectToCopy = $state(false);
  let currentTheme = $state('devterm-dark');
  let detectedShell = $state('');

  // Template management
  let templates = $state<SessionTemplate[]>([]);

  let loaded = $state(false);
  let fontInputEl = $state<HTMLInputElement | undefined>(undefined);

  async function loadSettings() {
    try {
      const [ff, fs, bg, fg, sh, stc] = await Promise.all([
        invoke<string | null>('get_setting', { key: 'fontFamily' }).catch(() => null),
        invoke<string | null>('get_setting', { key: 'fontSize' }).catch(() => null),
        invoke<string | null>('get_setting', { key: 'bgColor' }).catch(() => null),
        invoke<string | null>('get_setting', { key: 'fgColor' }).catch(() => null),
        invoke<string | null>('get_setting', { key: 'shellPath' }).catch(() => null),
        invoke<string | null>('get_setting', { key: 'selectToCopy' }).catch(() => null),
      ]);
      if (ff) fontFamily = ff;
      if (fs) fontSize = parseInt(fs) || 14;
      if (bg) bgColor = bg;
      if (fg) fgColor = fg;
      if (sh) shellPath = sh;
      if (stc) selectToCopy = stc === 'true';
      const theme = await loadThemeName();
      currentTheme = theme;
    } catch (_) {}
    detectedShell = await invoke<string>('detect_shell').catch(() => '');
    await loadKeybindings();
    templates = await invoke<SessionTemplate[]>('list_templates').catch(() => []);
    loaded = true;
  }

  async function save(key: string, value: string) {
    try { await invoke('set_setting', { key, value }); } catch (_) {}
  }

  function onFontFamilyChange() {
    save('fontFamily', fontFamily);
    apply();
  }
  function onFontSizeChange() {
    save('fontSize', String(fontSize));
    apply();
  }
  function onBgChange() {
    save('bgColor', bgColor);
    apply();
  }
  function onFgChange() {
    save('fgColor', fgColor);
    apply();
  }
  function onShellChange() {
    save('shellPath', shellPath);
  }
  function onSelectToCopyChange() {
    save('selectToCopy', String(selectToCopy));
  }

  async function selectTheme(name: string) {
    currentTheme = name;
    await saveThemeName(name);
    apply();
  }

  // ── Template management ──
  let tplEditor = $state(false);
  let tplEditId = $state<number | null>(null);
  let tplName = $state('');
  let tplCommand = $state('');
  let tplIcon = $state('');

  async function loadTemplates() {
    templates = await invoke<SessionTemplate[]>('list_templates').catch(() => []);
    appState.templates = templates;
  }

  function openTplEditor(id?: number) {
    if (id) {
      const t = templates.find(x => x.id === id);
      if (t) { tplEditId = t.id; tplName = t.name; tplCommand = t.launch_command; tplIcon = t.icon; }
    } else {
      tplEditId = null; tplName = ''; tplCommand = ''; tplIcon = '';
    }
    tplEditor = true;
  }

  function closeTplEditor() {
    tplEditor = false;
    tplEditId = null;
    tplName = '';
    tplCommand = '';
    tplIcon = '';
  }

  async function saveTpl() {
    if (!tplName.trim() || !tplCommand.trim()) return;
    try {
      if (tplEditId) {
        await invoke('update_template', { id: tplEditId, name: tplName.trim(), launchCommand: tplCommand.trim(), icon: tplIcon || '' });
      } else {
        await invoke('create_template', { name: tplName.trim(), launchCommand: tplCommand.trim(), icon: tplIcon || '' });
      }
      await loadTemplates();
    } catch (e) { console.error(e); }
    closeTplEditor();
  }

  async function deleteTpl(id: number) {
    try {
      await invoke('delete_template', { id });
      await loadTemplates();
    } catch (e) { console.error(e); }
  }

  function apply() {
    (globalThis as any).__devtermApplyTheme?.();
  }

  function startRecording(actionId: string) {
    recordingId.value = actionId;
  }

  async function resetKey(actionId: string) {
    await resetBinding(actionId);
  }

  function recordingDisplay(action: KeyAction): string {
    if (recordingId.value === action.id) return t('keybind.press_keys');
    return bindingToDisplay(getBinding(action.id));
  }

  $effect(() => {
    if (loaded) fontInputEl?.focus();
  });
  $effect(() => { loadSettings(); });
</script>

{#if loaded}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="st-overlay" onclick={onClose} role="presentation">
    <div class="st-dialog" onclick={(e) => e.stopPropagation()} role="dialog">
      <h2 class="st-title">{t('settings.title')}</h2>

      <div class="st-group">
        <label class="st-label">{t('settings.font_family')}</label>
        <input bind:this={fontInputEl} type="text" class="st-input" bind:value={fontFamily} onchange={onFontFamilyChange} />
        <span class="st-hint">{t('settings.font_family_hint')}</span>
      </div>

      <div class="st-group">
        <label class="st-label">{t('settings.font_size')}</label>
        <input type="number" class="st-input st-narrow" min="10" max="28" bind:value={fontSize} onchange={onFontSizeChange} />
      </div>

      <div class="st-row">
        <div class="st-group">
          <label class="st-label">{t('settings.background')}</label>
          <div class="st-color-row">
            <input type="color" class="st-color" bind:value={bgColor} onchange={onBgChange} />
            <input type="text" class="st-input st-narrow" bind:value={bgColor} onchange={onBgChange} />
          </div>
        </div>
        <div class="st-group">
          <label class="st-label">{t('settings.foreground')}</label>
          <div class="st-color-row">
            <input type="color" class="st-color" bind:value={fgColor} onchange={onFgChange} />
            <input type="text" class="st-input st-narrow" bind:value={fgColor} onchange={onFgChange} />
          </div>
        </div>
      </div>

      <div class="st-group">
        <label class="st-label">{t('settings.shell_path')}</label>
        <input type="text" class="st-input" bind:value={shellPath} onchange={onShellChange} placeholder={detectedShell || 'Auto-detected shell'} />
        <span class="st-hint">{t('settings.shell_hint')} ({detectedShell || 'unknown'})</span>
      </div>

      <div class="st-group">
        <label class="st-label">{t('settings.select_to_copy')}</label>
        <label class="st-toggle">
          <input type="checkbox" bind:checked={selectToCopy} onchange={onSelectToCopyChange} />
          <span class="st-toggle-label">{t('settings.select_to_copy_hint')}</span>
        </label>
      </div>

      <div class="st-group">
        <label class="st-label">{t('keybind.section_title')}</label>
        <div class="kb-list">
          {#each KEY_ACTIONS as action (action.id)}
            <div class="kb-row" class:kb-recording={recordingId.value === action.id}>
              <span class="kb-label">{t(action.labelKey)}</span>
              <button class="kb-key" onclick={() => startRecording(action.id)}>
                {recordingDisplay(action)}
              </button>
              <button class="kb-reset" onclick={() => resetKey(action.id)} title={t('keybind.reset')}>↺</button>
            </div>
          {/each}
        </div>
      </div>

      <div class="st-group">
        <label class="st-label">{t('settings.theme')}</label>
        <div class="theme-grid">
          {#each PRESETS as theme (theme.name)}
            <button
              class="theme-swatch"
              class:theme-active={currentTheme === theme.name}
              onclick={() => selectTheme(theme.name)}
              title={theme.label}
            >
              <span class="theme-colors" style="background: {theme.background};">
                <span class="theme-dot" style="background: {theme.ansi[1]};"></span>
                <span class="theme-dot" style="background: {theme.ansi[2]};"></span>
                <span class="theme-dot" style="background: {theme.ansi[4]};"></span>
                <span class="theme-dot" style="background: {theme.ansi[3]};"></span>
                <span class="theme-dot" style="background: {theme.ansi[5]};"></span>
                <span class="theme-dot" style="background: {theme.ansi[6]};"></span>
              </span>
              <span class="theme-name">{theme.label}</span>
            </button>
          {/each}
        </div>
        <span class="st-hint">{t('settings.theme_hint')}</span>
      </div>

      <div class="st-group">
        <label class="st-label">{t('templates.title')}</label>
        <div class="tpl-list">
          {#each templates as tpl (tpl.id)}
            <div class="tpl-row">
              <span class="tpl-row-icon">{tpl.icon || '🔧'}</span>
              <div class="tpl-row-info">
                <span class="tpl-row-name">{tpl.name}</span>
                <span class="tpl-row-cmd">{tpl.launch_command}</span>
              </div>
              <button class="tpl-row-btn" onclick={() => openTplEditor(tpl.id)} title={t('templates.edit')}>✏️</button>
              <button class="tpl-row-btn" onclick={() => deleteTpl(tpl.id)} title={t('templates.delete')}>✕</button>
            </div>
          {:else}
            <span class="st-hint">{t('templates.hint')}</span>
          {/each}
        </div>
        <button class="tpl-add-btn" onclick={() => openTplEditor()}>+ {t('templates.new')}</button>
      </div>

      <div class="st-group">
        <label class="st-label">{t('settings.language')}</label>
        <select class="st-select" value={locale.value} onchange={(e) => setLocale(e.currentTarget.value as 'en' | 'zh')}>
          <option value="en">English</option>
          <option value="zh">中文</option>
        </select>
        <span class="st-hint">{t('settings.language_hint')}</span>
      </div>
    </div>
  </div>
{/if}

{#if tplEditor}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="tpl-overlay" onclick={closeTplEditor} role="presentation"></div>
  <div class="tpl-dialog" role="dialog">
    <h3>{tplEditId ? t('templates.edit') : t('templates.new')}</h3>
    <label class="tpl-label">{t('templates.name')}</label>
    <input class="tpl-input" type="text" bind:value={tplName} placeholder={t('templates.name_placeholder')} />
    <label class="tpl-label">{t('templates.command')}</label>
    <input class="tpl-input" type="text" bind:value={tplCommand} placeholder={t('templates.command_placeholder')} />
    <label class="tpl-label">{t('templates.icon')}</label>
    <input class="tpl-input" type="text" bind:value={tplIcon} placeholder={t('templates.icon_placeholder')} maxlength="4" />
    <div class="tpl-actions">
      {#if tplEditId}
        <button class="tpl-btn danger" onclick={() => { deleteTpl(tplEditId!); closeTplEditor(); }}>{t('templates.delete')}</button>
      {/if}
      <div class="tpl-spacer"></div>
      <button class="tpl-btn secondary" onclick={closeTplEditor}>{t('templates.cancel')}</button>
      <button class="tpl-btn primary" onclick={saveTpl} disabled={!tplName.trim() || !tplCommand.trim()}>{t('templates.save')}</button>
    </div>
  </div>
{/if}

<style>
  .st-overlay {
    position: fixed; inset: 0; z-index: 300;
    background: rgba(0, 0, 0, 0.5);
    display: flex; justify-content: center; padding-top: 12vh;
  }
  .st-dialog {
    width: 460px; max-height: 70vh;
    background: #252526; border: 1px solid #454545;
    border-radius: 8px; box-shadow: 0 8px 32px rgba(0,0,0,0.6);
    padding: 20px 24px; overflow-y: auto;
  }
  .st-title { font-size: 16px; font-weight: 600; color: #ccc; margin: 0 0 16px 0; }
  .st-group { margin-bottom: 14px; flex: 1; }
  .st-label { display: block; font-size: 12px; color: #999; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px; }
  .st-input {
    width: 100%; padding: 6px 8px; background: #1e1e1e; border: 1px solid #3c3c3c;
    color: #ccc; border-radius: 4px; font-size: 13px; outline: none; font-family: inherit;
    box-sizing: border-box;
  }
  .st-input:focus { border-color: #007acc; }
  .st-narrow { width: 100px; }
  .st-hint { font-size: 11px; color: #666; margin-top: 3px; display: block; }
  .st-row { display: flex; gap: 14px; }
  .st-color-row { display: flex; align-items: center; gap: 6px; }
  .st-color { width: 28px; height: 28px; border: 1px solid #3c3c3c; border-radius: 4px; cursor: pointer; padding: 0; background: none; }
  .st-select {
    width: 100%; padding: 6px 8px; background: #1e1e1e; border: 1px solid #3c3c3c;
    color: #ccc; border-radius: 4px; font-size: 13px; outline: none; font-family: inherit;
    cursor: pointer; box-sizing: border-box;
  }
  .st-select:focus { border-color: #007acc; }
  .kb-list { max-height: 240px; overflow-y: auto; border: 1px solid #3c3c3c; border-radius: 4px; }
  .kb-row { display: flex; align-items: center; gap: 8px; padding: 5px 8px; border-bottom: 1px solid #3c3c3c; }
  .kb-row:last-child { border-bottom: none; }
  .kb-row.kb-recording { background: #094771; }
  .kb-label { flex: 1; font-size: 12px; color: #ccc; }
  .kb-key {
    background: #3c3c3c; border: 1px solid #555; color: #ccc; padding: 3px 10px;
    border-radius: 3px; font-size: 12px; cursor: pointer; font-family: inherit;
    min-width: 100px; text-align: center;
  }
  .kb-key:hover { background: #555; border-color: #007acc; }
  .kb-recording .kb-key { background: #007acc; border-color: #007acc; color: #fff; animation: blink 0.8s infinite; }
  @keyframes blink { 50% { opacity: 0.5; } }
  .kb-reset {
    background: none; border: 1px solid transparent; color: #888; cursor: pointer;
    font-size: 14px; padding: 2px 6px; border-radius: 3px;
  }
  .kb-reset:hover { background: #3c3c3c; color: #fff; border-color: #555; }
  .st-toggle {
    display: flex; align-items: center; gap: 8px;
    cursor: pointer; padding: 6px 0;
  }
  .st-toggle input[type="checkbox"] {
    width: 16px; height: 16px; accent-color: #007acc; cursor: pointer;
  }
  .st-toggle-label { font-size: 13px; color: #ccc; cursor: pointer; }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }
  .theme-swatch {
    background: #1e1e1e;
    border: 1px solid #3c3c3c;
    border-radius: 6px;
    padding: 8px;
    cursor: pointer;
    text-align: center;
    transition: border-color 0.15s, background 0.15s;
  }
  .theme-swatch:hover { border-color: #555; background: #333; }
  .theme-swatch.theme-active {
    border-color: #007acc;
    background: #094771;
  }
  .theme-colors {
    display: flex;
    gap: 3px;
    justify-content: center;
    padding: 6px 4px;
    border-radius: 4px;
    margin-bottom: 4px;
  }
  .theme-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid rgba(255,255,255,0.15);
  }
  .theme-name {
    font-size: 11px;
    color: #999;
    display: block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .theme-active .theme-name { color: #fff; }

  .tpl-list { border: 1px solid #3c3c3c; border-radius: 4px; max-height: 200px; overflow-y: auto; }
  .tpl-row { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border-bottom: 1px solid #3c3c3c; }
  .tpl-row:last-child { border-bottom: none; }
  .tpl-row-icon { font-size: 14px; flex-shrink: 0; }
  .tpl-row-info { flex: 1; min-width: 0; }
  .tpl-row-name { display: block; font-size: 13px; color: #ccc; }
  .tpl-row-cmd { display: block; font-size: 11px; color: #888; font-family: monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tpl-row-btn { background: none; border: 1px solid transparent; color: #888; cursor: pointer; font-size: 13px; padding: 2px 6px; border-radius: 3px; }
  .tpl-row-btn:hover { background: #3c3c3c; color: #fff; border-color: #555; }
  .tpl-add-btn { margin-top: 6px; padding: 5px 12px; background: #0e639c; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
  .tpl-add-btn:hover { background: #1177bb; }

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
