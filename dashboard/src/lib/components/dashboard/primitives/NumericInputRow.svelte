<script>
  export let id = '';
  export let label = '';
  export let labelClass = 'control-label';
  export let value = '';
  export let min = null;
  export let max = null;
  export let step = 1;
  export let inputmode = 'numeric';
  export let ariaLabel = '';
  export let disabled = false;
  export let rowClass = '';
  export let ariaDisabled = null;
  export let onInput = null;

  const handleInput = (event) => {
    if (typeof onInput === 'function') {
      onInput(event);
    }
  };
</script>

<div class={`input-row ${rowClass}`.trim()} aria-disabled={ariaDisabled}>
  <label class={labelClass} for={id}>{@html label}</label>
  <input
    class="input-field"
    type="number"
    {id}
    {min}
    {max}
    {step}
    {inputmode}
    aria-label={ariaLabel}
    bind:value
    {disabled}
    on:input={handleInput}
  >
</div>
