<script lang="ts">
  import { toolStatus } from '../stores';
</script>

<div class="empty drag-region">
  <div class="stack no-drag">
    <div class="symbol" aria-hidden="true">
      <svg width="56" height="56" viewBox="0 0 24 24" fill="none">
        <path d="M7 3h7l5 5v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"
          stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
        <path d="M10.5 6v2.5M12.5 6v2.5M14.5 6v2.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      </svg>
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
  }

  .symbol {
    color: var(--text-tertiary);
    margin-bottom: 14px;
    opacity: 0.65;
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
