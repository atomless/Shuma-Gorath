<script>
  export let writable = true;
  export let dirty = false;
  export let variant = 'edit';
  export let className = '';

  $: isReadOnly = !writable && variant !== 'export';
  $: variantClass = variant === 'export' ? 'config-export-pane' : 'config-edit-pane';
  $: panelClass = `control-group panel-soft pad-md ${variantClass} ${className}`.trim();
</script>

<div
  class={panelClass}
  class:config-panel--read-only={isReadOnly}
  class:config-edit-pane--dirty={variant !== 'export' && dirty}
  aria-disabled={isReadOnly ? 'true' : 'false'}
>
  <fieldset class="config-panel__fieldset" disabled={!writable && variant !== 'export'}>
    <slot />
  </fieldset>
</div>
