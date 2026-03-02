<script>
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let selectedTimeRange = 'hour';
  export let onSelectTimeRange = null;
  export let eventTypesCanvas = null;
  export let topIpsCanvas = null;
  export let timeSeriesCanvas = null;

  const selectRange = (range) => {
    if (typeof onSelectTimeRange === 'function') {
      onSelectTimeRange(range);
    }
  };
</script>

<div class="charts-row">
  <div class="chart-container panel-soft panel-border pad-md">
    <h2>Event Types (24h)</h2>
    <canvas id="eventTypesChart" bind:this={eventTypesCanvas}></canvas>
  </div>
  <div class="chart-container panel-soft panel-border pad-md">
    <h2>Top 10 <abbr title="Internet Protocol">IP</abbr>s by Events</h2>
    <canvas id="topIpsChart" bind:this={topIpsCanvas}></canvas>
  </div>
</div>

<SectionBlock
  title="Events Over Time"
  description="Recent events plotted over various time windows"
  rootClass="section"
>
  <div class="chart-header">
    <div class="time-range-buttons">
      <button type="button" class="btn time-btn" class:active={selectedTimeRange === 'hour'} data-range="hour" on:click={() => selectRange('hour')}>60 Mins</button>
      <button type="button" class="btn time-btn" class:active={selectedTimeRange === 'day'} data-range="day" on:click={() => selectRange('day')}>24 Hours</button>
      <button type="button" class="btn time-btn" class:active={selectedTimeRange === 'week'} data-range="week" on:click={() => selectRange('week')}>7 Days</button>
      <button type="button" class="btn time-btn" class:active={selectedTimeRange === 'month'} data-range="month" on:click={() => selectRange('month')}>30 Days</button>
    </div>
  </div>
  <div class="chart-container panel-soft panel-border pad-md">
    <canvas id="timeSeriesChart" bind:this={timeSeriesCanvas}></canvas>
  </div>
</SectionBlock>
