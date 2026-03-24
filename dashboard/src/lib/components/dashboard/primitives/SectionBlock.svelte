<script>
  export let title = '';
  export let description = '';
  export let rootClass = 'section';
</script>

<section class={rootClass} {...$$restProps}>
  {#if title}
    <h2>{@html title}</h2>
  {/if}
  {#if description}
    <p class="section-desc text-muted">{@html description}</p>
  {/if}
  <slot />
</section>
