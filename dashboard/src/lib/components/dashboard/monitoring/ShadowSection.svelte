<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let shadowSummary = {
    totalActions: '0',
    passThroughTotal: '0',
    topAction: {
      label: 'Top Simulated Action',
      value: 'None'
    },
    actions: []
  };
</script>

<SectionBlock
  title="Shadow Mode"
  description="Shadow-mode actions that would have imposed friction or enforcement, plus clean pass-through totals."
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard
      title="Simulated Actions"
      valueId="shadow-total-actions"
      {loading}
      value={shadowSummary.totalActions}
    />
    <MetricStatCard
      title="Pass Through"
      valueId="shadow-pass-through-total"
      {loading}
      value={shadowSummary.passThroughTotal}
    />
    <MetricStatCard
      title={shadowSummary.topAction.label}
      valueId="shadow-top-action"
      {loading}
      value={shadowSummary.topAction.value}
    />
  </div>

  <div class="panel panel-border pad-md-b">
    <h3>Simulated Action Mix</h3>
    <ul id="shadow-action-list" class="metric-list">
      {#if shadowSummary.actions.length === 0}
        <li class="text-muted">No simulated actions yet</li>
      {:else}
        {#each shadowSummary.actions as row}
          <li><strong>{row.label}:</strong> {formatCompactNumber(row.count, '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>
</SectionBlock>
