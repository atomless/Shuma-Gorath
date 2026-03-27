<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let runEvidence = null;
  export let formatTime = () => '-';

  const TOKEN_ACRONYMS = Object.freeze({
    ai: 'AI',
    http: 'HTTP',
    ip: 'IP',
    pow: 'PoW'
  });
  const toArray = (value) => (Array.isArray(value) ? value : []);
  const humanizeToken = (value) => {
    const normalized = String(value || '').trim();
    if (!normalized) return '-';
    return normalized
      .replace(/[_-]+/g, ' ')
      .replace(/\s+/g, ' ')
      .split(' ')
      .map((word) => {
        const lowered = word.toLowerCase();
        if (TOKEN_ACRONYMS[lowered]) return TOKEN_ACRONYMS[lowered];
        return lowered.charAt(0).toUpperCase() + lowered.slice(1);
      })
      .join(' ');
  };
  const formatList = (values = []) => {
    const rows = toArray(values).map((value) => humanizeToken(value)).filter(Boolean);
    return rows.length === 0 ? '-' : rows.join(', ');
  };
  const formatSample = (receipt) => {
    const method = String(receipt?.sampleRequestMethod || '').trim();
    const path = String(receipt?.sampleRequestPath || '').trim();
    const status = receipt?.sampleResponseStatus;
    if (!method && !path && (status === null || status === undefined)) return '-';
    const requestPart = [method, path].filter(Boolean).join(' ');
    if (status === null || status === undefined || status === '') return requestPart || '-';
    return `${requestPart || 'Observed'} -> ${status}`;
  };
</script>

<SectionBlock
  title="Scrapling"
  description="Receipt-backed proof from the most recent Scrapling run: observed personas, categories, defense-surface coverage, and sample attack receipts."
  id="red-team-scrapling-evidence"
>
  {#if !runEvidence}
    <p class="text-muted">No receipt-backed Scrapling run is currently visible in the bounded recent-run window.</p>
  {:else}
    <div class="stats-cards">
      <MetricStatCard
        title="Required Surfaces"
        valueId="red-team-scrapling-required-surfaces"
        {loading}
        value={formatCompactNumber(runEvidence.ownedSurfaceCoverage?.requiredSurfaceCount || 0, '0')}
      />
      <MetricStatCard
        title="Satisfied Surfaces"
        valueId="red-team-scrapling-satisfied-surfaces"
        {loading}
        value={formatCompactNumber(runEvidence.ownedSurfaceCoverage?.satisfiedSurfaceCount || 0, '0')}
      />
      <MetricStatCard
        title="Blocking Surfaces"
        valueId="red-team-scrapling-blocking-surfaces"
        {loading}
        value={formatCompactNumber(runEvidence.ownedSurfaceCoverage?.blockingSurfaceCount || 0, '0')}
      />
    </div>

    <div class="panel panel-border pad-md-b">
      <h3>Surface Checklist</h3>
      <ul id="red-team-scrapling-surface-checklist" class="metric-list metric-list--plain">
        {#if toArray(runEvidence.ownedSurfaceCoverage?.surfaceChecklistRows).length === 0}
          <li class="text-muted">No surface checklist available</li>
        {:else}
          {#each runEvidence.ownedSurfaceCoverage.surfaceChecklistRows as row (row.surfaceId)}
            <li
              aria-label={`${row.state} ${row.surfaceLabel || humanizeToken(row.surfaceId)}`}
              data-surface-id={row.surfaceId}
              data-surface-state={row.state}
            >
              <strong aria-hidden="true">
                {#if row.state === 'satisfied'}&#10003;{:else if row.state === 'not_required'}-{:else}&#10005;{/if}
              </strong>
              {' '}
              {row.surfaceLabel || humanizeToken(row.surfaceId)}
              <span class="text-muted"> {row.stateLabel || humanizeToken(row.state)}</span>
            </li>
          {/each}
        {/if}
      </ul>
    </div>

    <div class="status-rows">
      <div class="info-row">
        <span class="info-label text-muted">Run:</span>
        <span class="status-value"><code>{runEvidence.runId}</code> | {formatTime(runEvidence.lastTs)}</span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Scrapling Modes Used:</span>
        <span class="status-value">{formatList(runEvidence.observedFulfillmentModes)}</span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Non-human Categories Fulfilled:</span>
        <span class="status-value">{formatList(runEvidence.observedCategoryIds)}</span>
      </div>
      <div class="info-row">
        <span class="info-label text-muted">Defence Surfaces Covered:</span>
        <span class="status-value">
          {formatCompactNumber(runEvidence.ownedSurfaceCoverage?.satisfiedSurfaceCount || 0, '0')}
          /
          {formatCompactNumber(runEvidence.ownedSurfaceCoverage?.requiredSurfaceCount || 0, '0')}
          surfaces satisfied
        </span>
      </div>
    </div>

    <TableWrapper>
      <table id="red-team-scrapling-evidence-receipts" class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">Surface</th>
            <th class="caps-label">Contract</th>
            <th class="caps-label">Observed</th>
            <th class="caps-label">Attempts</th>
            <th class="caps-label">Sample</th>
          </tr>
        </thead>
        <tbody>
          {#if toArray(runEvidence.ownedSurfaceCoverage?.receipts).length === 0}
            <TableEmptyRow colspan={5}>No owned-surface receipts</TableEmptyRow>
          {:else}
            {#each runEvidence.ownedSurfaceCoverage.receipts as receipt (receipt.surfaceId)}
              <tr>
                <td>{receipt.surfaceLabel || humanizeToken(receipt.surfaceId)}</td>
                <td>{humanizeToken(receipt.successContract)}</td>
                <td>
                  {humanizeToken(receipt.coverageStatus)}
                  <span class="text-muted"> | {receipt.surfaceStateLabel || (receipt.satisfied ? 'satisfied' : 'blocked')}</span>
                </td>
                <td>{formatCompactNumber(receipt.attemptCount || 0, '0')}</td>
                <td><code>{formatSample(receipt)}</code></td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </TableWrapper>
  {/if}
</SectionBlock>
