<script lang="ts">
  import { cards, currentView, toolStatus, type View } from '../stores';

  function select(v: View) {
    currentView.set(v);
  }

  let sdCards = $derived($cards.filter((c) => c.kind === 'sd'));
  let cameras = $derived($cards.filter((c) => c.kind === 'camera'));

  let isHistory = $derived(
    $currentView.kind === 'history' || $currentView.kind === 'import-detail'
  );
  let isSettings = $derived($currentView.kind === 'settings');

  function isCardSelected(mount: string) {
    return $currentView.kind === 'card' && $currentView.mount === mount;
  }
</script>

<aside class="sidebar drag-region">
  <div class="titlebar-region"></div>

  <nav class="nav no-drag">
    <section>
      <h2>Cards</h2>
      {#if sdCards.length === 0}
        <p class="empty">None connected</p>
      {:else}
        {#each sdCards as c (c.mount)}
          <button
            class="row"
            class:selected={isCardSelected(c.mount)}
            onclick={() => select({ kind: 'card', mount: c.mount })}
          >
            <svg class="ico" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M7 3h7l5 5v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"
                stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
              <path d="M10.5 6v2.5M12.5 6v2.5M14.5 6v2.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <span class="label">{c.label}</span>
          </button>
        {/each}
      {/if}
    </section>

    <section>
      <h2>Cameras</h2>
      {#if $toolStatus && !$toolStatus.gphoto2_installed}
        <button class="row subtle" onclick={() => select({ kind: 'settings' })}>
          <svg class="ico" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M9 4h6l1.6 2.4H21a1 1 0 0 1 1 1V19a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V7.4a1 1 0 0 1 1-1h4.4L9 4Z"
              stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
            <circle cx="12" cy="13" r="4" stroke="currentColor" stroke-width="1.5"/>
          </svg>
          <span class="label">Set up camera mode</span>
        </button>
      {:else if cameras.length === 0}
        <p class="empty">None connected</p>
      {:else}
        {#each cameras as c (c.mount)}
          <button
            class="row"
            class:selected={isCardSelected(c.mount)}
            onclick={() => select({ kind: 'card', mount: c.mount })}
          >
            <svg class="ico" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M9 4h6l1.6 2.4H21a1 1 0 0 1 1 1V19a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V7.4a1 1 0 0 1 1-1h4.4L9 4Z"
                stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
              <circle cx="12" cy="13" r="4" stroke="currentColor" stroke-width="1.5"/>
            </svg>
            <span class="label">{c.label}</span>
          </button>
        {/each}
      {/if}
    </section>

    <section>
      <h2>Library</h2>
      <button
        class="row"
        class:selected={isHistory}
        onclick={() => select({ kind: 'history' })}
      >
        <svg class="ico" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
          <path d="M12 7v5l3 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <span class="label">History</span>
      </button>
    </section>
  </nav>

  <div class="bottom no-drag">
    <button
      class="row"
      class:selected={isSettings}
      onclick={() => select({ kind: 'settings' })}
    >
      <svg class="ico" width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="12" cy="12" r="2.5" stroke="currentColor" stroke-width="1.5"/>
        <path d="M19.4 15a1.7 1.7 0 0 0 .34 1.86l.06.07a2 2 0 1 1-2.83 2.83l-.07-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1.04 1.55V21a2 2 0 0 1-4 0v-.09a1.7 1.7 0 0 0-1.1-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.07a1.7 1.7 0 0 0 .34-1.87 1.7 1.7 0 0 0-1.55-1.04H3a2 2 0 0 1 0-4h.09A1.7 1.7 0 0 0 4.64 8.6a1.7 1.7 0 0 0-.34-1.87l-.06-.07a2 2 0 1 1 2.83-2.83l.07.06a1.7 1.7 0 0 0 1.86.34H9a1.7 1.7 0 0 0 1.04-1.55V3a2 2 0 0 1 4 0v.09a1.7 1.7 0 0 0 1.04 1.55 1.7 1.7 0 0 0 1.86-.34l.07-.06a2 2 0 1 1 2.83 2.83l-.06.07a1.7 1.7 0 0 0-.34 1.86V9a1.7 1.7 0 0 0 1.55 1.04H21a2 2 0 0 1 0 4h-.09a1.7 1.7 0 0 0-1.55 1.04Z"
          stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <span class="label">Settings</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-w);
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 0 8px 8px;
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
    gap: 14px;
    flex: 1;
    overflow-y: auto;
  }

  section { display: flex; flex-direction: column; gap: 1px; }

  h2 {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
    margin: 0 0 4px;
    padding: 0 10px;
  }

  .empty {
    margin: 0;
    padding: 2px 10px 4px;
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 4px 10px;
    height: 26px;
    border-radius: 5px;
    text-align: left;
    color: var(--text-primary);
    transition: background var(--transition);
  }
  .row:hover { background: var(--bg-hover); }
  .row.selected {
    background: var(--accent);
    color: var(--text-on-accent);
  }
  .row.selected .ico { color: var(--text-on-accent); }
  .row.subtle .ico { color: var(--warning); }

  .ico {
    color: var(--text-secondary);
    flex-shrink: 0;
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
    padding-top: 6px;
    border-top: 1px solid var(--divider);
    margin-top: 4px;
  }
</style>
