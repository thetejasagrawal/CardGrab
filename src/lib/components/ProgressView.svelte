<script lang="ts">
  import { api } from '../api';
  import { activeProgress, lastCompleted, currentView } from '../stores';
  import { formatBytes, formatThroughput, formatDuration } from '../format';
  import Button from './Button.svelte';

  let { importId } = $props<{ importId: string }>();
  let cancelling = $state(false);

  let progress = $derived($activeProgress);
  let completed = $derived(
    $lastCompleted && $lastCompleted.importId === importId ? $lastCompleted : null
  );

  let pct = $derived.by(() => {
    if (completed) return 100;
    if (!progress || progress.bytes_total === 0) return 0;
    return Math.min(100, (progress.bytes_done / progress.bytes_total) * 100);
  });

  async function cancel() {
    cancelling = true;
    try { await api.cancelImport(importId); } catch {}
  }
  function goHistory() { currentView.set({ kind: 'import-detail', importId }); }
</script>

<div class="view">
  <header class="titlebar drag-region">
    <div class="title no-drag">
      {#if completed}
        {#if completed.status === 'completed'}All in.
        {:else if completed.status === 'cancelled'}Stopped
        {:else}Finished with errors
        {/if}
      {:else}
        Importing
      {/if}
    </div>
    <div class="actions no-drag">
      {#if completed}
        <Button variant="ghost" size="sm" onclick={goHistory}>View details</Button>
      {:else}
        <Button variant="ghost" size="sm" onclick={cancel} loading={cancelling}>Cancel</Button>
      {/if}
    </div>
  </header>

  <div class="body">
    <div class="head">
      <div class="head-title">
        {#if completed}
          {completed.fileCount.toLocaleString()} files · {formatBytes(completed.bytesTotal)}
          {#if completed.failures > 0}<span class="warn">· {completed.failures} failed</span>{/if}
        {:else if progress}
          File {progress.file_index + 1} of {progress.file_total}
        {/if}
      </div>
    </div>

    <div class="bar-wrap">
      <div class="bar">
        <div
          class="fill"
          style="width: {pct}%"
          class:done={completed && completed.status === 'completed'}
          class:err={completed && completed.status !== 'completed' && completed.status !== 'cancelled'}
        ></div>
      </div>
      <div class="bar-meta">
        <div class="pct tabular">{pct.toFixed(1)}%</div>
        {#if !completed && progress}
          <div class="meta-right">
            <span class="muted">{formatBytes(progress.bytes_done)} / {formatBytes(progress.bytes_total)}</span>
            <span class="sep">·</span>
            <span>{formatThroughput(progress.throughput_bps)}</span>
            <span class="sep">·</span>
            <span class="muted">ETA {formatDuration(progress.eta_seconds)}</span>
          </div>
        {/if}
      </div>
    </div>

    {#if !completed && progress}
      <div class="now">
        <div class="now-label">Current file</div>
        <div class="now-file mono">{progress.file_name}</div>
      </div>
    {/if}

    {#if completed && completed.status === 'completed'}
      <div class="done">
        <div class="done-symbol">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" fill="currentColor" fill-opacity="0.10"/>
            <path d="m8 12 3 3 5-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <div class="done-title">Everything's on your Mac</div>
        <div class="done-sub">The card was never written to.</div>
        <div class="done-actions">
          <Button variant="glass" size="md" onclick={goHistory}>Open import</Button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .view {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .titlebar {
    height: var(--titlebar-h);
    padding: 0 var(--panel-pad-x);
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--divider);
    background: linear-gradient(180deg, rgba(255,255,255,0.02), transparent);
    flex-shrink: 0;
  }
  .title { font-size: 13.5px; font-weight: 600; letter-spacing: -0.005em; }
  .actions { display: flex; gap: 6px; }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 28px var(--panel-pad-x) 32px;
    max-width: 680px;
    width: 100%;
    align-self: center;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  .head-title { font-size: 13px; color: var(--text-secondary); }
  .warn { color: var(--warning); margin-left: 4px; }

  .bar-wrap { display: flex; flex-direction: column; gap: 10px; }
  .bar {
    height: 6px;
    border-radius: 3px;
    background: var(--bg-surface-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 200ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  .fill.done { background: var(--success); }
  .fill.err  { background: var(--warning); }
  .bar-meta {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 12px;
  }
  .pct {
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.015em;
  }
  .meta-right { display: flex; gap: 6px; align-items: center; color: var(--text-primary); }
  .sep { opacity: 0.4; }

  .now {
    background: var(--bg-card);
    border-radius: var(--radius-md);
    padding: 12px 14px;
  }
  .now-label {
    font-size: 10.5px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin-bottom: 3px;
  }
  .now-file {
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mono { font-family: var(--font-mono); font-size: 12px; }

  .done {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 4px;
    padding: 18px 0;
  }
  .done-symbol { color: var(--success); }
  .done-title { font-size: 16px; font-weight: 600; letter-spacing: -0.01em; margin-top: 6px; }
  .done-sub { color: var(--text-secondary); font-size: 12.5px; }
  .done-actions { margin-top: 14px; }

  .muted { color: var(--text-secondary); }
</style>
