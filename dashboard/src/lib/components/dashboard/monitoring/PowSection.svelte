<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let powSummary = {
    totalAttempts: '0',
    totalSuccesses: '0',
    totalFailures: '0',
    uniqueOffenders: '0',
    successRate: '-',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let powReasonRows = [];
  export let powOutcomeRows = [];
  export let powTrendCanvas = null;
</script>

<SectionBlock
  title='<abbr title="Proof of Work">PoW</abbr> Verification'
  description='<abbr title="Proof of Work">PoW</abbr> verify outcomes with failure reasons, trend, and top offender.'
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Verifies" valueId="pow-total-attempts" {loading} value={powSummary.totalAttempts} />
    <MetricStatCard title="Total Failures" valueId="pow-failures-total" {loading} value={powSummary.totalFailures} />
    <MetricStatCard title="Unique Offenders" valueId="pow-failures-unique" {loading} value={powSummary.uniqueOffenders} />
    <MetricStatCard
      title={loading ? 'Top Offender' : powSummary.topOffender.label}
      titleId="pow-top-offender-label"
      valueId="pow-top-offender"
      {loading}
      value={powSummary.topOffender.value}
    />
  </div>
  <div class="chart-container panel-soft panel-border pad-md-trb">
    <canvas id="powFailuresTrendChart" bind:this={powTrendCanvas}></canvas>
  </div>
  <div class="panel panel-border">
    <TableWrapper>
      <table class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Reason</th>
            <th class="caps-label">Count</th>
          </tr>
        </thead>
        <tbody id="pow-failure-reasons">
          {#if powReasonRows.length === 0}
            <TableEmptyRow colspan={2}>No failures in window</TableEmptyRow>
          {:else}
            {#each powReasonRows as row}
              <tr>
                <td>{row.label}</td>
                <td>{formatCompactNumber(row.count, '0')}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </TableWrapper>
  </div>
  <div class="panel panel-border pad-md-b">
    <h3>Verify Outcomes</h3>
    <ul id="pow-outcomes-list" class="metric-list">
      {#if powOutcomeRows.length === 0}
        <li class="text-muted">No verify outcomes yet</li>
      {:else}
        {#each powOutcomeRows as row}
          <li><strong>{row.label}:</strong> {formatCompactNumber(row.count, '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>
</SectionBlock>
