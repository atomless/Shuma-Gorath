<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let tarpitSummary = {
    activationsProgressive: '0',
    progressAdvanced: '0',
    fallbackMaze: '0',
    fallbackBlock: '0',
    escalationShortBan: '0',
    escalationBlock: '0',
    progressOutcomes: [],
    budgetOutcomes: [],
    escalationOutcomes: []
  };
</script>

<SectionBlock
  title="Tarpit Progression"
  description="Progressive tarpit activations, fallback paths, and escalation outcomes."
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard
      title="Activations"
      valueId="tarpit-activations-progressive"
      {loading}
      value={tarpitSummary.activationsProgressive}
    />
    <MetricStatCard
      title="Progress Advanced"
      valueId="tarpit-progress-advanced"
      {loading}
      value={tarpitSummary.progressAdvanced}
    />
  </div>

  <div class="panel panel-border pad-md-b">
    <h3>Budget Fallback Outcomes</h3>
    <ul id="tarpit-budget-outcomes-list" class="metric-list">
      {#if tarpitSummary.budgetOutcomes.length === 0}
        <li class="text-muted">No fallback outcomes yet</li>
      {:else}
        {#each tarpitSummary.budgetOutcomes as row}
          <li><strong>{row[0]}:</strong> {formatCompactNumber(row[1], '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>

  <div class="panel panel-border pad-md-b">
    <h3>Escalation Outcomes</h3>
    <ul id="tarpit-escalation-outcomes-list" class="metric-list">
      {#if tarpitSummary.escalationOutcomes.length === 0}
        <li class="text-muted">No escalation outcomes yet</li>
      {:else}
        {#each tarpitSummary.escalationOutcomes as row}
          <li><strong>{row[0]}:</strong> {formatCompactNumber(row[1], '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>
</SectionBlock>
