<script>
  export let writable = true;
  export let dirty = false;
  export let variant = 'edit';
  export let className = '';

  $: variantClass = variant === 'export' ? 'config-export-pane' : 'config-edit-pane';
  $: panelClass = `control-group panel-soft pad-md ${variantClass} ${className}`.trim();
</script>

<div
  class={panelClass}
  class:hidden={!writable}
  class:config-edit-pane--dirty={variant !== 'export' && dirty}
>
  <slot />
</div>
