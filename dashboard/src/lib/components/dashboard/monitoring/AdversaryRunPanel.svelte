<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import { formatAdversarySimLaneLabel } from '../../../domain/adversary-sim.js';
  import {
    formatIdentityRealismSummary,
    formatTransportRealismSummary
  } from '../monitoring-view-model.js';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let runRows = [];
  export let activeBanCount = 0;
  export let freshnessStateKey = 'stale';
  export let formatTime = () => '-';
  export let title = 'Recent Adversary Runs';
  export let description = 'Recent runs linked to defense deltas in monitoring and ban workflows.';
  export let summaryLabel = 'Active bans visible in <a href="#ip-bans">IP Bans</a>';
  export let emptyText = 'No recent adversary runs were observed in the current monitoring window.';
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
  const formatCoverageSummary = (row = {}) => {
    const record = row?.coverageSummary && typeof row.coverageSummary === 'object'
      ? row.coverageSummary
      : null;
    if (!record) return '-';
    const touched = formatCompactNumber(record.touchedSurfaceCount || 0, '0');
    const total = formatCompactNumber(record.totalSurfaceCount || 0, '0');
    const evidenceLabel = String(record.evidenceLabel || '').trim();
    return [evidenceLabel, `${touched} / ${total} surfaces`]
      .filter(Boolean)
      .join(' | ');
  };
  const formatCountLabel = (value, singular, plural) => {
    const count = Number(value || 0);
    const normalized = Number.isFinite(count) ? count : 0;
    const label = normalized === 1 ? singular : plural;
    return `${formatCompactNumber(normalized, '0')} ${label}`;
  };
  const formatMonitoringDeltasSummary = (row = {}) => {
    if (row?.hasSharedObservedTelemetry === false) {
      return 'No shared telemetry observed';
    }
    return `${formatCountLabel(row.monitoringEventCount, 'event', 'events')} across ${formatCountLabel(row.defenseDeltaCount, 'defense', 'defenses')}`;
  };
  const formatProviderLabel = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'openai') return 'OpenAI';
    if (normalized === 'xai') return 'xAI';
    return humanizeToken(value);
  };
  const formatLaneLabel = (value) => {
    const normalized = String(value || '').trim();
    if (!normalized) return '-';
    return formatAdversarySimLaneLabel(normalized, '-');
  };
  const joinSummaryParts = (parts = []) => {
    const filtered = parts.filter(Boolean);
    return filtered.length > 0 ? filtered.join(' | ') : 'Not materialized';
  };
  const extractRealismReceipt = (row = {}) => {
    const llmReceipt =
      row?.llmRuntimeSummary?.latestRealismReceipt
      && typeof row.llmRuntimeSummary.latestRealismReceipt === 'object'
        ? row.llmRuntimeSummary.latestRealismReceipt
        : null;
    return llmReceipt || (
      row?.latestScraplingRealismReceipt && typeof row.latestScraplingRealismReceipt === 'object'
        ? row.latestScraplingRealismReceipt
        : null
    );
  };
  const extractIdentitySummary = (row = {}) =>
    row?.identitySummary && typeof row.identitySummary === 'object'
      ? row.identitySummary
      : extractRealismReceipt(row);
  const extractTransportSummary = (row = {}) =>
    row?.transportSummary && typeof row.transportSummary === 'object'
      ? row.transportSummary
      : extractRealismReceipt(row);
  const formatExecutionSummary = (row = {}) => {
    const summary = row?.llmRuntimeSummary && typeof row.llmRuntimeSummary === 'object'
      ? row.llmRuntimeSummary
      : null;
    if (summary) {
      const parts = [];
      const generationSource = humanizeToken(summary.generationSource || '');
      const providerLabel = formatProviderLabel(summary.provider || '');
      const modelId = String(summary.modelId || '').trim();
      const executed = formatCompactNumber(summary.executedActionCount || 0, '0');
      const generated = formatCompactNumber(summary.generatedActionCount || 0, '0');
      const terminalFailure = humanizeToken(summary.terminalFailure || '');
      const fallbackReason = humanizeToken(summary.fallbackReason || '');
      const outcome = summary.failedTickCount > 0 || summary.failureClass || summary.error || summary.terminalFailure
        ? 'failed'
        : 'passed';
      if (generationSource) parts.push(generationSource);
      if (providerLabel || modelId) {
        parts.push([providerLabel, modelId].filter(Boolean).join(' '));
      }
      parts.push(`${executed} / ${generated} actions`);
      parts.push(outcome);
      if (terminalFailure) parts.push(terminalFailure);
      else if (fallbackReason) parts.push(fallbackReason);
      return joinSummaryParts(parts);
    }
    const receipt = row?.latestScraplingRealismReceipt && typeof row.latestScraplingRealismReceipt === 'object'
      ? row.latestScraplingRealismReceipt
      : null;
    if (!receipt && !row?.scraplingActivityCount) return 'Not materialized';
    const parts = [];
    const activityCount = Number(row?.scraplingActivityCount || receipt?.activityCount || 0);
    if (activityCount > 0) parts.push(`${formatCompactNumber(activityCount, '0')} activities executed`);
    return joinSummaryParts(parts);
  };
  const formatIdentitySummary = (row = {}) => {
    const summary = extractIdentitySummary(row);
    if (!summary) return 'Not materialized';
    const identitySummary = formatIdentityRealismSummary(summary);
    return identitySummary || 'Not materialized';
  };
  const formatTransportSummary = (row = {}) => {
    const summary = extractTransportSummary(row);
    if (!summary) return 'Not materialized';
    const transportSummary = formatTransportRealismSummary(summary);
    return transportSummary || 'Not materialized';
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
          <th class="caps-label">Started</th>
          <th class="caps-label">Lane</th>
          <th class="caps-label">Modes</th>
          <th class="caps-label">Categories</th>
          <th class="caps-label">Execution</th>
          <th class="caps-label">Identity</th>
          <th class="caps-label">Transport</th>
          <th class="caps-label">Coverage</th>
          <th class="caps-label">Monitoring Deltas</th>
          <th class="caps-label">Ban Outcomes</th>
        </tr>
      </thead>
      <tbody>
        {#if runRows.length === 0}
          <TableEmptyRow colspan={10}>No adversary runs</TableEmptyRow>
        {:else}
          {#each runRows as row}
            <tr>
              <td>{formatTime(row.firstTs)}</td>
              <td>{formatLaneLabel(row.lane)}</td>
              <td>{formatTokenList(row.observedFulfillmentModes)}</td>
              <td>{formatTokenList(row.observedCategoryIds)}</td>
              <td>{formatExecutionSummary(row)}</td>
              <td>{formatIdentitySummary(row)}</td>
              <td>{formatTransportSummary(row)}</td>
              <td>{formatCoverageSummary(row)}</td>
              <td>{formatMonitoringDeltasSummary(row)}</td>
              <td>{formatCompactNumber(row.banOutcomeCount, '0')}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</SectionBlock>
