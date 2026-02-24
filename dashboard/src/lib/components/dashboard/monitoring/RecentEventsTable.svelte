<script>
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let recentEvents = [];
  export let formatTime = () => '-';
  export let eventBadgeClass = () => 'badge';
</script>

<SectionBlock
  title="Recent Events"
  description="Last 100 recorded events"
>
  <TableWrapper>
    <table id="events" class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">Time</th>
          <th class="caps-label">Type</th>
          <th class="caps-label"><abbr title="Internet Protocol">IP</abbr></th>
          <th class="caps-label">Reason</th>
          <th class="caps-label">Outcome</th>
          <th class="caps-label">Admin</th>
        </tr>
      </thead>
      <tbody>
        {#if recentEvents.length === 0}
          <TableEmptyRow colspan={6}>No recent events</TableEmptyRow>
        {:else}
          {#each recentEvents as ev}
            <tr>
              <td>{formatTime(ev.ts)}</td>
              <td><span class={eventBadgeClass(ev.event)}>{ev.event || '-'}</span></td>
              <td><code>{ev.ip || '-'}</code></td>
              <td>{ev.reason || '-'}</td>
              <td>{ev.outcome || '-'}</td>
              <td>{ev.admin || '-'}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</SectionBlock>
