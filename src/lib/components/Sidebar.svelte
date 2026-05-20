<script lang="ts">
  import { cards, currentView, toolStatus, type View } from '../stores';
  import Icon from './Icon.svelte';

  function select(v: View) {
    currentView.set(v);
  }

  let sources = $derived([...$cards].sort((a, b) => {
    const order = { sd: 0, camera: 1 };
    return order[a.kind] - order[b.kind] || a.label.localeCompare(b.label);
  }));

  let isHistory = $derived(
    $currentView.kind === 'history' || $currentView.kind === 'import-detail'
  );
  let isSettings = $derived($currentView.kind === 'settings');

  function isCardSelected(mount: string) {
    return $currentView.kind === 'card' && $currentView.mount === mount;
  }

  function sourceKindLabel(kind: string) {
    return kind === 'camera' ? 'Camera' : 'Card';
  }
</script>

<aside class="sidebar">
  <div class="titlebar-region" data-tauri-drag-region></div>

  <nav class="nav no-drag" aria-label="Main">
    <section class="source-section">
      <div class="section-head">
        <h2>Cards</h2>
        {#if sources.length > 0}
          <span class="section-count">{sources.length}</span>
        {/if}
      </div>

      {#if sources.length === 0}
        <div class="source-empty">
          <div class="empty-icon"><Icon name="sd-card" size={18} stroke={1.6} /></div>
          <div>
            <div class="empty-title">No card connected</div>
            <div class="empty-sub">Insert an SD card or plug in a camera.</div>
          </div>
        </div>

        {#if $toolStatus && !$toolStatus.gphoto2_installed}
          <button class="setup-card" onclick={() => select({ kind: 'settings' })}>
            <span class="setup-icon"><Icon name="camera" size={15} /></span>
            <span>
              <span class="setup-title">Enable camera mode</span>
              <span class="setup-sub">Install gphoto2 for USB import</span>
            </span>
          </button>
        {/if}
      {:else}
        <div class="source-list">
        {#each sources as c (c.mount)}
          <button
            class="source-row"
            class:selected={isCardSelected(c.mount)}
            aria-current={isCardSelected(c.mount) ? 'page' : undefined}
            onclick={() => select({ kind: 'card', mount: c.mount })}
          >
            <span class="source-ico">
              <Icon name={c.kind === 'camera' ? 'camera' : 'sd-card'} size={16} />
            </span>
            <span class="source-copy">
              <span class="source-name">{c.label}</span>
              <span class="source-meta">
                {sourceKindLabel(c.kind)}
                {#if c.camera_model && c.camera_model !== c.label}
                  · {c.camera_model}
                {/if}
              </span>
            </span>
          </button>
        {/each}
        </div>
      {/if}
    </section>
  </nav>

  <div class="bottom no-drag" aria-label="Library and settings">
    <button
      class="row"
      class:selected={isHistory}
      aria-current={isHistory ? 'page' : undefined}
      onclick={() => select({ kind: 'history' })}
    >
      <span class="ico"><Icon name="clock" size={15} /></span>
      <span class="label">Import history</span>
    </button>
    <button
      class="row"
      class:selected={isSettings}
      aria-current={isSettings ? 'page' : undefined}
      onclick={() => select({ kind: 'settings' })}
    >
      <span class="ico"><Icon name="gear" size={15} /></span>
      <span class="label">Settings</span>
    </button>
    <div class="wordmark">cardgrab</div>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-w);
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 0 10px 10px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--divider);
    backdrop-filter: saturate(180%) blur(30px);
    -webkit-backdrop-filter: saturate(180%) blur(30px);
    overflow: hidden;
  }

  /* Reserve the macOS traffic-light strip. Match the main-view titlebar height
     so the first sidebar section aligns with the main-view title baseline. */
  .titlebar-region { height: var(--titlebar-h); flex-shrink: 0; }

  .nav {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    overflow-y: auto;
    padding-top: 4px;
  }

  section { display: flex; flex-direction: column; gap: 6px; }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
  }

  h2 {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
    margin: 0;
  }

  .section-count {
    font-size: 10.5px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .source-empty {
    display: flex;
    gap: 9px;
    align-items: flex-start;
    margin: 0 2px;
    padding: 10px 8px;
    border-radius: 8px;
    color: var(--text-secondary);
    background: rgba(255,255,255,0.25);
    border: 1px solid var(--divider);
  }
  @media (prefers-color-scheme: dark) {
    .source-empty { background: rgba(255,255,255,0.035); }
  }
  .empty-icon {
    color: var(--text-tertiary);
    margin-top: 1px;
    flex-shrink: 0;
  }
  .empty-title {
    color: var(--text-primary);
    font-size: 12.5px;
    font-weight: 550;
    line-height: 1.25;
  }
  .empty-sub {
    margin-top: 2px;
    color: var(--text-tertiary);
    font-size: 11.5px;
    line-height: 1.35;
  }

  .setup-card {
    display: grid;
    grid-template-columns: 18px 1fr;
    gap: 8px;
    text-align: left;
    align-items: start;
    margin: 0 2px;
    padding: 8px;
    border-radius: 8px;
    color: var(--text-primary);
    background: rgba(255, 159, 10, 0.10);
    border: 1px solid rgba(255, 159, 10, 0.18);
    transition: background var(--transition), border-color var(--transition);
  }
  .setup-card:hover {
    background: rgba(255, 159, 10, 0.15);
    border-color: rgba(255, 159, 10, 0.28);
  }
  .setup-icon { color: var(--warning); margin-top: 1px; }
  .setup-title {
    display: block;
    font-size: 12px;
    font-weight: 550;
    line-height: 1.25;
  }
  .setup-sub {
    display: block;
    margin-top: 1px;
    font-size: 11px;
    line-height: 1.3;
    color: var(--text-secondary);
  }

  .source-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .source-row {
    width: 100%;
    display: grid;
    grid-template-columns: 24px 1fr;
    align-items: center;
    gap: 8px;
    min-height: 42px;
    padding: 6px 8px;
    border-radius: 8px;
    text-align: left;
    color: var(--text-primary);
    transition:
      background 160ms var(--ease-snap),
      color 160ms var(--ease-snap),
      box-shadow 160ms var(--ease-snap),
      transform 200ms var(--ease-spring);
  }
  .source-row:hover { background: var(--bg-hover); }
  .source-row:active { transform: scale(0.985); transition-duration: 90ms; }
  .source-row.selected {
    background: linear-gradient(180deg,
      color-mix(in srgb, var(--accent) 88%, white),
      var(--accent));
    color: var(--text-on-accent);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.22),
      inset 0 -1px 0 rgba(0, 0, 0, 0.14),
      0 1px 1px rgba(0, 0, 0, 0.08);
  }
  .source-ico {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: grid;
    place-items: center;
    color: var(--accent);
    background: var(--bg-selected);
    transition: background 160ms var(--ease-snap), color 160ms var(--ease-snap);
  }
  .source-row.selected .source-ico {
    background: rgba(255,255,255,0.20);
    color: var(--text-on-accent);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.12);
  }
  .source-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .source-name {
    font-size: 13px;
    line-height: 1.2;
    font-weight: 550;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .source-meta {
    font-size: 11px;
    line-height: 1.25;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .source-row.selected .source-meta {
    color: rgba(255,255,255,0.74);
  }

  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 5px 8px;
    min-height: 30px;
    border-radius: 7px;
    text-align: left;
    color: var(--text-primary);
    transition:
      background 160ms var(--ease-snap),
      color 160ms var(--ease-snap),
      box-shadow 160ms var(--ease-snap),
      transform 200ms var(--ease-spring);
  }
  .row:hover { background: var(--bg-hover); }
  .row:active { transform: scale(0.985); transition-duration: 90ms; }
  .row.selected {
    background: linear-gradient(180deg,
      color-mix(in srgb, var(--accent) 88%, white),
      var(--accent));
    color: var(--text-on-accent);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.22),
      inset 0 -1px 0 rgba(0, 0, 0, 0.14),
      0 1px 1px rgba(0, 0, 0, 0.08);
  }
  .row.selected .ico { color: var(--text-on-accent); }

  .ico {
    color: var(--text-secondary);
    flex-shrink: 0;
    display: grid;
    place-items: center;
    width: 15px;
    height: 15px;
  }

  .label {
    font-size: 13px;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .bottom {
    flex-shrink: 0;
    padding-top: 8px;
    border-top: 1px solid var(--divider);
    margin-top: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .wordmark {
    margin-top: 6px;
    padding: 4px 8px 0;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.005em;
    color: var(--text-tertiary);
    opacity: 0.7;
    user-select: none;
  }
</style>
