<script>
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let recentEvents = [];
  export let filterOptions = {
    origins: [],
    modes: [],
    scenarios: [],
    lanes: [],
    defenses: [],
    outcomes: []
  };
  export let filters = {
    origin: 'all',
    mode: 'all',
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
    <div class="input-row">
      <label class="control-label control-label--wide" for="monitoring-filter-origin">Origin</label>
      <select
        class="input-field"
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
    <div class="input-row">
      <label class="control-label control-label--wide" for="monitoring-filter-mode">Mode</label>
      <select
        class="input-field"
        id="monitoring-filter-mode"
        value={filters.mode}
        on:change={(event) => onFilterChange('mode', event?.currentTarget?.value || 'all')}
      >
        <option value="all">All</option>
        {#each filterOptions.modes as option}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
    <div class="input-row">
      <label class="control-label control-label--wide" for="monitoring-filter-scenario">Scenario</label>
      <select
        class="input-field"
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
    <div class="input-row">
      <label class="control-label control-label--wide" for="monitoring-filter-lane">Lane</label>
      <select
        class="input-field"
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
    <div class="input-row">
      <label class="control-label control-label--wide" for="monitoring-filter-defense">Defense</label>
      <select
        class="input-field"
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
    <div class="input-row">
      <label class="control-label control-label--wide" for="monitoring-filter-outcome">Outcome</label>
      <select
        class="input-field"
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
          <th class="caps-label">Mode</th>
          <th class="caps-label"><abbr title="Internet Protocol">IP</abbr></th>
          <th class="caps-label">Reason</th>
          <th class="caps-label">Outcome</th>
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
              <td>{ev.executionModeLabel || '-'}</td>
              <td><code>{ev.ip || '-'}</code></td>
              <td>{ev.reason || '-'}</td>
              <td>{ev.outcome || '-'}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
</SectionBlock>
