<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';

  export let loading = false;
  export let totalBans = 0;
  export let activeBans = 0;
  export let eventCount = 0;
  export let uniqueIps = 0;
</script>

<div class="stats-cards">
  <MetricStatCard title="Bans (24h)" valueId="total-bans" {loading} value={formatCompactNumber(totalBans, '0')} />
  <MetricStatCard title="Active Bans" valueId="active-bans" {loading} value={formatCompactNumber(activeBans, '0')} />
  <MetricStatCard title="Events (24h)" valueId="total-events" {loading} value={formatCompactNumber(eventCount, '0')} />
  <MetricStatCard title='Unique <abbr title="Internet Protocol">IP</abbr>s' valueId="unique-ips" {loading} value={formatCompactNumber(uniqueIps, '0')} />
</div>
