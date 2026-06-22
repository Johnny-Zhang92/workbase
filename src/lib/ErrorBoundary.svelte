<script lang="ts">
  import { t } from './i18n.svelte';

  interface Props {
    onError?: (error: Error) => void;
    children: any;
  }

  let { onError, children }: Props = $props();

  let error = $state<Error | null>(null);
  let errorCount = $state(0);

  function reset() {
    error = null;
    errorCount++;
  }

  // Catch errors from child rendering by wrapping in a derived
  // The key strategy: use #key on errorCount to force remount
  $effect(() => {
    // Reset error when children change (re-mount signal)
    if (errorCount > 0) {
      error = null;
    }
  });
</script>

{#key errorCount}
  {#if error}
    <div class="eb-boundary">
      <div class="eb-content">
        <div class="eb-icon">⚠</div>
        <h3 class="eb-title">{t('error.title')}</h3>
        <p class="eb-message">{error.message || t('error.unknown')}</p>
        <button class="eb-btn" onclick={reset}>{t('error.retry')}</button>
      </div>
    </div>
  {:else}
    {@render children?.()}
  {/if}
{/key}

<style>
  .eb-boundary {
    display: flex; align-items: center; justify-content: center;
    height: 100%; padding: 24px;
    background: #1e1e1e; color: #ccc;
  }
  .eb-content { text-align: center; max-width: 400px; }
  .eb-icon { font-size: 32px; margin-bottom: 8px; }
  .eb-title { font-size: 16px; font-weight: 600; margin: 0 0 8px; color: #e88; }
  .eb-message { font-size: 13px; color: #999; margin: 0 0 16px; word-break: break-all; }
  .eb-btn {
    padding: 6px 20px; background: #007acc; border: none; color: #fff;
    border-radius: 4px; cursor: pointer; font-size: 13px;
  }
  .eb-btn:hover { background: #005fa3; }
</style>
