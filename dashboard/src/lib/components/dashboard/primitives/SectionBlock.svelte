<script>
  export let title = '';
  export let description = '';
  export let rootClass = 'section events';
</script>

<div class={rootClass}>
  <h2>{@html title}</h2>
  {#if description}
    <p class="section-desc text-muted">{@html description}</p>
  {/if}
  <slot />
</div>
