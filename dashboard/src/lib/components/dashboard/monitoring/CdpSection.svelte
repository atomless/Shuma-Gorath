<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import MetricStatCard from '../primitives/MetricStatCard.svelte';
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let loading = false;
  export let cdpDetections = 0;
  export let cdpAutoBans = 0;
  export let cdpFingerprintEvents = 0;
  export let cdpFingerprintFlowViolations = 0;
  export let recentCdpEvents = [];
  export let formatTime = () => '-';
  export let readCdpField = () => '-';
</script>

<SectionBlock
  title='<abbr title="Chrome DevTools Protocol">CDP</abbr> Detections'
  description='Browser automation detection and bans in the last 24hrs'
>
  <div class="stats-cards stats-cards--compact">
    <MetricStatCard title="Total Detections" valueId="cdp-total-detections" {loading} value={formatCompactNumber(cdpDetections, '0')} />
    <MetricStatCard title="Auto-Bans" valueId="cdp-total-auto-bans" {loading} value={formatCompactNumber(cdpAutoBans, '0')} />
    <MetricStatCard title='<abbr title="Fingerprint">FP</abbr> Mismatch Events' valueId="cdp-fp-events" {loading} value={formatCompactNumber(cdpFingerprintEvents, '0')} />
    <MetricStatCard title='<abbr title="Fingerprint">FP</abbr> Flow Violations' valueId="cdp-fp-flow-violations" {loading} value={formatCompactNumber(cdpFingerprintFlowViolations, '0')} />
  </div>
  <TableWrapper>
    <table id="cdp-events" class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Time</th>
          <th class="caps-label"><abbr title="Internet Protocol">IP</abbr></th>
          <th class="caps-label">Type</th>
          <th class="caps-label">Tier</th>
          <th class="caps-label">Score</th>
          <th class="caps-label">Details</th>
        </tr>
      </thead>
      <tbody>
        {#if recentCdpEvents.length === 0}
          <TableEmptyRow colspan={6}>
            No <abbr title="Chrome DevTools Protocol">CDP</abbr> detections or auto-bans in the selected window
          </TableEmptyRow>
        {:else}
          {#each recentCdpEvents as ev}
            {@const reason = String(ev.reason || '')}
            {@const outcome = String(ev.outcome || '-')}
            {@const isBan = reason.toLowerCase() === 'cdp_automation'}
            {@const tierSource = isBan ? outcome : reason}
            {@const tier = readCdpField(tierSource, 'tier').toUpperCase()}
            {@const score = readCdpField(tierSource, 'score')}
            {@const details = isBan
              ? `Auto-ban: ${outcome}`
              : (outcome.toLowerCase().startsWith('checks:') ? outcome.replace(/^checks:/i, 'Checks: ') : outcome)}
            <tr>
              <td>{formatTime(ev.ts)}</td>
              <td><code>{ev.ip || '-'}</code></td>
              <td><span class={`badge ${isBan ? 'ban' : 'challenge'}`}>{isBan ? 'BAN' : 'DETECTION'}</span></td>
              <td>{tier}</td>
              <td>{score}</td>
              <td>{details}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</SectionBlock>
