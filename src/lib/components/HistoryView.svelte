<script lang="ts">
  import { api, type ImportRecord } from '../api';
  import { importHistory, currentView } from '../stores';
  import { formatBytes, formatRelativeTime, pluralize } from '../format';

  let loading = $state(true);

  $effect(() => { refresh(); });

  async function refresh() {
    loading = true;
    try {
      const rows = await api.listImports(200);
      importHistory.set(rows);
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function open(r: ImportRecord) {
    currentView.set({ kind: 'import-detail', importId: r.id });
  }

  function statusLabel(s: string): string {
    if (s === 'completed') return 'Complete';
    if (s === 'completed_with_errors') return 'Some errors';
    if (s === 'cancelled') return 'Cancelled';
    if (s === 'running') return 'Running';
    return s;
  }
</script>

<div class="view">
  <header class="titlebar drag-region">
    <div class="title no-drag">History</div>
  </header>

  <div class="body">
    {#if loading}
      <div class="empty">Loading…</div>
    {:else if $importHistory.length === 0}
      <div class="empty">
        <div class="empty-title">No imports yet</div>
        <div class="empty-sub">Every file you pull in lands here, with the journal entry.</div>
      </div>
    {:else}
      <div class="list">
        {#each $importHistory as r}
          <button class="row" onclick={() => open(r)}>
            <div class="row-main">
              <div class="row-title">
                {r.card_label ?? 'Card'}
                {#if r.camera_model}<span class="dim"> · {r.camera_model}</span>{/if}
              </div>
              <div class="row-sub">
                {pluralize(r.file_count, 'file')} · {formatBytes(r.bytes)} · {formatRelativeTime(r.started_at)}
              </div>
            </div>
            <div class="row-right">
              <span class="status">{statusLabel(r.status)}</span>
              <span class="chev" aria-hidden="true">›</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .view { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
  .titlebar {
    height: var(--titlebar-h);
    padding: 0 var(--panel-pad-x);
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--divider);
    background: linear-gradient(180deg, rgba(255,255,255,0.02), transparent);
    flex-shrink: 0;
  }
  .title { font-size: 13.5px; font-weight: 600; letter-spacing: -0.005em; }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 22px 28px 28px;
    max-width: 720px;
    width: 100%;
    align-self: center;
  }

  .list {
    background: var(--bg-card);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 11px 14px;
    background: transparent;
    text-align: left;
    border-bottom: 1px solid var(--divider);
    transition: background var(--transition);
  }
  .row:last-child { border-bottom: none; }
  .row:hover { background: var(--bg-hover); }

  .row-main { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .row-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .row-sub { font-size: 11.5px; color: var(--text-secondary); }
  .dim { color: var(--text-tertiary); font-weight: 400; }

  .row-right { display: flex; align-items: center; gap: 10px; }
  .status { font-size: 11.5px; color: var(--text-secondary); }
  .chev { color: var(--text-tertiary); font-size: 16px; line-height: 1; }

  .empty {
    padding: 60px 24px;
    text-align: center;
    color: var(--text-secondary);
  }
  .empty-title {
    font-size: 14px;
    color: var(--text-primary);
    font-weight: 500;
    margin-bottom: 4px;
  }
  .empty-sub { font-size: 12.5px; }
</style>
