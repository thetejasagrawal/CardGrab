<script lang="ts">
  import { api, type Card, type ScanReport, type StartImportArgs, type TemplateRow } from '../api';
  import { settings, templates, currentView, activeProgress } from '../stores';
  import { formatBytes, pluralize } from '../format';
  import Button from './Button.svelte';

  let { card, report, onclose } = $props<{
    card: Card;
    report: ScanReport;
    onclose: () => void;
  }>();

  let destRoot = $state<string>('');
  let selectedTemplateId = $state<string>('');
  let customPattern = $state<string>('');
  let usingCustom = $state(false);
  let collision = $state<'rename' | 'skip' | 'overwrite'>('rename');
  let previews = $state<string[]>([]);
  let starting = $state(false);

  $effect(() => {
    if ($settings && !destRoot) destRoot = $settings.default_dest;
  });

  $effect(() => {
    if ($templates.length && !selectedTemplateId && !usingCustom) {
      const def = $templates.find((t) => t.is_default) ?? $templates[0];
      selectedTemplateId = def.id;
      customPattern = def.pattern;
    }
  });

  let activePattern = $derived(
    usingCustom
      ? customPattern
      : $templates.find((t) => t.id === selectedTemplateId)?.pattern ?? ''
  );

  $effect(() => {
    if (!activePattern || !report.files.length) {
      previews = [];
      return;
    }
    // sample first few files of each kind for representative preview
    const sample = pickSampleFiles(report.files, 5);
    api.renderTemplatePreview(activePattern, sample, sample.length).then((p) => (previews = p));
  });

  function pickSampleFiles(all: ScanReport['files'], n: number) {
    const byKind: Record<string, typeof all[number]> = {};
    for (const f of all) {
      if (!byKind[f.kind]) byKind[f.kind] = f;
      if (Object.keys(byKind).length >= n) break;
    }
    const chosen = Object.values(byKind);
    return chosen.length >= n ? chosen : all.slice(0, n);
  }

  async function pickDest() {
    const p = await api.pickDestinationDir();
    if (p) destRoot = p;
  }

  function chooseTemplate(t: TemplateRow) {
    usingCustom = false;
    selectedTemplateId = t.id;
    customPattern = t.pattern;
  }

  function enterCustom() {
    usingCustom = true;
    if (!customPattern) customPattern = '{year}/{date}/{kind}';
  }

  async function start() {
    if (!destRoot) return;
    starting = true;
    const args: StartImportArgs = {
      card_label: card.label,
      card_mount: card.mount,
      camera_model: card.camera_model,
      dest_root: destRoot,
      pattern: activePattern,
      collision,
      worker_count: $settings?.worker_count ?? null,
    };
    try {
      const id = await api.startImport(args, report);
      currentView.set({ kind: 'progress', importId: id });
    } catch (e) {
      console.error(e);
      starting = false;
    }
  }
</script>

<section class="setup">
  <div class="row-head">
    <h3>Where it goes</h3>
  </div>

  <div class="field">
    <div class="label">Destination</div>
    <div class="dest-input">
      <input type="text" bind:value={destRoot} placeholder="Choose a folder…" spellcheck="false" />
      <Button variant="secondary" size="md" onclick={pickDest}>Choose…</Button>
    </div>
  </div>

  <div class="field">
    <div class="label">Folder template</div>
    <div class="template-list">
      {#each $templates as t}
        <button
          type="button"
          class="tile"
          class:selected={!usingCustom && selectedTemplateId === t.id}
          onclick={() => chooseTemplate(t)}
        >
          <div class="tile-name">{t.name}</div>
          <div class="tile-pattern">{t.pattern}</div>
        </button>
      {/each}
      <button
        type="button"
        class="tile"
        class:selected={usingCustom}
        onclick={enterCustom}
      >
        <div class="tile-name">Custom</div>
        <div class="tile-pattern dim">Edit pattern below</div>
      </button>
    </div>
    {#if usingCustom}
      <input
        type="text"
        class="custom-pattern"
        bind:value={customPattern}
        placeholder="{'{year}/{date}/{kind}'}"
        spellcheck="false"
      />
      <div class="hint">
        Variables: <code>{'{year}'}</code> <code>{'{month}'}</code> <code>{'{day}'}</code>
        <code>{'{date}'}</code> <code>{'{time}'}</code> <code>{'{camera}'}</code>
        <code>{'{lens}'}</code> <code>{'{kind}'}</code> <code>{'{ext}'}</code>
        <code>{'{orig_name}'}</code>
      </div>
    {/if}
  </div>

  <div class="field">
    <div class="label">Preview</div>
    <div class="preview-block">
      {#if previews.length === 0}
        <div class="dim small">No preview available</div>
      {:else}
        {#each previews as p}
          <div class="preview-line">{p}</div>
        {/each}
      {/if}
    </div>
  </div>

  <div class="field">
    <div class="label">If a file with the same name exists</div>
    <div class="seg">
      <button type="button" class:on={collision === 'rename'} onclick={() => (collision = 'rename')}>Rename</button>
      <button type="button" class:on={collision === 'skip'} onclick={() => (collision = 'skip')}>Skip</button>
      <button type="button" class:on={collision === 'overwrite'} onclick={() => (collision = 'overwrite')}>Overwrite</button>
    </div>
  </div>

  <div class="actions">
    <div class="summary">
      <span>{pluralize(report.files.length, 'file')}</span>
      <span class="sep">·</span>
      <span>{formatBytes(report.total_bytes)}</span>
    </div>
    <div class="action-buttons">
      <Button variant="ghost" onclick={onclose}>Cancel</Button>
      <Button variant="glass" size="lg" onclick={start} disabled={!destRoot} loading={starting}>
        Import to this folder
      </Button>
    </div>
  </div>
</section>

<style>
  .setup {
    background: var(--bg-card);
    border: 1px solid var(--divider);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card);
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .row-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 2px;
  }

  h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: -0.005em;
    color: var(--text-primary);
  }

  .field { display: flex; flex-direction: column; gap: 7px; }

  .label {
    font-size: 11px;
    color: var(--text-tertiary);
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .dest-input {
    display: flex;
    gap: 8px;
  }

  .dest-input input {
    flex: 1;
    font-size: 13px;
    height: 30px;
    padding: 0 10px;
    min-width: 0;
  }

  .template-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 7px;
  }

  .tile {
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    background: var(--bg-surface);
    text-align: left;
    transition:
      border-color var(--transition),
      background var(--transition),
      box-shadow var(--transition);
    cursor: default;
    min-width: 0;
  }
  .tile:hover { background: var(--bg-hover); }
  .tile.selected {
    border-color: var(--accent);
    background: var(--bg-selected);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .tile-name {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-primary);
  }
  .tile-pattern {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-secondary);
    margin-top: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .custom-pattern {
    font-family: var(--font-mono);
    font-size: 12px;
    margin-top: 6px;
    height: 30px;
    padding: 0 10px;
    width: 100%;
  }

  .preview-block {
    background: var(--bg-surface);
    border: 1px solid var(--divider);
    border-radius: var(--radius-md);
    padding: 9px 12px;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.75;
    color: var(--text-secondary);
    overflow-x: auto;
    max-height: 132px;
    overflow-y: auto;
  }

  .preview-line { white-space: nowrap; }

  .seg {
    display: inline-flex;
    background: var(--bg-surface);
    border: 1px solid var(--divider);
    border-radius: 7px;
    padding: 2px;
    width: fit-content;
  }
  .seg button {
    padding: 4px 14px;
    font-size: 12px;
    color: var(--text-secondary);
    border-radius: 5px;
    transition: background var(--transition), color var(--transition);
  }
  .seg button.on {
    background: var(--bg-card);
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  }

  .actions {
    margin-top: 6px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-top: 1px solid var(--divider);
    padding-top: 16px;
    gap: 14px;
    flex-wrap: wrap;
    row-gap: 10px;
  }

  .summary {
    color: var(--text-secondary);
    font-size: 12.5px;
    display: flex;
    gap: 6px;
    align-items: center;
    font-variant-numeric: tabular-nums;
  }
  .sep { opacity: 0.4; }

  .hint {
    font-size: 11px;
    color: var(--text-tertiary);
    margin-top: 6px;
    line-height: 1.8;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 6px;
    align-items: center;
  }
  .hint code {
    background: var(--bg-surface);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 10.5px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .small { font-size: 12px; }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
