<script>
  import HalfDoughnutChart from '../primitives/HalfDoughnutChart.svelte';
  export let selectedTimeRange = 'hour';
  export let onSelectTimeRange = null;
  export let eventTypesCanvas = null;
  export let eventTypesReadout = null;
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
    <h2>Event Types (Enforced, 24h)</h2>
    <HalfDoughnutChart
      canvasId="eventTypesChart"
      ariaLabel="Event types chart"
      bind:canvas={eventTypesCanvas}
      readout={eventTypesReadout}
    />
  </div>
  <div class="chart-container panel-soft panel-border pad-md">
    <h2>Top 10 <abbr title="Internet Protocol">IP</abbr>s by Enforced Events (24h)</h2>
    <canvas id="topIpsChart" bind:this={topIpsCanvas}></canvas>
  </div>
</div>

<h2>Events Over Time</h2>
<p class="section-desc text-muted">Enforced events plotted over various time windows</p>
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
