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
  export let title = 'Recent Adversary Runs';
  export let description = 'Run identifiers linked to defense deltas in monitoring and ban workflows.';
  export let summaryLabel = 'Active bans visible in <a href="#ip-bans">IP Bans</a>';
  export let emptyText = 'No adversary run identifiers were observed in the current monitoring window.';
  export let degradedText =
    'Monitoring freshness is degraded or stale. Missing run rows may indicate delayed telemetry rather than no attacks.';

  const TOKEN_ACRONYMS = Object.freeze({
    ai: 'AI',
    http: 'HTTP',
    ip: 'IP',
    pow: 'PoW'
  });
  $: isDegraded = freshnessStateKey === 'degraded' || freshnessStateKey === 'stale';
  $: activeBanCountLabel = (() => {
    if (activeBanCount === null || activeBanCount === undefined || activeBanCount === '') return '-';
    const parsed = Number(activeBanCount);
    if (Number.isFinite(parsed)) return formatCompactNumber(parsed, '0');
    return String(activeBanCount);
  })();
  const humanizeToken = (value) => String(value || '')
    .trim()
    .replace(/[_-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .split(' ')
    .map((word) => {
      const lowered = word.toLowerCase();
      if (TOKEN_ACRONYMS[lowered]) return TOKEN_ACRONYMS[lowered];
      return lowered.charAt(0).toUpperCase() + lowered.slice(1);
    })
    .join(' ');
  const formatTokenList = (values = []) => {
    const rows = Array.isArray(values)
      ? values.map((value) => humanizeToken(value)).filter(Boolean)
      : [];
    return rows.length === 0 ? '-' : rows.join(', ');
  };
  const formatCoverageSummary = (coverage) => {
    const record = coverage && typeof coverage === 'object' ? coverage : null;
    if (!record) return '-';
    const satisfied = formatCompactNumber(record.satisfiedSurfaceCount || 0, '0');
    const required = formatCompactNumber(record.requiredSurfaceCount || 0, '0');
    const status = humanizeToken(record.overallStatus || '');
    return `${status} | ${satisfied} / ${required} surfaces`;
  };
</script>

<SectionBlock
  {title}
  {description}
>
  <p id="adversary-run-summary" class="control-desc text-muted">
    {@html summaryLabel}: {activeBanCountLabel}
  </p>
  {#if !loading && runRows.length === 0 && isDegraded}
    <p id="adversary-run-state-degraded" class="message warning">
      {degradedText}
    </p>
  {:else if !loading && runRows.length === 0}
    <p id="adversary-run-state-empty" class="text-muted">
      {emptyText}
    </p>
  {/if}
  <TableWrapper>
    <table id="adversary-runs" class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Run <abbr title="Identifier">ID</abbr></th>
          <th class="caps-label">Lane</th>
          <th class="caps-label">Profile</th>
          <th class="caps-label">Modes</th>
          <th class="caps-label">Categories</th>
          <th class="caps-label">Coverage</th>
          <th class="caps-label">Last Event</th>
          <th class="caps-label">Monitoring Deltas</th>
          <th class="caps-label">Ban Outcomes</th>
        </tr>
      </thead>
      <tbody>
        {#if runRows.length === 0}
          <TableEmptyRow colspan={9}>No adversary runs</TableEmptyRow>
        {:else}
          {#each runRows as row}
            <tr>
              <td><code>{row.runId}</code></td>
              <td>{row.lane || '-'}</td>
              <td>{row.profile || '-'}</td>
              <td>{formatTokenList(row.observedFulfillmentModes)}</td>
              <td>{formatTokenList(row.observedCategoryIds)}</td>
              <td>{formatCoverageSummary(row.ownedSurfaceCoverage)}</td>
              <td>{formatTime(row.lastTs)}</td>
              <td>
                {formatCompactNumber(row.monitoringEventCount, '0')} events
                · {formatCompactNumber(row.defenseDeltaCount, '0')} defenses
              </td>
              <td>{formatCompactNumber(row.banOutcomeCount, '0')}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</SectionBlock>
