<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let honeypot = {
    totalHits: '0',
    uniqueCrawlers: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
  export let topPaths = [];
</script>

<SectionBlock
  title="Honeypot Hits"
  description="Structured honeypot telemetry (hits, offender buckets, and trap paths)."
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Hits" valueId="honeypot-total-hits" {loading} value={honeypot.totalHits} />
    <MetricStatCard title="Unique Crawlers" valueId="honeypot-unique-crawlers" {loading} value={honeypot.uniqueCrawlers} />
    <MetricStatCard
      title={loading ? 'Top Offender' : honeypot.topOffender.label}
      titleId="honeypot-top-offender-label"
      valueId="honeypot-top-offender"
      {loading}
      value={honeypot.topOffender.value}
    />
  </div>
  <div class="panel panel-border pad-md-b">
    <h3>Top Honeypot Paths</h3>
    <div id="honeypot-top-paths" class="crawler-list">
      {#if topPaths.length === 0}
        <p class="no-data">No honeypot path data yet</p>
      {:else}
        {#each topPaths as row}
          <div class="crawler-item panel panel-border">
            <span class="crawler-ip">{row.path}</span>
            <span class="crawler-hits">{formatCompactNumber(row.count, '0')} hits</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</SectionBlock>
