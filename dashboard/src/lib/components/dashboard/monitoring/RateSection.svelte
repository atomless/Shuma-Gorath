<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let rateSummary = {
    totalViolations: '0',
    uniqueOffenders: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let rateOutcomeRows = [];
</script>

<SectionBlock
  title="Rate Limiting Violations"
  description="Rate-limit outcomes and top offender bucket."
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Violations" valueId="rate-violations-total" {loading} value={rateSummary.totalViolations} />
    <MetricStatCard title="Unique Offenders" valueId="rate-violations-unique" {loading} value={rateSummary.uniqueOffenders} />
    <MetricStatCard
      title={loading ? 'Top Offender' : rateSummary.topOffender.label}
      titleId="rate-top-offender-label"
      valueId="rate-top-offender"
      {loading}
      value={rateSummary.topOffender.value}
    />
  </div>
  <div class="panel panel-border pad-md-b">
    <h3>Enforcement Outcomes</h3>
    <ul id="rate-outcomes-list" class="metric-list">
      {#if rateOutcomeRows.length === 0}
        <li class="text-muted">No outcomes yet</li>
      {:else}
        {#each rateOutcomeRows as row}
          <li><strong>{row.label}:</strong> {formatCompactNumber(row.count, '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>
</SectionBlock>
