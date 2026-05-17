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
    gap: 6px;
    font-weight: 500;
    letter-spacing: -0.005em;
    border-radius: var(--radius-sm);
    transition:
      background var(--transition),
      color var(--transition),
      box-shadow var(--transition),
      transform var(--transition),
      opacity var(--transition);
    white-space: nowrap;
    border: 1px solid transparent;
    position: relative;
  }
  .btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .btn.sm { padding: 3px 10px; font-size: 12px; min-height: 22px; }
  .btn.md { padding: 5px 14px; font-size: 13px; min-height: 28px; }
  .btn.lg { padding: 8px 18px; font-size: 14px; min-height: 36px; }

  .btn.primary {
    background: var(--accent);
    color: var(--text-on-accent);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.18),
      0 1px 0 rgba(0, 0, 0, 0.05);
  }
  .btn.primary:hover:not(:disabled) { background: var(--accent-hover); }
  .btn.primary:active:not(:disabled) { background: var(--accent-pressed); }

  .btn.secondary {
    background: var(--bg-card);
    border-color: var(--border-input);
    color: var(--text-primary);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.04);
  }
  .btn.secondary:hover:not(:disabled) { background: var(--bg-hover); }
  .btn.secondary:active:not(:disabled) { background: var(--bg-active); }

  .btn.ghost { background: transparent; color: var(--text-primary); }
  .btn.ghost:hover:not(:disabled)  { background: var(--bg-hover); }
  .btn.ghost:active:not(:disabled) { background: var(--bg-active); }

  .btn.danger { background: var(--danger); color: white; }

  /* ────────────────────────────────────────────────────────────────────
   * "Liquid Glass" — pill, radial top-center highlight, soft blue halo.
   * Used for the hero CTA only. Looks identical in light + dark mode.
   * ──────────────────────────────────────────────────────────────────── */
  .btn.glass {
    border-radius: 999px;
    color: #0b1538;
    border: 0.5px solid rgba(255, 255, 255, 0.85);
    background:
      radial-gradient(
        140% 200% at 50% -40%,
        rgba(96, 160, 255, 0.85) 0%,
        rgba(170, 200, 255, 0.75) 22%,
        rgba(232, 240, 255, 0.95) 55%,
        rgba(252, 253, 255, 1) 85%
      ),
      #ffffff;
    box-shadow:
      inset 0 1.2px 1.2px rgba(255, 255, 255, 0.95),
      inset 0 -1px 1px rgba(120, 160, 230, 0.32),
      inset 0 0 0 0.5px rgba(255, 255, 255, 0.45),
      0 0.5px 1px rgba(0, 0, 0, 0.06),
      0 12px 28px -10px rgba(40, 100, 220, 0.48),
      0 4px 10px -6px rgba(0, 0, 0, 0.22);
    backdrop-filter: blur(14px) saturate(160%);
    -webkit-backdrop-filter: blur(14px) saturate(160%);
    padding: 9px 22px;
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: -0.005em;
    min-height: 34px;
  }
  .btn.glass.sm { padding: 5px 16px; font-size: 12.5px; min-height: 26px; }
  .btn.glass.md { padding: 7px 20px; font-size: 13px;   min-height: 30px; }
  .btn.glass.lg { padding: 11px 26px; font-size: 14px;  min-height: 40px; }

  .btn.glass:hover:not(:disabled) {
    background:
      radial-gradient(
        140% 200% at 50% -40%,
        rgba(120, 175, 255, 0.95) 0%,
        rgba(185, 210, 255, 0.85) 22%,
        rgba(238, 244, 255, 1) 55%,
        rgba(255, 255, 255, 1) 85%
      ),
      #ffffff;
    box-shadow:
      inset 0 1.2px 1.2px rgba(255, 255, 255, 1),
      inset 0 -1px 1px rgba(120, 160, 230, 0.38),
      inset 0 0 0 0.5px rgba(255, 255, 255, 0.55),
      0 0.5px 1px rgba(0, 0, 0, 0.07),
      0 16px 36px -10px rgba(40, 100, 220, 0.55),
      0 6px 14px -8px rgba(0, 0, 0, 0.28);
  }
  .btn.glass:active:not(:disabled) {
    transform: translateY(0.5px) scale(0.997);
    box-shadow:
      inset 0 1px 1px rgba(255, 255, 255, 0.85),
      inset 0 -1px 1px rgba(120, 160, 230, 0.28),
      0 0.5px 1px rgba(0, 0, 0, 0.05),
      0 6px 16px -8px rgba(40, 100, 220, 0.38);
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
