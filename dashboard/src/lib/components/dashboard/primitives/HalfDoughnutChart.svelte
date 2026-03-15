<script>
  export let canvas = null;
  export let canvasId = '';
  export let ariaLabel = '';
  export let shellClass = '';
  export let readout = null;

  $: isActive = readout?.active === true;
  $: labelText = String(readout?.label || '');
  $: valueText = isActive ? String(readout?.value || '') : '';
  $: shellClasses = ['chart-canvas-shell', 'chart-canvas-shell--half-doughnut', shellClass]
    .filter(Boolean)
    .join(' ');
</script>

<div class={shellClasses}>
  <canvas id={canvasId} bind:this={canvas} aria-label={ariaLabel}></canvas>
  {#if isActive}
    <div class="chart-doughnut-readout" data-active="true">
      <p class="caps-label chart-doughnut-readout__label">{labelText}</p>
      <p class="stat-value chart-doughnut-readout__value">{valueText}</p>
    </div>
  {/if}
</div>
