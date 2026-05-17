<script lang="ts">
  import { api, type Settings } from '../api';
  import { settings, templates, toolStatus } from '../stores';
  import Button from './Button.svelte';

  let local = $state<Settings | null>(null);
  let savedIndicator = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if ($settings && !local) local = { ...$settings };
  });

  function commit() {
    if (!local) return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      if (!local) return;
      try {
        await api.setSettings(local);
        settings.set(local);
        savedIndicator = true;
        setTimeout(() => (savedIndicator = false), 1200);
      } catch (e) {
        console.error(e);
      }
    }, 250);
  }

  async function pickDest() {
    const p = await api.pickDestinationDir();
    if (p && local) {
      local.default_dest = p;
      commit();
    }
  }

  function setCollision(v: 'rename' | 'skip' | 'overwrite') {
    if (!local) return;
    local.collision_policy = v;
    commit();
  }

  function shortenPath(p: string): string {
    if (!p) return '';
    if (p.startsWith('/Users/')) {
      const segs = p.split('/');
      if (segs.length >= 3) return '~/' + segs.slice(3).join('/');
    }
    return p;
  }
</script>

<div class="settings">
  <header class="titlebar drag-region">
    <div class="title no-drag">Settings</div>
    {#if savedIndicator}
      <div class="saved-pill no-drag">Saved</div>
    {/if}
  </header>

  <div class="body">
    {#if local}
      <section>
        <h3>Import</h3>
        <div class="group">
          <div class="row">
            <div class="lbl">
              <div class="title">Default destination</div>
              <div class="sub mono">{shortenPath(local.default_dest)}</div>
            </div>
            <div class="ctrl">
              <Button variant="secondary" size="sm" onclick={pickDest}>Choose…</Button>
            </div>
          </div>

          <div class="row">
            <div class="lbl">
              <div class="title">If a file already exists</div>
              <div class="sub">When the destination has a file with the same name.</div>
            </div>
            <div class="ctrl">
              <div class="seg">
                <button class:on={local.collision_policy === 'rename'} onclick={() => setCollision('rename')}>Rename</button>
                <button class:on={local.collision_policy === 'skip'} onclick={() => setCollision('skip')}>Skip</button>
                <button class:on={local.collision_policy === 'overwrite'} onclick={() => setCollision('overwrite')}>Overwrite</button>
              </div>
            </div>
          </div>

          <div class="row">
            <div class="lbl">
              <div class="title">Verify each file after copy</div>
              <div class="sub">Hash-compare to catch corruption. Slower.</div>
            </div>
            <div class="ctrl">
              <label class="switch">
                <input type="checkbox" bind:checked={local.verify_hash} onchange={commit}/>
                <span class="track"></span>
              </label>
            </div>
          </div>

          <div class="row">
            <div class="lbl">
              <div class="title">Parallel transfers</div>
              <div class="sub">Workers reading the card simultaneously.</div>
            </div>
            <div class="ctrl">
              <input
                type="number"
                min="1"
                max="8"
                class="num"
                bind:value={local.worker_count}
                onchange={commit}
              />
            </div>
          </div>
        </div>
      </section>

      <section>
        <h3>Camera mode</h3>
        <div class="group">
          <div class="row">
            <div class="lbl">
              <div class="title">
                gphoto2
                {#if $toolStatus?.gphoto2_installed}
                  <span class="chip ok">Installed</span>
                {:else}
                  <span class="chip warn">Missing</span>
                {/if}
              </div>
              <div class="sub">
                {#if $toolStatus?.gphoto2_installed}
                  Cameras you plug in over USB will appear in the sidebar.
                {:else}
                  Run <span class="mono">brew install gphoto2</span> in Terminal, then reopen cardgrab.
                {/if}
              </div>
            </div>
            <div class="ctrl"></div>
          </div>
        </div>
      </section>

      <section>
        <h3>Templates</h3>
        <div class="group">
          {#each $templates as t}
            <div class="row">
              <div class="lbl">
                <div class="title">
                  {t.name}
                  {#if t.built_in}<span class="chip">Built-in</span>{/if}
                </div>
                <div class="sub mono">{t.pattern}</div>
              </div>
              <div class="ctrl"></div>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  </div>
</div>

<style>
  .settings {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Unified-titlebar feel: thin top strip with the active section name.
     Padding-left clears the macOS traffic lights. */
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
    color: var(--text-primary);
  }
  .saved-pill {
    font-size: 11px;
    color: var(--success);
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(48, 184, 80, 0.12);
    animation: fadein 220ms ease-out;
  }
  @keyframes fadein { from { opacity: 0; } to { opacity: 1; } }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 22px 28px 40px;
    max-width: 720px;
    width: 100%;
    align-self: center;
  }

  section + section { margin-top: 22px; }
  section { display: flex; flex-direction: column; gap: 8px; }

  h3 {
    margin: 0 0 0 2px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
  }

  .group {
    background: var(--bg-card);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 16px;
    align-items: center;
    padding: 11px 14px 11px 16px;
    min-height: 50px;
    border-bottom: 1px solid var(--divider);
  }
  .row:last-child { border-bottom: none; }

  .lbl { min-width: 0; }
  .title {
    font-size: 13px;
    font-weight: 400;
    color: var(--text-primary);
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .sub {
    font-size: 11.5px;
    color: var(--text-secondary);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .ctrl { display: flex; align-items: center; gap: 6px; }

  .seg {
    display: inline-flex;
    background: var(--bg-surface);
    border: 1px solid var(--divider);
    border-radius: 6px;
    padding: 1px;
  }
  .seg button {
    padding: 3px 10px;
    font-size: 11.5px;
    color: var(--text-secondary);
    border-radius: 4px;
    transition: background var(--transition), color var(--transition);
  }
  .seg button.on {
    background: var(--bg-card);
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(0,0,0,0.10);
  }

  .switch {
    position: relative;
    width: 32px;
    height: 18px;
    display: inline-block;
  }
  .switch input { opacity: 0; width: 0; height: 0; position: absolute; }
  .track {
    position: absolute; cursor: pointer; inset: 0;
    background: var(--bg-surface-2);
    border-radius: 999px;
    transition: background var(--transition);
  }
  .track::before {
    content: '';
    position: absolute;
    width: 14px; height: 14px;
    left: 2px; top: 2px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 1px 2px rgba(0,0,0,0.20);
    transition: transform var(--transition);
  }
  .switch input:checked + .track { background: var(--accent); }
  .switch input:checked + .track::before { transform: translateX(14px); }
  .switch input:disabled + .track { opacity: 0.45; cursor: not-allowed; }

  .num {
    width: 54px;
    padding: 4px 8px;
    font-size: 12px;
    text-align: center;
  }

  .chip {
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--bg-surface);
    color: var(--text-tertiary);
    font-weight: 500;
  }
  .chip.ok   { background: rgba(48, 184, 80, 0.14); color: var(--success); }
  .chip.warn { background: rgba(255, 159, 10, 0.14); color: var(--warning); }
</style>
