<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let runRows = [];
  export let activeBanCount = 0;
  export let freshnessStateKey = 'stale';
  export let formatTime = () => '-';

  $: isDegraded = freshnessStateKey === 'degraded' || freshnessStateKey === 'stale';
</script>

<SectionBlock
  title="Recent Adversary Runs"
  description="Run identifiers linked to defense deltas in monitoring and ban workflows."
>
  <p id="adversary-run-summary" class="control-desc text-muted">
    Active bans visible in <a href="#ip-bans">IP Bans</a>: {formatCompactNumber(activeBanCount, '0')}
  </p>
  {#if !loading && runRows.length === 0 && isDegraded}
    <p id="adversary-run-state-degraded" class="message warning">
      Monitoring freshness is degraded or stale. Missing run rows may indicate delayed telemetry rather than no attacks.
    </p>
  {:else if !loading && runRows.length === 0}
    <p id="adversary-run-state-empty" class="text-muted">
      No adversary run identifiers were observed in the current monitoring window.
    </p>
  {/if}
  <TableWrapper>
    <table id="adversary-runs" class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Run <abbr title="Identifier">ID</abbr></th>
          <th class="caps-label">Lane</th>
          <th class="caps-label">Profile</th>
          <th class="caps-label">Last Event</th>
          <th class="caps-label">Monitoring Deltas</th>
          <th class="caps-label">Ban Outcomes</th>
          <th class="caps-label">Links</th>
        </tr>
      </thead>
      <tbody>
        {#if runRows.length === 0}
          <TableEmptyRow colspan={7}>No adversary runs</TableEmptyRow>
        {:else}
          {#each runRows as row}
            <tr>
              <td><code>{row.runId}</code></td>
              <td>{row.lane || '-'}</td>
              <td>{row.profile || '-'}</td>
              <td>{formatTime(row.lastTs)}</td>
              <td>
                {formatCompactNumber(row.monitoringEventCount, '0')} events
                · {formatCompactNumber(row.defenseDeltaCount, '0')} defenses
              </td>
              <td>{formatCompactNumber(row.banOutcomeCount, '0')}</td>
              <td>
                <a href={row.monitoringHref || '#monitoring'}>Monitoring</a>
                ·
                <a href={row.ipBansHref || '#ip-bans'}>IP Bans</a>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</SectionBlock>
