<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let challengeSummary = {
    totalFailures: '0',
    uniqueOffenders: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let notABotSummary = {
    served: '0',
    submitted: '0',
    pass: '0',
    escalate: '0',
    fail: '0',
    replay: '0',
    abandonmentsEstimated: '0',
    abandonmentRate: '0.0%'
  };
  export let challengeReasonRows = [];
  export let notABotOutcomeRows = [];
  export let notABotLatencyRows = [];
  export let challengeTrendCanvas = null;
</script>

<SectionBlock
  title="Challenge Outcomes"
  description="Challenge rejections and confirmed attack signals by reason class, with trend and top offender."
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Rejections" valueId="challenge-failures-total" {loading} value={challengeSummary.totalFailures} />
    <MetricStatCard title="Unique Offenders" valueId="challenge-failures-unique" {loading} value={challengeSummary.uniqueOffenders} />
    <MetricStatCard
      title={loading ? 'Top Offender' : challengeSummary.topOffender.label}
      titleId="challenge-top-offender-label"
      valueId="challenge-top-offender"
      {loading}
      value={challengeSummary.topOffender.value}
    />
  </div>
  <div class="panel panel-border">
    <h3 class="caps-label">Not-a-Bot Outcomes (24h)</h3>
    <div class="stats-cards stats-cards--compact">
      <MetricStatCard title="Served" titleLevel="h4" {loading} small={true} value={notABotSummary.served} />
      <MetricStatCard title="Submitted" titleLevel="h4" {loading} small={true} value={notABotSummary.submitted} />
      <MetricStatCard title="Pass / Escalate" titleLevel="h4" {loading} small={true} value={`${notABotSummary.pass} / ${notABotSummary.escalate}`} />
      <MetricStatCard title="Fail / Replay" titleLevel="h4" {loading} small={true} value={`${notABotSummary.fail} / ${notABotSummary.replay}`} />
      <MetricStatCard title="Abandonment" titleLevel="h4" {loading} small={true} value={`${notABotSummary.abandonmentsEstimated} (${notABotSummary.abandonmentRate})`} />
    </div>
    <TableWrapper>
      <table class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Outcome</th>
            <th class="caps-label">Count</th>
          </tr>
        </thead>
        <tbody id="not-a-bot-outcomes">
          {#if notABotOutcomeRows.length === 0}
            <TableEmptyRow colspan={2}>No not-a-bot submissions in window</TableEmptyRow>
          {:else}
            {#each notABotOutcomeRows as row}
              <tr>
                <td>{row.label}</td>
                <td>{formatCompactNumber(row.count, '0')}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
      <table class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Solve Latency</th>
            <th class="caps-label">Count</th>
          </tr>
        </thead>
        <tbody id="not-a-bot-latency">
          {#if notABotLatencyRows.length === 0}
            <TableEmptyRow colspan={2}>No solve latency data in window</TableEmptyRow>
          {:else}
            {#each notABotLatencyRows as row}
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
  <div class="chart-container panel-soft panel-border pad-md-trb">
    <canvas id="challengeFailuresTrendChart" bind:this={challengeTrendCanvas}></canvas>
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
        <tbody id="challenge-failure-reasons">
          {#if challengeReasonRows.length === 0}
            <TableEmptyRow colspan={2}>No failures in window</TableEmptyRow>
          {:else}
            {#each challengeReasonRows as row}
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
</SectionBlock>
