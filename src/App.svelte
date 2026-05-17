<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import {
    api,
    onCardAttached,
    onCardDetached,
    onImportProgress,
    onImportComplete,
    type Card,
  } from './lib/api';

  // Explicit window-drag handler. WKWebView (which Tauri uses on macOS) does NOT
  // support -webkit-app-region: drag (Chromium-only extension), so JS is the
  // only path. Attached in capture phase so no child can swallow the mousedown.
  function handleWindowDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (!target) return;
    if (target.closest('button, input, textarea, select, a, [contenteditable="true"]')) return;
    if (!target.closest('[data-tauri-drag-region]')) return;
    getCurrentWindow().startDragging().catch((err) => console.error('startDragging:', err));
  }
  import {
    cards,
    currentView,
    activeProgress,
    lastCompleted,
    settings,
    templates,
    toolStatus,
  } from './lib/stores';
  import Sidebar from './lib/components/Sidebar.svelte';
  import EmptyState from './lib/components/EmptyState.svelte';
  import CardPanel from './lib/components/CardPanel.svelte';
  import ProgressView from './lib/components/ProgressView.svelte';
  import HistoryView from './lib/components/HistoryView.svelte';
  import ImportDetail from './lib/components/ImportDetail.svelte';
  import SettingsView from './lib/components/SettingsView.svelte';

  onMount(() => {
    // Attach drag handler at document capture phase — guarantees we run before
    // any child stopPropagation can swallow the mousedown.
    document.addEventListener('mousedown', handleWindowDrag, true);

    // Initial loads
    Promise.all([
      api.listCards().then((cs) => cards.set(cs)),
      api.getSettings().then((s) => settings.set(s)).catch(() => {}),
      api.listTemplates().then((t) => templates.set(t)).catch(() => {}),
      api.toolStatus().then((t) => toolStatus.set(t)).catch(() => {}),
    ]);

    // Subscribe to events
    const unsubs: Array<Promise<() => void>> = [];
    unsubs.push(onCardAttached((card) => {
      cards.update((cs) => {
        if (cs.find((c) => c.mount === card.mount)) return cs;
        return [...cs, card];
      });
      // Auto-focus the new card if user is on the empty screen
      currentView.update((v) => v.kind === 'empty' ? { kind: 'card', mount: card.mount } : v);
    }));
    unsubs.push(onCardDetached((card) => {
      cards.update((cs) => cs.filter((c) => c.mount !== card.mount));
      currentView.update((v) =>
        v.kind === 'card' && v.mount === card.mount ? { kind: 'empty' } : v
      );
    }));
    unsubs.push(onImportProgress((p) => activeProgress.set(p)));
    unsubs.push(onImportComplete((c) => {
      lastCompleted.set({
        importId: c.import_id,
        destRoot: '',
        status: c.status,
        fileCount: c.file_count,
        bytesTotal: c.bytes_total,
        failures: c.failures,
      });
    }));

    return () => {
      document.removeEventListener('mousedown', handleWindowDrag, true);
      unsubs.forEach((p) => p.then((u) => u()).catch(() => {}));
    };
  });

  // Auto-select first card whenever we go from 0 → ≥1 cards while on empty view
  $effect(() => {
    if ($currentView.kind === 'empty' && $cards.length > 0) {
      currentView.set({ kind: 'card', mount: $cards[0].mount });
    }
  });

  let cardForView = $derived.by(() => {
    if ($currentView.kind !== 'card') return null;
    return $cards.find((c) => c.mount === $currentView.mount) ?? null;
  });
</script>

<div class="root">
  <Sidebar />

  <main class="content">
    <!-- Traffic-light spacer fades to transparent at the top -->
    <div class="content-inner">
      {#if $currentView.kind === 'empty'}
        <EmptyState />
      {:else if $currentView.kind === 'card'}
        {#if cardForView}
          {#key $currentView.mount}
            <CardPanel card={cardForView!} />
          {/key}
        {:else}
          <EmptyState />
        {/if}
      {:else if $currentView.kind === 'progress'}
        <ProgressView importId={$currentView.importId} />
      {:else if $currentView.kind === 'history'}
        <HistoryView />
      {:else if $currentView.kind === 'import-detail'}
        <ImportDetail importId={$currentView.importId} />
      {:else if $currentView.kind === 'settings'}
        <SettingsView />
      {/if}
    </div>
  </main>
</div>

<style>
  .root {
    display: flex;
    height: 100vh;
    width: 100vw;
    background: var(--bg-window);
    color: var(--text-primary);
    backdrop-filter: saturate(180%);
    position: relative;
  }


  .content {
    flex: 1;
    min-width: 0;
    height: 100vh;
    background: var(--bg-content);
    overflow: hidden;
    position: relative;
  }

  .content-inner {
    height: 100%;
    overflow: hidden;
  }
</style>
