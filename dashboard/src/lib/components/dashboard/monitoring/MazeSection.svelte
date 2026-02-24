<script>
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let mazeStats = {
    totalHits: '0',
    uniqueCrawlers: '0',
    mazeAutoBans: '0',
    topOffender: { label: 'Top Offender', value: '-' }
  };
</script>

<SectionBlock
  title="Maze"
  description="Crawlers trapped in infinite fake pages"
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Hits" valueId="maze-total-hits" {loading} value={mazeStats.totalHits} />
    <MetricStatCard title="Unique Crawlers" valueId="maze-unique-crawlers" {loading} value={mazeStats.uniqueCrawlers} />
    <MetricStatCard title="Auto-Banned" valueId="maze-auto-bans" {loading} value={mazeStats.mazeAutoBans} />
    <MetricStatCard
      title={loading ? 'Top Offender' : mazeStats.topOffender.label}
      titleId="maze-top-offender-label"
      valueId="maze-top-offender"
      {loading}
      value={mazeStats.topOffender.value}
    />
  </div>
</SectionBlock>
