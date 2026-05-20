<script lang="ts">
  import { api, type Card, type FileInfo, type MediaKind, type ScanReport, type StartImportArgs } from '../api';
  import { scanByMount, settings, templates, currentView } from '../stores';
  import { formatBytes, formatDateRange, pluralize } from '../format';
  import Button from './Button.svelte';
  import Thumb from './Thumb.svelte';
  import ImportSetup from './ImportSetup.svelte';
  import Icon from './Icon.svelte';

  let { card } = $props<{ card: Card }>();

  let report = $derived($scanByMount[card.mount] ?? null);
  let isLoading = $derived(report === 'loading');
  let scanReport: ScanReport | null = $derived(
    typeof report === 'object' && report !== null ? (report as ScanReport) : null
  );
  let isCamera = $derived(card.kind === 'camera');

  // ---------- State ----------
  let searchQuery = $state('');
  type SortMode = 'date-desc' | 'date-asc';
  type ViewMode = 'grid' | 'list';
  let sortMode = $state<SortMode>('date-desc');
  let viewMode = $state<ViewMode>('grid');
  let kindFilter = $state<Set<MediaKind>>(new Set());
  let selection = $state<Set<string>>(new Set());
  let lastClickedKey: string | null = $state(null);
  let showSetup = $state(false);
  let starting = $state(false);

  // ---------- Scan kickoff ----------
  $effect(() => {
    if (!$scanByMount[card.mount]) runScan();
  });

  async function runScan() {
    scanByMount.update((m) => ({ ...m, [card.mount]: 'loading' }));
    try {
      const r = await api.scanCard(card.mount);
      scanByMount.update((m) => ({ ...m, [card.mount]: r }));
    } catch (e) {
      console.error(e);
      scanByMount.update((m) => ({ ...m, [card.mount]: 'error' }));
    }
  }

  // Select-all-by-default whenever the scan completes for the first time.
  $effect(() => {
    if (scanReport && selection.size === 0) {
      selection = new Set(scanReport.files.map(fileKey));
    }
  });

  function fileKey(f: FileInfo): string {
    return f.src_abs;
  }

  async function eject() {
    if (card.kind !== 'sd') return;
    try { await api.ejectCard(card.mount); } catch (e) { console.error(e); }
  }

  // ---------- Filtering ----------
  let filteredFiles = $derived.by((): FileInfo[] => {
    if (!scanReport) return [];
    const q = searchQuery.trim().toLowerCase();
    return scanReport.files.filter((f) => {
      if (kindFilter.size > 0 && !kindFilter.has(f.kind)) return false;
      if (q && !f.orig_name.toLowerCase().includes(q)) return false;
      return true;
    });
  });

  // ---------- Sorting ----------
  function fileTime(f: FileInfo): number {
    const t = f.shot_at ?? f.mtime;
    return t ? new Date(t).getTime() : 0;
  }

  function compareFiles(a: FileInfo, b: FileInfo, mode: SortMode): number {
    switch (mode) {
      case 'date-desc': return fileTime(b) - fileTime(a) || a.orig_name.localeCompare(b.orig_name);
      case 'date-asc':  return fileTime(a) - fileTime(b) || a.orig_name.localeCompare(b.orig_name);
    }
  }

  // ---------- Grouping (always by date) ----------
  type Group = { key: string; label: string; files: FileInfo[] };

  let groups = $derived.by((): Group[] => {
    if (!scanReport) return [];
    const list = [...filteredFiles].sort((a, b) => compareFiles(a, b, sortMode));

    const buckets = new Map<string, FileInfo[]>();
    for (const f of list) {
      const key = dateGroupKey(f);
      let arr = buckets.get(key);
      if (!arr) { arr = []; buckets.set(key, arr); }
      arr.push(f);
    }

    const out: Group[] = [];
    for (const [key, files] of buckets) {
      out.push({ key, label: dateGroupLabel(key), files });
    }
    out.sort((a, b) => {
      if (a.key === '__nodate__') return 1;
      if (b.key === '__nodate__') return -1;
      const cmp = b.key.localeCompare(a.key); // YYYY-MM-DD desc
      return sortMode === 'date-asc' ? -cmp : cmp;
    });
    return out;
  });

  function dateGroupKey(f: FileInfo): string {
    const t = f.shot_at ?? f.mtime;
    if (!t) return '__nodate__';
    const d = new Date(t);
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  function dateGroupLabel(key: string): string {
    if (key === '__nodate__') return 'No date';
    return formatDateHeader(key);
  }

  function formatDateHeader(ymd: string): string {
    const [y, m, d] = ymd.split('-').map(Number);
    const dt = new Date(y, m - 1, d);
    const today = new Date(); today.setHours(0, 0, 0, 0);
    const yesterday = new Date(today); yesterday.setDate(today.getDate() - 1);
    if (dt.getTime() === today.getTime()) return 'Today';
    if (dt.getTime() === yesterday.getTime()) return 'Yesterday';
    const sameYear = dt.getFullYear() === today.getFullYear();
    return dt.toLocaleDateString(undefined, {
      weekday: 'long',
      month: 'long',
      day: 'numeric',
      year: sameYear ? undefined : 'numeric',
    });
  }

  // ---------- Selection ----------
  let totalFiles = $derived(scanReport?.files.length ?? 0);
  let selectedCount = $derived(selection.size);
  let selectedBytes = $derived.by((): number => {
    if (!scanReport) return 0;
    let sum = 0;
    for (const f of scanReport.files) if (selection.has(fileKey(f))) sum += f.bytes;
    return sum;
  });

  // ordered list of currently visible keys, used for shift-range selection
  let visibleKeys = $derived(groups.flatMap((g) => g.files.map(fileKey)));

  function toggle(f: FileInfo, e: MouseEvent) {
    const k = fileKey(f);
    if (e.shiftKey && lastClickedKey && lastClickedKey !== k) {
      // Range select within visible order
      const a = visibleKeys.indexOf(lastClickedKey);
      const b = visibleKeys.indexOf(k);
      if (a >= 0 && b >= 0) {
        const [lo, hi] = a < b ? [a, b] : [b, a];
        const turnOn = !selection.has(k);
        const next = new Set(selection);
        for (let i = lo; i <= hi; i++) {
          if (turnOn) next.add(visibleKeys[i]);
          else next.delete(visibleKeys[i]);
        }
        selection = next;
      }
    } else {
      const next = new Set(selection);
      if (next.has(k)) next.delete(k);
      else next.add(k);
      selection = next;
    }
    lastClickedKey = k;
  }

  function selectAllVisible() {
    const next = new Set(selection);
    for (const k of visibleKeys) next.add(k);
    selection = next;
  }
  function deselectAllVisible() {
    const next = new Set(selection);
    for (const k of visibleKeys) next.delete(k);
    selection = next;
  }
  function selectGroup(g: Group) {
    const next = new Set(selection);
    const allSelected = g.files.every((f) => next.has(fileKey(f)));
    if (allSelected) for (const f of g.files) next.delete(fileKey(f));
    else for (const f of g.files) next.add(fileKey(f));
    selection = next;
  }

  function isGroupFullySelected(g: Group): boolean {
    return g.files.every((f) => selection.has(fileKey(f)));
  }

  // Kind filter chips
  function toggleKind(k: MediaKind) {
    const next = new Set(kindFilter);
    if (next.has(k)) next.delete(k);
    else next.add(k);
    kindFilter = next;
  }

  // ---------- Keyboard ----------
  let searchInput: HTMLInputElement | null = $state(null);

  function onKey(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    const isField = t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT');

    // Cmd/Ctrl + F → focus search (works even inside other inputs)
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      searchInput?.focus();
      searchInput?.select();
      return;
    }

    if (isField) return;

    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'a') {
      e.preventDefault();
      if (selection.size === visibleKeys.length) deselectAllVisible();
      else selectAllVisible();
    } else if ((e.metaKey || e.ctrlKey) && e.key === '1') {
      e.preventDefault();
      viewMode = 'grid';
    } else if ((e.metaKey || e.ctrlKey) && e.key === '2') {
      e.preventDefault();
      viewMode = 'list';
    } else if (e.key === '/') {
      e.preventDefault();
      searchInput?.focus();
    } else if (e.key === 'Escape') {
      if (selection.size > 0) selection = new Set();
    }
  }

  // ---------- Direct import (skip the setup screen) ----------
  let defaultDest = $derived($settings?.default_dest ?? '');
  let defaultTemplate = $derived(
    $templates.find((t) => t.is_default) ?? $templates[0] ?? null
  );
  let defaultCollision = $derived(
    ($settings?.collision_policy ?? 'rename') as 'skip' | 'rename' | 'overwrite'
  );

  async function startDirectImport() {
    if (!filteredReport || !defaultDest || !defaultTemplate) return;
    starting = true;
    const args: StartImportArgs = {
      card_label: card.label,
      card_mount: card.mount,
      camera_model: card.camera_model,
      dest_root: defaultDest,
      pattern: defaultTemplate.pattern,
      collision: defaultCollision,
      verify_hash: $settings?.verify_hash ?? false,
      worker_count: $settings?.worker_count ?? null,
    };
    try {
      const id = await api.startImport(args, filteredReport);
      currentView.set({ kind: 'progress', importId: id });
    } catch (e) {
      console.error(e);
    } finally {
      starting = false;
    }
  }

  function shortenPath(p: string): string {
    if (!p) return '';
    if (p.startsWith('/Users/')) {
      const segs = p.split('/');
      if (segs.length >= 3) return '~/' + segs.slice(3).join('/');
    }
    return p;
  }

  // ---------- Import handoff (overlay for "Change…") ----------
  let filteredReport = $derived.by((): ScanReport | null => {
    if (!scanReport) return null;
    const picked = scanReport.files.filter((f) => selection.has(fileKey(f)));
    let photo_count = 0, raw_count = 0, video_count = 0, audio_count = 0, sidecar_count = 0, other_count = 0;
    let total_bytes = 0;
    let earliest: string | null = null;
    let latest: string | null = null;
    for (const f of picked) {
      total_bytes += f.bytes;
      if (f.kind === 'photo') photo_count++;
      else if (f.kind === 'raw') raw_count++;
      else if (f.kind === 'video') video_count++;
      else if (f.kind === 'audio') audio_count++;
      else if (f.kind === 'sidecar') sidecar_count++;
      else other_count++;
      const t = f.shot_at ?? f.mtime;
      if (t) {
        if (!earliest || t < earliest) earliest = t;
        if (!latest  || t > latest)  latest = t;
      }
    }
    return {
      ...scanReport,
      files: picked,
      total_bytes,
      photo_count, raw_count, video_count, audio_count, sidecar_count, other_count,
      earliest, latest,
    };
  });

  // Kind chips available — only show kinds present in this scan
  let availableKinds = $derived.by((): { kind: MediaKind; count: number }[] => {
    if (!scanReport) return [];
    const order: MediaKind[] = ['photo', 'raw', 'video', 'audio', 'sidecar', 'other'];
    return order
      .map((k) => ({ kind: k, count: scanReport!.files.filter((f) => f.kind === k).length }))
      .filter((x) => x.count > 0);
  });

  function kindShortLabel(k: MediaKind): string {
    return { photo: 'Photos', raw: 'Raw', video: 'Videos', audio: 'Audio', sidecar: 'Sidecars', other: 'Other' }[k];
  }

  function fileDateLabel(f: FileInfo): string {
    const t = f.shot_at ?? f.mtime;
    if (!t) return 'No date';
    return new Date(t).toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  }

  function kindLabel(k: MediaKind): string {
    return { photo: 'Photo', raw: 'Raw', video: 'Video', audio: 'Audio', sidecar: 'Sidecar', other: 'Other' }[k];
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="panel">
  <header class="titlebar" data-tauri-drag-region>
    <div class="title">{card.label}</div>
    <div class="actions">
      {#if isCamera}
        <span class="port-pill">{card.port}</span>
      {:else}
        <Button variant="ghost" size="sm" onclick={eject}>Eject</Button>
      {/if}
    </div>
  </header>

  {#if isLoading}
    <div class="state">
      <span class="spinner"></span>
      <span class="muted">
        {isCamera ? 'Reading the camera over USB…' : 'Scanning card…'}
      </span>
    </div>
  {:else if scanReport && showSetup && filteredReport}
    <div class="setup-wrap">
      <ImportSetup card={card} report={filteredReport} onclose={() => (showSetup = false)} />
    </div>
  {:else if scanReport}
    <div class="hero">
      <div class="hero-eyebrow">{isCamera ? 'Camera over USB' : 'SD card'}</div>
      <div class="hero-title">{card.camera_model ?? card.label}</div>
      <div class="hero-sub">
        {pluralize(scanReport.files.length, 'file')} · {formatBytes(scanReport.total_bytes)}
        {#if scanReport.earliest}
          <span class="sep">·</span>
          <span class="dim">{formatDateRange(scanReport.earliest, scanReport.latest)}</span>
        {/if}
      </div>
    </div>

    <div class="toolbar">
      <div class="search-wrap">
        <span class="search-icon"><Icon name="search" size={13} stroke={2} /></span>
        <input
          bind:this={searchInput}
          class="search"
          type="text"
          bind:value={searchQuery}
          placeholder="Search filenames"
          spellcheck="false"
        />
        {#if searchQuery}
          <button class="clear" onclick={() => (searchQuery = '')} aria-label="Clear">×</button>
        {/if}
      </div>

      <div class="toolbar-spacer"></div>

      <button
        type="button"
        class="tool-btn"
        onclick={() => (sortMode = sortMode === 'date-desc' ? 'date-asc' : 'date-desc')}
        title="Toggle sort direction"
      >
        <Icon name={sortMode === 'date-desc' ? 'sort-desc' : 'sort-asc'} size={13} stroke={2} />
        <span>{sortMode === 'date-desc' ? 'Newest' : 'Oldest'}</span>
      </button>

      <div class="view-switch" data-mode={viewMode} aria-label="View mode">
        <div class="view-indicator" aria-hidden="true"></div>
        <button
          type="button"
          class:on={viewMode === 'grid'}
          onclick={() => (viewMode = 'grid')}
          title="Grid"
          aria-label="Grid"
        >
          <Icon name="grid" size={13} stroke={1.9} />
        </button>
        <button
          type="button"
          class:on={viewMode === 'list'}
          onclick={() => (viewMode = 'list')}
          title="List"
          aria-label="List"
        >
          <Icon name="list" size={14} stroke={1.9} />
        </button>
      </div>
    </div>

    {#if availableKinds.length > 1}
      <div class="chips" role="tablist" aria-label="Filter by kind">
        <button
          type="button"
          class="chip"
          class:on={kindFilter.size === 0}
          onclick={() => (kindFilter = new Set())}
          role="tab"
          aria-selected={kindFilter.size === 0}
        >
          <span>All</span>
          <span class="chip-count">{scanReport.files.length.toLocaleString()}</span>
        </button>
        {#each availableKinds as k}
          <button
            type="button"
            class="chip"
            class:on={kindFilter.has(k.kind)}
            onclick={() => toggleKind(k.kind)}
            role="tab"
            aria-selected={kindFilter.has(k.kind)}
          >
            <span>{kindShortLabel(k.kind)}</span>
            <span class="chip-count">{k.count.toLocaleString()}</span>
          </button>
        {/each}
      </div>
    {/if}

    {#if filteredFiles.length === 0}
      <div class="empty-scan muted">
        {scanReport.files.length === 0
          ? "No files on this card."
          : "No files match the filters."}
      </div>
    {:else}
      <div class="grid-scroll" class:list-mode={viewMode === 'list'}>
        {#each groups as g (g.key)}
          {#if g.label}
            <div class="group-head">
              <div class="group-title">
                <span>{g.label}</span>
                <span class="group-count">{pluralize(g.files.length, 'item')}</span>
              </div>
              <button class="group-select" onclick={() => selectGroup(g)}>
                {isGroupFullySelected(g) ? 'Deselect' : 'Select all'}
              </button>
            </div>
          {/if}
          {#if viewMode === 'list'}
            <div class="file-list">
              {#each g.files as f (f.src_abs)}
                <button
                  type="button"
                  class="file-row"
                  class:selected={selection.has(fileKey(f))}
                  onclick={(e) => toggle(f, e)}
                  title={f.src_rel}
                >
                  <span class="row-check" aria-hidden="true">
                    {#if selection.has(fileKey(f))}<span>✓</span>{/if}
                  </span>
                  <span class="row-name">{f.orig_name}</span>
                  <span class="row-kind">{kindLabel(f.kind)}</span>
                  <span class="row-date">{fileDateLabel(f)}</span>
                  <span class="row-size">{formatBytes(f.bytes)}</span>
                </button>
              {/each}
            </div>
          {:else}
            <div class="grid">
              {#each g.files as f (f.src_abs)}
                <div class="cell">
                  <Thumb
                    src={f.src_abs}
                    kind={f.kind}
                    name={f.orig_name}
                    selected={selection.has(fileKey(f))}
                    isVideo={f.kind === 'video'}
                    fit="contain"
                    onClick={(e) => toggle(f, e)}
                  />
                  <div class="cell-caption" title={f.orig_name}>{f.orig_name}</div>
                </div>
              {/each}
            </div>
          {/if}
        {/each}
      </div>
    {/if}

    <footer class="bar">
      <div class="bar-left">
        <div class="bar-count-line">
          <span class="bar-count">
            <strong>{selectedCount.toLocaleString()}</strong> of {totalFiles.toLocaleString()} selected
          </span>
          <span class="sep">·</span>
          <span class="dim">{formatBytes(selectedBytes)}</span>
          {#if selection.size > 0 && selection.size < totalFiles}
            <button class="link" onclick={() => (selection = new Set(scanReport!.files.map(fileKey)))}>
              Select all
            </button>
          {:else if selection.size === totalFiles}
            <button class="link" onclick={() => (selection = new Set())}>Deselect all</button>
          {/if}
        </div>
        {#if defaultDest && selectedCount > 0}
          <div class="dest-line">
            <span class="dim">to</span>
            <span class="mono">{shortenPath(defaultDest)}</span>
            <button class="link" onclick={() => (showSetup = true)}>Change…</button>
          </div>
        {/if}
      </div>
      <div class="bar-right">
        <Button
          variant="primary"
          size="md"
          disabled={selectedCount === 0 || starting}
          loading={starting}
          onclick={startDirectImport}
        >
          {selectedCount === 0
            ? 'Choose items to import'
            : `Import ${selectedCount.toLocaleString()} ${selectedCount === 1 ? 'item' : 'items'}`}
        </Button>
      </div>
    </footer>
  {:else if report === 'error'}
    <div class="state muted">
      Couldn't read {isCamera ? 'the camera' : 'the card'}. Unplug and try again.
    </div>
  {/if}
</div>

<style>
  .panel {
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
  .title {
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: -0.005em;
  }
  .actions { display: flex; gap: 6px; align-items: center; }

  .port-pill {
    font-size: 10.5px;
    color: var(--text-secondary);
    background: var(--bg-surface);
    border: 1px solid var(--divider);
    padding: 2px 8px;
    border-radius: 4px;
    font-family: var(--font-mono);
  }

  .hero {
    padding: 18px var(--panel-pad-x) 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
    min-width: 0;
    animation: fade-in-up 320ms var(--ease-out) both;
  }
  @keyframes fade-in-up {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .hero-eyebrow {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
    font-weight: 600;
  }
  .hero-title {
    font-size: 22px;
    font-weight: 600;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .hero-sub {
    margin-top: 3px;
    font-size: 12.5px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sep { opacity: 0.5; margin: 0 4px; }
  .dim { color: var(--text-tertiary); }

  .toolbar {
    padding: 14px var(--panel-pad-x) 6px;
    display: flex;
    gap: 8px;
    align-items: center;
    flex-shrink: 0;
    flex-wrap: wrap;
    row-gap: 8px;
  }

  .toolbar-spacer { flex: 1; }

  .search-wrap {
    position: relative;
    flex: 0 1 260px;
    min-width: 180px;
  }
  .search-icon {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-tertiary);
    pointer-events: none;
  }
  .search {
    width: 100%;
    height: 26px;
    padding: 0 26px 0 26px;
    font-size: 12.5px;
    border-radius: 6px;
    background: var(--bg-surface);
    border-color: transparent;
    transition: background var(--transition), border var(--transition), box-shadow var(--transition);
  }
  .search:focus {
    background: var(--bg-input);
    border-color: var(--accent);
  }
  .clear {
    position: absolute;
    right: 5px;
    top: 50%;
    transform: translateY(-50%);
    width: 16px;
    height: 16px;
    border-radius: 50%;
    color: var(--text-tertiary);
    background: var(--bg-surface-2);
    font-size: 13px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .clear:hover { color: var(--text-primary); }

  .tool-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 10px;
    font-size: 12px;
    color: var(--text-secondary);
    background: var(--bg-surface);
    border-radius: 6px;
    box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.04);
    transition:
      background 130ms var(--ease-snap),
      color 130ms var(--ease-snap),
      box-shadow 130ms var(--ease-snap),
      transform 180ms var(--ease-spring);
    flex-shrink: 0;
  }
  .tool-btn :global(.icon) { color: var(--text-tertiary); transition: color 130ms var(--ease-snap); }
  .tool-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.08);
  }
  .tool-btn:hover :global(.icon) { color: var(--text-secondary); }
  .tool-btn:active {
    transform: scale(0.96);
    transition-duration: 80ms;
    box-shadow:
      inset 0 1px 1px rgba(0, 0, 0, 0.08),
      inset 0 0 0 0.5px rgba(0, 0, 0, 0.08);
  }

  .view-switch {
    position: relative;
    display: inline-flex;
    height: 26px;
    padding: 2px;
    border-radius: 7px;
    background: var(--bg-surface);
    box-shadow: inset 0 0 0 0.5px rgba(0, 0, 0, 0.05);
    flex-shrink: 0;
  }
  .view-indicator {
    position: absolute;
    top: 2px;
    bottom: 2px;
    left: 2px;
    width: 28px;
    border-radius: 5px;
    background: var(--bg-card);
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.10),
      0 0 0 0.5px rgba(0, 0, 0, 0.06),
      inset 0 1px 0 rgba(255, 255, 255, 0.80);
    transition: transform 260ms var(--ease-spring);
    pointer-events: none;
  }
  @media (prefers-color-scheme: dark) {
    .view-indicator {
      box-shadow:
        0 1px 2px rgba(0, 0, 0, 0.34),
        0 0 0 0.5px rgba(0, 0, 0, 0.40),
        inset 0 1px 0 rgba(255, 255, 255, 0.06);
    }
  }
  .view-switch[data-mode="list"] .view-indicator { transform: translateX(28px); }
  .view-switch button {
    position: relative;
    z-index: 1;
    width: 28px;
    display: grid;
    place-items: center;
    border-radius: 5px;
    color: var(--text-tertiary);
    transition:
      color 180ms var(--ease-snap),
      transform 180ms var(--ease-spring);
  }
  .view-switch button:hover { color: var(--text-secondary); }
  .view-switch button.on { color: var(--text-primary); }
  .view-switch button:active { transform: scale(0.92); transition-duration: 90ms; }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 6px var(--panel-pad-x) 2px;
    flex-shrink: 0;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 11.5px;
    background: transparent;
    color: var(--text-secondary);
    transition:
      background 160ms var(--ease-snap),
      color 160ms var(--ease-snap),
      transform 200ms var(--ease-spring),
      box-shadow 160ms var(--ease-snap);
    line-height: 1.2;
  }
  .chip:hover { background: var(--bg-hover); color: var(--text-primary); }
  .chip.on {
    background: var(--bg-selected);
    color: var(--accent);
    box-shadow: inset 0 0 0 0.5px color-mix(in srgb, var(--accent) 35%, transparent);
  }
  .chip:active { transform: scale(0.94); transition-duration: 90ms; }
  .chip-count {
    font-variant-numeric: tabular-nums;
    color: var(--text-tertiary);
    font-size: 11px;
  }
  .chip.on .chip-count { color: var(--accent); opacity: 0.72; }

  .grid-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0 20px;
    min-height: 0;
  }
  .grid-scroll.list-mode {
    padding-bottom: 10px;
  }

  .group-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 14px var(--panel-pad-x) 10px;
    position: sticky;
    top: 0;
    background: var(--bg-content);
    border-bottom: 1px solid var(--divider);
    z-index: 3;
    animation: fade-in-up 280ms var(--ease-out) both;
  }
  .group-title {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
    overflow: hidden;
  }
  .group-title > span:first-child {
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: -0.005em;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .group-count {
    font-size: 11.5px;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }
  .group-select {
    font-size: 11.5px;
    color: var(--accent);
    padding: 2px 8px;
    border-radius: 4px;
    transition:
      background 140ms var(--ease-snap),
      transform 180ms var(--ease-spring);
    flex-shrink: 0;
  }
  .group-select:hover { background: var(--bg-hover); }
  .group-select:active { transform: scale(0.94); transition-duration: 80ms; }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(136px, 1fr));
    gap: 14px 10px;
    padding: 0 var(--panel-pad-x);
  }
  .cell {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .cell-caption {
    font-size: 11px;
    line-height: 1.25;
    color: var(--text-secondary);
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 2px;
    font-variant-numeric: tabular-nums;
  }

  .file-list {
    margin: 0 var(--panel-pad-x);
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--bg-card);
  }
  .file-row {
    width: 100%;
    display: grid;
    grid-template-columns: 20px minmax(160px, 1fr) 76px 150px 82px;
    gap: 12px;
    align-items: center;
    min-height: 34px;
    padding: 6px 12px;
    text-align: left;
    border-bottom: 1px solid var(--divider);
    color: var(--text-primary);
    transition: background var(--transition);
  }
  .file-row:last-child { border-bottom: none; }
  .file-row:hover { background: var(--bg-hover); }
  .file-row.selected { background: var(--bg-selected); }
  .row-check {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 1px solid var(--divider-strong);
    display: grid;
    place-items: center;
    color: white;
    font-size: 11px;
    line-height: 1;
  }
  .file-row.selected .row-check {
    background: var(--accent);
    border-color: var(--accent);
  }
  .row-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12.5px;
    font-weight: 500;
  }
  .row-kind,
  .row-date,
  .row-size {
    font-size: 11.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row-size {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  @media (max-width: 960px) {
    .file-row {
      grid-template-columns: 20px minmax(120px, 1fr) 70px 76px;
    }
    .row-date { display: none; }
  }

  .empty-scan {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: var(--text-secondary);
  }

  .bar {
    flex-shrink: 0;
    min-height: 58px;
    padding: 10px var(--panel-pad-x);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    background: rgba(245, 245, 247, 0.85);
    backdrop-filter: saturate(180%) blur(22px);
    -webkit-backdrop-filter: saturate(180%) blur(22px);
    border-top: 1px solid var(--divider);
  }
  @media (prefers-color-scheme: dark) {
    .bar {
      background: rgba(28, 28, 30, 0.80);
    }
  }
  .bar-left {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .bar-count-line {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-size: 12.5px;
    color: var(--text-secondary);
    flex-wrap: wrap;
    row-gap: 4px;
  }
  .bar-count {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .bar-count strong { color: var(--text-primary); font-weight: 600; }
  .dest-line {
    display: flex;
    align-items: baseline;
    gap: 6px;
    font-size: 11.5px;
    color: var(--text-tertiary);
    min-width: 0;
    overflow: hidden;
  }
  .dest-line .mono {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .bar-right { flex-shrink: 0; }
  .link {
    color: var(--accent);
    font-size: 12px;
    padding: 2px 6px;
    border-radius: 4px;
    margin-left: 4px;
    transition: background var(--transition);
  }
  .link:hover { background: var(--bg-hover); }

  .setup-wrap {
    flex: 1;
    overflow-y: auto;
    padding: 22px 28px 28px;
    max-width: 680px;
    width: 100%;
    align-self: center;
  }

  .state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: 10px;
    padding: 40px 0;
    color: var(--text-secondary);
  }
  .spinner {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid var(--divider-strong);
    border-top-color: var(--text-secondary);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .muted { color: var(--text-secondary); }
</style>
