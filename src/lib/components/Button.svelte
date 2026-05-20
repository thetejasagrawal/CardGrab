<script lang="ts">
  type Variant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'glass';
  type Size = 'sm' | 'md' | 'lg';

  let {
    variant = 'secondary' as Variant,
    size = 'md' as Size,
    disabled = false,
    loading = false,
    onclick,
    type = 'button' as 'button' | 'submit',
    children,
    ...rest
  } = $props<{
    variant?: Variant;
    size?: Size;
    disabled?: boolean;
    loading?: boolean;
    onclick?: (e: MouseEvent) => void;
    type?: 'button' | 'submit';
    children: any;
  }>();
</script>

<button
  {type}
  class="btn no-drag {variant} {size}"
  class:loading
  disabled={disabled || loading}
  onclick={onclick}
  {...rest}
>
  <span class="content">{@render children()}</span>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
  {/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    font-weight: 560;
    letter-spacing: 0;
    border-radius: 7px;
    transition:
      background 140ms var(--ease-snap),
      border-color 140ms var(--ease-snap),
      color 140ms var(--ease-snap),
      box-shadow 140ms var(--ease-snap),
      transform 180ms var(--ease-spring),
      opacity 140ms var(--ease-snap);
    white-space: nowrap;
    border: 1px solid transparent;
    position: relative;
    isolation: isolate;
    -webkit-font-smoothing: antialiased;
  }
  .btn:disabled { opacity: 0.48; cursor: not-allowed; }
  .btn:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 3px rgba(10, 132, 255, 0.24),
      var(--button-shadow, none);
  }
  .btn:active:not(:disabled) {
    transform: scale(0.97);
    transition-duration: 80ms;
  }

  .btn.sm { padding: 3px 10px; font-size: 12px; min-height: 24px; }
  .btn.md { padding: 5px 14px; font-size: 13px; min-height: 30px; }
  .btn.lg { padding: 8px 18px; font-size: 13.5px; min-height: 38px; }

  .btn.primary {
    --button-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.30),
      inset 0 -1px 0 rgba(0, 0, 0, 0.18),
      0 1px 1px rgba(0, 0, 0, 0.10),
      0 8px 20px -14px rgba(10, 132, 255, 0.80);
    background: linear-gradient(180deg, color-mix(in srgb, var(--accent) 84%, white), var(--accent));
    color: var(--text-on-accent);
    border-color: color-mix(in srgb, var(--accent) 76%, black);
    box-shadow: var(--button-shadow);
  }
  .btn.primary:hover:not(:disabled) {
    background: linear-gradient(180deg, color-mix(in srgb, var(--accent-hover) 84%, white), var(--accent-hover));
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.34),
      inset 0 -1px 0 rgba(0, 0, 0, 0.18),
      0 1px 1px rgba(0, 0, 0, 0.10),
      0 10px 22px -12px rgba(10, 132, 255, 0.80);
  }
  .btn.primary:active:not(:disabled) {
    background: var(--accent-pressed);
    box-shadow:
      inset 0 2px 3px rgba(0, 0, 0, 0.28),
      0 0 0 0.5px rgba(0, 0, 0, 0.15);
  }

  .btn.secondary {
    --button-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.88),
      inset 0 -1px 0 rgba(0, 0, 0, 0.05),
      0 0.5px 0 rgba(0, 0, 0, 0.05),
      0 1.5px 2px rgba(0, 0, 0, 0.04);
    background: linear-gradient(180deg, #fefefe, #f0f0f2);
    border-color: var(--border-input);
    color: var(--text-primary);
    box-shadow: var(--button-shadow);
  }
  @media (prefers-color-scheme: dark) {
    .btn.secondary {
      --button-shadow:
        inset 0 1px 0 rgba(255, 255, 255, 0.10),
        inset 0 -1px 0 rgba(0, 0, 0, 0.30),
        0 0.5px 0 rgba(0, 0, 0, 0.30),
        0 1.5px 2px rgba(0, 0, 0, 0.32);
      background: linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 86%, white), var(--bg-card));
    }
  }
  .btn.secondary:hover:not(:disabled) {
    border-color: var(--divider-strong);
    background: linear-gradient(180deg, #ffffff, #f3f3f5);
  }
  @media (prefers-color-scheme: dark) {
    .btn.secondary:hover:not(:disabled) {
      background: linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 78%, white), color-mix(in srgb, var(--bg-card) 96%, white));
    }
  }
  .btn.secondary:active:not(:disabled) {
    background: var(--bg-active);
    box-shadow:
      inset 0 1px 2px rgba(0, 0, 0, 0.10),
      0 0.5px 0 rgba(0, 0, 0, 0.05);
  }

  .btn.ghost {
    background: transparent;
    color: var(--text-primary);
    border-color: transparent;
    box-shadow: none;
  }
  .btn.ghost:hover:not(:disabled)  {
    background: var(--bg-hover);
    border-color: transparent;
  }
  .btn.ghost:active:not(:disabled) { background: var(--bg-active); }

  .btn.danger {
    background: linear-gradient(180deg, color-mix(in srgb, var(--danger) 84%, white), var(--danger));
    border-color: color-mix(in srgb, var(--danger) 80%, black);
    color: white;
    box-shadow:
      inset 0 1px 0 rgba(255,255,255,0.20),
      inset 0 -1px 0 rgba(0,0,0,0.14),
      0 1px 1px rgba(0,0,0,0.10);
  }
  .btn.danger:hover:not(:disabled) {
    filter: brightness(1.03);
  }

  .btn.glass {
    background: color-mix(in srgb, var(--bg-card) 72%, transparent);
    border-color: color-mix(in srgb, var(--divider-strong) 70%, transparent);
    color: var(--text-primary);
    box-shadow:
      inset 0 1px 0 rgba(255,255,255,0.38),
      0 1px 2px rgba(0,0,0,0.08);
    backdrop-filter: blur(16px) saturate(170%);
    -webkit-backdrop-filter: blur(16px) saturate(170%);
  }
  .btn.glass:hover:not(:disabled) { background: color-mix(in srgb, var(--bg-card) 86%, transparent); }
  .btn.glass:active:not(:disabled) {
    background: var(--bg-active);
  }

  .content { display: inline-flex; align-items: center; gap: 6px; }

  .spinner {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    animation: spin 0.6s linear infinite;
    margin-left: 4px;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
