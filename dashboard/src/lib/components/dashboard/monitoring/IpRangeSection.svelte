<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let summary = {
    mode: 'off',
    totalMatches: 0,
    totalFallbacks: 0,
    uniqueSourceIds: 0,
    catalog: {
      version: '-',
      generatedAt: '-',
      ageHours: null,
      stale: false,
      managedSetCount: 0,
      managedSetStaleCount: 0,
      managedPolicyCount: 0,
      managedPolicyEnabledCount: 0,
      customRuleCount: 0,
      customRuleEnabledCount: 0,
      emergencyAllowlistCount: 0,
      managedMaxStalenessHours: 0,
      allowStaleManagedEnforce: false
    }
  };
  export let reasonRows = [];
  export let sourceRows = [];
  export let actionRows = [];
  export let detectionRows = [];
  export let sourceIdRows = [];
  export let fallbackRows = [];
  export let trendRows = [];

  const toModeLabel = (mode) => {
    const normalized = String(mode || '').toLowerCase();
    if (normalized === 'enforce') return 'Enforce';
    if (normalized === 'advisory') return 'Advisory';
    return 'Off';
  };

  const toCatalogAgeLabel = (hours) => {
    if (!Number.isFinite(hours) || hours < 0) return '-';
    return `${hours}h`;
  };

  const topRows = (rows, limit = 5) =>
    Array.isArray(rows) ? rows.slice(0, limit) : [];

  $: topSourceIdRows = topRows(sourceIdRows, 5);
  $: trendPreviewRows = topRows(trendRows, 6);
</script>

<SectionBlock
  title='<abbr title="Internet Protocol">IP</abbr> Range Policy'
  description="Match outcomes, source coverage, and managed-catalog health."
>

  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Mode" valueId="ip-range-mode" {loading} value={toModeLabel(summary.mode)} />
    <MetricStatCard title="Matches (24h)" valueId="ip-range-matches-total" {loading} value={formatCompactNumber(summary.totalMatches, '0')} />
    <MetricStatCard title="Fallbacks (24h)" valueId="ip-range-fallback-total" {loading} value={formatCompactNumber(summary.totalFallbacks, '0')} />
    <MetricStatCard title='Unique Source <abbr title="Identifiers">IDs</abbr>' valueId="ip-range-source-id-unique" {loading} value={formatCompactNumber(summary.uniqueSourceIds, '0')} />
  </div>

  <TableWrapper>
    <table class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Reason</th>
          <th class="caps-label">Count</th>
        </tr>
      </thead>
      <tbody id="ip-range-reasons">
        {#if reasonRows.length === 0}
          <TableEmptyRow colspan={2}>No <abbr title="Internet Protocol">IP</abbr>-range matches in window</TableEmptyRow>
        {:else}
          {#each reasonRows as row}
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
          <th class="caps-label">Source / Action</th>
          <th class="caps-label">Count</th>
        </tr>
      </thead>
      <tbody id="ip-range-source-actions">
        {#if sourceRows.length === 0 && actionRows.length === 0}
          <TableEmptyRow colspan={2}>No source/action data in window</TableEmptyRow>
        {:else}
          {#each sourceRows as row}
            <tr>
              <td>Source: {row.label}</td>
              <td>{formatCompactNumber(row.count, '0')}</td>
            </tr>
          {/each}
          {#each actionRows as row}
            <tr>
              <td>Action: {row.label}</td>
              <td>{formatCompactNumber(row.count, '0')}</td>
            </tr>
          {/each}
          {/if}
        </tbody>
      </table>
  </TableWrapper>

  <TableWrapper>
    <table class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Detection / Fallback</th>
          <th class="caps-label">Count</th>
        </tr>
      </thead>
      <tbody id="ip-range-detection-fallback">
        {#if detectionRows.length === 0 && fallbackRows.length === 0}
          <TableEmptyRow colspan={2}>No detection/fallback data in window</TableEmptyRow>
        {:else}
          {#each detectionRows as row}
            <tr>
              <td>Detection: {row.label}</td>
              <td>{formatCompactNumber(row.count, '0')}</td>
            </tr>
          {/each}
          {#each fallbackRows as row}
            <tr>
              <td>Fallback: {row.label}</td>
              <td>{formatCompactNumber(row.count, '0')}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>

    <table class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Top Source IDs</th>
          <th class="caps-label">Count</th>
        </tr>
      </thead>
      <tbody id="ip-range-source-ids">
        {#if topSourceIdRows.length === 0}
          <TableEmptyRow colspan={2}>No source <abbr title="Identifiers">IDs</abbr> in window</TableEmptyRow>
        {:else}
          {#each topSourceIdRows as row}
            <tr>
              <td>{row.label}</td>
              <td>{formatCompactNumber(row.count, '0')}</td>
            </tr>
          {/each}
          {/if}
        </tbody>
      </table>
  </TableWrapper>

  <div class="panel panel-border pad-md-b">
    <h3>Managed Catalog</h3>
    <ul class="metric-list" id="ip-range-catalog">
      <li><strong>Version:</strong> {loading ? '...' : summary.catalog.version}</li>
      <li><strong>Generated:</strong> {loading ? '...' : summary.catalog.generatedAt}</li>
      <li><strong>Age:</strong> {loading ? '...' : toCatalogAgeLabel(summary.catalog.ageHours)}</li>
      <li><strong>Stale:</strong> {loading ? '...' : (summary.catalog.stale ? 'Yes' : 'No')}</li>
      <li><strong>Managed Sets:</strong> {loading ? '...' : formatCompactNumber(summary.catalog.managedSetCount, '0')}</li>
      <li><strong>Managed Set Policies:</strong> {loading ? '...' : `${formatCompactNumber(summary.catalog.managedPolicyEnabledCount, '0')} enabled / ${formatCompactNumber(summary.catalog.managedPolicyCount, '0')} total`}</li>
      <li><strong>Custom Rules:</strong> {loading ? '...' : `${formatCompactNumber(summary.catalog.customRuleEnabledCount, '0')} enabled / ${formatCompactNumber(summary.catalog.customRuleCount, '0')} total`}</li>
      <li><strong>Emergency Allowlist <abbr title="Classless Inter-Domain Routing">CIDRs</abbr>:</strong> {loading ? '...' : formatCompactNumber(summary.catalog.emergencyAllowlistCount, '0')}</li>
      <li><strong>Stale Managed Sets:</strong> {loading ? '...' : formatCompactNumber(summary.catalog.managedSetStaleCount, '0')}</li>
      <li><strong>Max Staleness:</strong> {loading ? '...' : `${formatCompactNumber(summary.catalog.managedMaxStalenessHours, '0')}h`}</li>
      <li><strong>Allow Stale Enforce:</strong> {loading ? '...' : (summary.catalog.allowStaleManagedEnforce ? 'Yes' : 'No')}</li>
    </ul>
  </div>

  <div class="panel panel-border pad-md-b">
    <h3>Hourly Match Trend (Recent)</h3>
    <ul class="metric-list" id="ip-range-trend-list">
      {#if trendPreviewRows.length === 0}
        <li class="text-muted">No trend data in window</li>
      {:else}
        {#each trendPreviewRows as row}
          <li><strong>{row.label}:</strong> {formatCompactNumber(row.count, '0')}</li>
        {/each}
      {/if}
    </ul>
  </div>
</SectionBlock>
