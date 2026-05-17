<script lang="ts">
  let {
    label,
    value,
    accent = 'neutral' as 'neutral' | 'blue' | 'green' | 'orange' | 'purple' | 'red',
  } = $props<{
    label: string;
    value: string | number;
    accent?: 'neutral' | 'blue' | 'green' | 'orange' | 'purple' | 'red';
  }>();
</script>

<div class="chip {accent}">
  <div class="value tabular">{value}</div>
  <div class="label">{label}</div>
</div>

<style>
  .chip {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--divider);
    min-width: 86px;
  }

  .value {
    font-size: 18px;
    font-weight: 600;
    letter-spacing: -0.015em;
    color: var(--text-primary);
  }

  .label {
    font-size: var(--fs-12);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .chip.blue .value { color: var(--accent); }
  .chip.green .value { color: var(--success); }
  .chip.orange .value { color: var(--warning); }
  .chip.purple .value { color: #b56cff; }
  .chip.red .value { color: var(--danger); }
</style>
