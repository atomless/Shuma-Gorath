<script>
  export let title = '';
  export let titleId = '';
  export let titleLevel = 'h3';
  export let value = '-';
  export let valueId = '';
  export let loading = false;
  export let loadingValue = '...';
  export let small = false;
</script>

<div class="card panel panel-border pad-md-b">
  {#if titleLevel === 'h4'}
    <h4 class="caps-label" id={titleId || undefined}>{@html title}</h4>
  {:else}
    <h3 class="caps-label" id={titleId || undefined}>{@html title}</h3>
  {/if}
  <div class={`stat-value ${small ? 'stat-value--small' : ''}`.trim()} id={valueId || undefined}>
    {loading ? loadingValue : value}
  </div>
  <slot />
</div>
