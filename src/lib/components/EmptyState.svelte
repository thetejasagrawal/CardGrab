<script lang="ts">
  import { toolStatus } from '../stores';
  import Icon from './Icon.svelte';
</script>

<div class="empty">
  <div class="stack">
    <div class="symbol" aria-hidden="true">
      <Icon name="sd-card" size={54} stroke={1.35} />
    </div>
    <h1>No card connected</h1>
    <p>Plug in an SD card or a camera. cardgrab will pick it up automatically.</p>
    {#if $toolStatus && !$toolStatus.gphoto2_installed}
      <p class="tip">
        For camera mode: <span class="mono">brew install gphoto2</span>
      </p>
    {/if}
  </div>
</div>

<style>
  .empty {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 32px;
  }

  .stack {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 360px;
    animation: empty-in 420ms var(--ease-out) both;
  }
  @keyframes empty-in {
    from { opacity: 0; transform: translateY(6px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .symbol {
    color: var(--text-tertiary);
    margin-bottom: 14px;
    opacity: 0.65;
    animation: float 4s ease-in-out infinite;
  }
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50%      { transform: translateY(-3px); }
  }

  h1 {
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.005em;
    margin: 0 0 4px;
    color: var(--text-primary);
  }

  p {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-secondary);
  }

  .tip {
    margin-top: 16px;
    font-size: 11.5px;
    color: var(--text-tertiary);
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-surface);
    padding: 1px 5px;
    border-radius: 4px;
    color: var(--text-secondary);
  }
</style>
