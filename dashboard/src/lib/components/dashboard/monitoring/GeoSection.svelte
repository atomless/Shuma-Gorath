<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let geoSummary = {
    totalViolations: '0',
    actionMix: {
      block: 0,
      challenge: 0,
      maze: 0
    }
  };
  export let geoTopCountries = [];
</script>

<SectionBlock
  title='<abbr title="Geolocation">GEO</abbr> Violations'
  description='<abbr title="Geolocation">GEO</abbr> policy actions by route and top country sources.'
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Violations" valueId="geo-violations-total" {loading} value={geoSummary.totalViolations} />
    <MetricStatCard
      title="Action Mix"
      valueId="geo-action-mix"
      {loading}
      small={true}
      value={`block ${geoSummary.actionMix.block} | challenge ${geoSummary.actionMix.challenge} | maze ${geoSummary.actionMix.maze}`}
    />
  </div>
  <div class="panel panel-border pad-md-b">
    <h3>Top Countries Triggering <abbr title="Geolocation">GEO</abbr> Actions</h3>
    <div id="geo-top-countries" class="crawler-list">
      {#if geoTopCountries.length === 0}
        <p class="no-data">No <abbr title="Geolocation">GEO</abbr> violations yet</p>
      {:else}
        {#each geoTopCountries as row}
          <div class="crawler-item panel panel-border">
            <span class="crawler-ip">{row.country}</span>
            <span class="crawler-hits">{formatCompactNumber(row.count, '0')} actions</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</SectionBlock>
