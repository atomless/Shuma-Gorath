<script>
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let recentEvents = [];
  export let filterOptions = {
    origins: [],
    scenarios: [],
    lanes: [],
    defenses: [],
    outcomes: []
  };
  export let filters = {
    origin: 'all',
    scenario: 'all',
    lane: 'all',
    defense: 'all',
    outcome: 'all'
  };
  export let onFilterChange = () => {};
  export let emptyState = {
    kind: 'empty',
    message: 'No recent events'
  };
  export let formatTime = () => '-';
  export let eventBadgeClass = () => 'badge';
</script>

<SectionBlock
  title="Recent Events"
  description="Last 100 recorded events with adversary-source filters."
>
  <div id="monitoring-event-filters" class="control-group panel-soft pad-sm">
    <div class="field-row">
      <label for="monitoring-filter-origin">Origin</label>
      <select
        id="monitoring-filter-origin"
        value={filters.origin}
        on:change={(event) => onFilterChange('origin', event?.currentTarget?.value || 'all')}
      >
        <option value="all">All</option>
        {#each filterOptions.origins as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label for="monitoring-filter-scenario">Scenario</label>
      <select
        id="monitoring-filter-scenario"
        value={filters.scenario}
        on:change={(event) => onFilterChange('scenario', event?.currentTarget?.value || 'all')}
      >
        <option value="all">All</option>
        {#each filterOptions.scenarios as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label for="monitoring-filter-lane">Lane</label>
      <select
        id="monitoring-filter-lane"
        value={filters.lane}
        on:change={(event) => onFilterChange('lane', event?.currentTarget?.value || 'all')}
      >
        <option value="all">All</option>
        {#each filterOptions.lanes as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label for="monitoring-filter-defense">Defense</label>
      <select
        id="monitoring-filter-defense"
        value={filters.defense}
        on:change={(event) => onFilterChange('defense', event?.currentTarget?.value || 'all')}
      >
        <option value="all">All</option>
        {#each filterOptions.defenses as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label for="monitoring-filter-outcome">Outcome</label>
      <select
        id="monitoring-filter-outcome"
        value={filters.outcome}
        on:change={(event) => onFilterChange('outcome', event?.currentTarget?.value || 'all')}
      >
        <option value="all">All</option>
        {#each filterOptions.outcomes as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>
  <TableWrapper>
    <table id="monitoring-events" class="panel panel-border">
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
          <TableEmptyRow colspan={6}>
            {emptyState.message || 'No recent events'}
          </TableEmptyRow>
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
