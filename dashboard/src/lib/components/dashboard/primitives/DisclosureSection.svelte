<script>
  export let title = '';
  export let description = '';
  export let rootClass = 'section panel panel-border';
  export let open = false;
</script>

<details class={`${rootClass} disclosure-section`} {open} {...$$restProps}>
  <summary class="disclosure-summary">
    <div class="disclosure-summary-copy">
      <h2>{title}</h2>
      {#if description}
        <p class="section-desc text-muted">{description}</p>
      {/if}
    </div>
  </summary>

  <div class="disclosure-body">
    <slot />
  </div>
</details>
