<script>
  export let id = '';
  export let label = '';
  export let labelClass = 'control-label';
  export let checked = false;
  export let ariaLabel = '';
  export let hint = '';
  export let disabled = false;
  export let rowClass = '';
  export let onChange = null;

  const handleChange = (event) => {
    if (typeof onChange === 'function') {
      onChange(event);
    }
  };
</script>

<div class={`toggle-row ${rowClass}`.trim()}>
  <label class={labelClass} for={id}>{@html label}</label>
  <label class="toggle-switch" for={id}>
    <input
      type="checkbox"
      {id}
      aria-label={ariaLabel}
      bind:checked
      {disabled}
      on:change={handleChange}
    >
    <span class="toggle-slider"></span>
  </label>
  {#if hint}
    <span class="toggle-hint">{@html hint}</span>
  {/if}
</div>
