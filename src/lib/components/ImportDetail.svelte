<script lang="ts">
  import { api, type ImportFileRow, type ImportEventRow, type ImportRecord } from '../api';
  import { importHistory, currentView } from '../stores';
  import { formatBytes, formatRelativeTime, pluralize } from '../format';
  import Button from './Button.svelte';

  let { importId } = $props<{ importId: string }>();

  let record = $state<ImportRecord | null>(null);
  let files = $state<ImportFileRow[]>([]);
  let events = $state<ImportEventRow[]>([]);
  let loading = $state(true);
  let tab = $state<'files' | 'events'>('files');

  $effect(() => {
    load();
  });

  async function load() {
    loading = true;
    try {
      const [imports, f, ev] = await Promise.all([
        api.listImports(200),
        api.getImportFiles(importId),
        api.getImportEvents(importId),
      ]);
      record = imports.find((r) => r.id === importId) ?? null;
      files = f;
      events = ev;
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function back() {
    currentView.set({ kind: 'history' });
  }

  function revealDest() {
    if (record) api.revealInFinder(record.dest_root);
  }
</script>

<div class="detail">
  <header class="head drag-region">
    <div class="head-inner no-drag">
      <div>
        <button class="back" onclick={back}>← History</button>
        <div class="head-title">
          {#if record}
            {record.card_label ?? 'Card'}
            {#if record.camera_model}<span class="dim"> · {record.camera_model}</span>{/if}
          {:else}
            Import
          {/if}
        </div>
        {#if record}
          <div class="head-sub">
            {pluralize(record.file_count, 'file')} · {formatBytes(record.bytes)} · {formatRelativeTime(record.started_at)}
          </div>
        {/if}
      </div>
      {#if record}
        <Button variant="secondary" onclick={revealDest}>Show in Finder</Button>
      {/if}
    </div>
  </header>

  <div class="body">
    {#if record}
      <div class="meta-grid">
        <div>
          <div class="meta-label">Destination</div>
          <div class="meta-value mono">{record.dest_root}</div>
        </div>
        <div>
          <div class="meta-label">Template</div>
          <div class="meta-value mono">{record.template_pattern}</div>
        </div>
        <div>
          <div class="meta-label">Status</div>
          <div class="meta-value">{record.status}</div>
        </div>
      </div>
    {/if}

    <div class="tabs">
      <button class:on={tab === 'files'} onclick={() => (tab = 'files')}>Files <span class="count">{files.length}</span></button>
      <button class:on={tab === 'events'} onclick={() => (tab = 'events')}>Events <span class="count">{events.length}</span></button>
    </div>

    {#if loading}
      <div class="empty">Loading…</div>
    {:else if tab === 'files'}
      <div class="table">
        <div class="thead">
          <div class="c-status"></div>
          <div class="c-src">Source</div>
          <div class="c-dst">Destination</div>
          <div class="c-bytes">Size</div>
        </div>
        {#each files as f}
          <div class="trow" class:err={f.status === 'failed'}>
            <div class="c-status">
              {#if f.status === 'ok'}<span class="tick">✓</span>
              {:else if f.status === 'skipped'}<span class="skip">–</span>
              {:else if f.status === 'failed'}<span class="x">×</span>
              {:else}<span class="muted">{f.status}</span>{/if}
            </div>
            <div class="c-src mono" title={f.src_abs}>{f.src_rel}</div>
            <div class="c-dst mono" title={f.dst_abs}>{shortPath(f.dst_abs)}</div>
            <div class="c-bytes tabular">{formatBytes(f.bytes)}</div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="events">
        {#each events as e}
          <div class="evrow">
            <span class="lvl {e.level}">{e.level}</span>
            <span class="msg">{e.message}</span>
            <span class="ts dim">{formatRelativeTime(e.ts)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<script module lang="ts">
  export function shortPath(p: string): string {
    const parts = p.split('/');
    if (parts.length <= 3) return p;
    return `…/${parts.slice(-3).join('/')}`;
  }
</script>

<style>
  .detail { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
  .head { flex-shrink: 0; padding-top: var(--titlebar-h); border-bottom: 1px solid var(--divider); }
  .head-inner {
    padding: 14px var(--panel-pad-x) 18px;
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }
  .back {
    color: var(--accent);
    font-size: 12px;
    margin-bottom: 6px;
    padding: 2px 0;
    transition: color var(--transition);
  }
  .back:hover { color: var(--accent-hover); }
  .head-title {
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.01em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .head-sub { margin-top: 2px; font-size: 12.5px; color: var(--text-secondary); }
  .dim { color: var(--text-tertiary); font-weight: 400; }

  .body { flex: 1; overflow-y: auto; padding: 18px var(--panel-pad-x) 28px; display: flex; flex-direction: column; gap: 18px; }

  .meta-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 14px;
    padding: 14px 16px;
    background: var(--bg-card);
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
  }
  .meta-label { font-size: 10.5px; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 4px; }
  .meta-value { font-size: 12.5px; color: var(--text-primary); }
  .mono { font-family: var(--font-mono); font-size: 11.5px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .tabs {
    display: flex;
    gap: 4px;
    border-bottom: 1px solid var(--divider);
  }
  .tabs button {
    padding: 6px 12px;
    font-size: 12.5px;
    color: var(--text-secondary);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color var(--transition), border var(--transition);
  }
  .tabs button.on {
    color: var(--text-primary);
    border-bottom-color: var(--accent);
  }
  .count { color: var(--text-tertiary); font-size: 11px; margin-left: 4px; }

  .table {
    background: var(--bg-card);
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .thead, .trow {
    display: grid;
    grid-template-columns: 24px minmax(140px, 1fr) minmax(180px, 1.4fr) 80px;
    gap: 12px;
    padding: 6px 14px;
    font-size: 11.5px;
    align-items: center;
  }
  .thead {
    color: var(--text-tertiary);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--divider);
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .trow {
    border-bottom: 1px solid var(--divider);
    color: var(--text-primary);
  }
  .trow:last-child { border-bottom: none; }
  .trow.err { background: rgba(255, 69, 58, 0.04); }

  .tick { color: var(--success); }
  .skip { color: var(--text-tertiary); }
  .x { color: var(--danger); }

  .events {
    background: var(--bg-card);
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .evrow {
    display: grid;
    grid-template-columns: 60px 1fr auto;
    gap: 12px;
    padding: 8px 14px;
    font-size: 12px;
    border-bottom: 1px solid var(--divider);
  }
  .evrow:last-child { border-bottom: none; }
  .lvl {
    font-size: 10px;
    text-transform: uppercase;
    color: var(--text-tertiary);
    letter-spacing: 0.05em;
  }
  .lvl.info { color: var(--accent); }
  .lvl.warn { color: var(--warning); }
  .lvl.error { color: var(--danger); }
  .ts { font-size: 11px; }

  .empty { padding: 60px; text-align: center; color: var(--text-secondary); }
  .muted { color: var(--text-tertiary); }
  .c-src, .c-dst { overflow: hidden; }
  .c-bytes { text-align: right; }
</style>
