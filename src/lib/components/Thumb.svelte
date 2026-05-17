<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api, type MediaKind } from '../api';

  let {
    src,
    kind,
    name,
    selected = false,
    isVideo = false,
    onClick,
  } = $props<{
    src: string;
    kind: MediaKind;
    name: string;
    selected?: boolean;
    isVideo?: boolean;
    onClick?: (e: MouseEvent) => void;
  }>();

  let el: HTMLDivElement | null = $state(null);
  let url: string | null = $state(null);
  let loaded = $state(false);
  let failed = $state(false);
  let inView = $state(false);
  let observer: IntersectionObserver | null = null;
  let loading = false;

  // Camera-PTP files cannot be previewed (no local file yet)
  let previewable = $derived(!src.startsWith('camera://'));

  onMount(() => {
    if (!previewable || !el) return;
    observer = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) {
            inView = true;
            observer?.disconnect();
            break;
          }
        }
      },
      { rootMargin: '300px' }
    );
    observer.observe(el);
  });

  onDestroy(() => observer?.disconnect());

  $effect(() => {
    if (inView && !url && !loading && !failed) {
      loading = true;
      api.getThumbnail(src)
        .then((u) => {
          url = u;
        })
        .catch(() => {
          failed = true;
        })
        .finally(() => {
          loading = false;
        });
    }
  });

  function handleImgLoad() {
    loaded = true;
  }
  function handleImgError() {
    failed = true;
  }

  function handleClick(e: MouseEvent) {
    onClick?.(e);
  }
</script>

<div
  bind:this={el}
  class="thumb"
  class:selected
  class:has-image={loaded}
  role="button"
  tabindex="0"
  aria-pressed={selected}
  onclick={handleClick}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onClick?.(e as unknown as MouseEvent);
    }
  }}
  title={name}
>
  {#if url && !failed}
    <img src={url} alt={name} onload={handleImgLoad} onerror={handleImgError} />
  {/if}

  {#if !loaded || failed || !previewable}
    <div class="placeholder kind-{kind}">
      {#if kind === 'photo'}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="5" width="18" height="14" rx="2"/>
          <circle cx="12" cy="12" r="3.5"/>
        </svg>
      {:else if kind === 'raw'}
        <span class="badge-text">RAW</span>
      {:else if kind === 'video'}
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z"/>
        </svg>
      {:else if kind === 'audio'}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 18V6l10-2v12"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="16" cy="16" r="3"/>
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
          <path d="M14 3v6h6"/>
        </svg>
      {/if}
    </div>
  {/if}

  {#if isVideo && loaded}
    <div class="play-overlay">
      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
    </div>
  {/if}

  {#if kind === 'raw' && loaded}
    <div class="kind-tag">RAW</div>
  {/if}

  <div class="check">
    <svg viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="7" class="check-bg"/>
      <path d="m5 8 2 2 4-5" stroke="white" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" class="check-tick"/>
    </svg>
  </div>
</div>

<style>
  .thumb {
    position: relative;
    aspect-ratio: 1 / 1;
    background: var(--bg-surface-2);
    border-radius: 7px;
    overflow: hidden;
    cursor: default;
    transition:
      transform 160ms var(--ease-out),
      box-shadow 160ms var(--ease-out);
    user-select: none;
    outline: none;
    will-change: transform;
  }
  .thumb:hover {
    transform: scale(1.02);
  }
  .thumb:active {
    transform: scale(0.985);
    transition-duration: 80ms;
  }
  .thumb:focus-visible {
    box-shadow: 0 0 0 2px var(--accent);
  }
  .thumb.selected {
    box-shadow: 0 0 0 2px var(--accent);
  }

  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 260ms var(--ease-out);
  }
  .thumb.has-image img {
    opacity: 1;
  }

  .placeholder {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
  }
  .placeholder svg {
    width: 28%;
    height: 28%;
    max-width: 32px;
    max-height: 32px;
  }
  .placeholder.kind-raw .badge-text {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-tertiary);
  }
  .placeholder.kind-video { color: var(--text-secondary); }

  .play-overlay {
    position: absolute;
    right: 6px;
    bottom: 6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    padding-left: 1px;
  }
  .play-overlay svg { width: 9px; height: 9px; }

  .kind-tag {
    position: absolute;
    top: 6px;
    left: 6px;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: white;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(8px);
    padding: 2px 5px;
    border-radius: 3px;
  }

  .check {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 18px;
    height: 18px;
    opacity: 0;
    transform: scale(0.7);
    transition:
      opacity 140ms var(--ease-out),
      transform 220ms cubic-bezier(0.34, 1.56, 0.64, 1);
    pointer-events: none;
  }
  .thumb:hover .check,
  .thumb.selected .check {
    opacity: 1;
    transform: scale(1);
  }
  .check-bg {
    fill: rgba(255, 255, 255, 0.85);
    stroke: rgba(0, 0, 0, 0.18);
    stroke-width: 0.5;
  }
  .check-tick {
    opacity: 0;
    stroke: #1a73ff;
  }
  .thumb.selected .check-bg {
    fill: var(--accent);
    stroke: var(--accent);
  }
  .thumb.selected .check-tick {
    opacity: 1;
    stroke: white;
  }
</style>
