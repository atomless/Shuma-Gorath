<script>
  export let title = '';
</script>

<div class={`card panel panel-border pad-md-b`}>
  <h3 class="caps-label">{@html title}</h3>
  <slot />
</div>
