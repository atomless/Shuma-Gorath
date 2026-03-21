<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';

  export let loading = false;
  export let totalBans = 0;
  export let activeBans = 0;
  export let eventCount = 0;
  export let uniqueIps = 0;

  const formatStatValue = (value, fallback = '0') => {
    if (value === null || value === undefined || value === '') return String(fallback);
    const parsed = Number(value);
    if (Number.isFinite(parsed)) return formatCompactNumber(parsed, fallback);
    return String(value);
  };
</script>

<div class="stats-cards">
  <MetricStatCard title="Bans (24h)" valueId="total-bans" {loading} value={formatStatValue(totalBans, '0')} />
  <MetricStatCard title="Active Bans" valueId="active-bans" {loading} value={formatStatValue(activeBans, '0')} />
  <MetricStatCard title="Events (24h)" valueId="total-events" {loading} value={formatStatValue(eventCount, '0')} />
  <MetricStatCard title='Unique <abbr title="Internet Protocol">IP</abbr>s' valueId="unique-ips" {loading} value={formatStatValue(uniqueIps, '0')} />
</div>
