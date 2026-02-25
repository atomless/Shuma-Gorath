<script>
  export let id = '';
  export let label = '';
  export let rows = 3;
  export let value = '';
  export let ariaLabel = '';
  export let spellcheck = false;
  export let disabled = false;
  export let labelClass = 'control-label';
  export let monospace = false;
  export let ariaInvalid = undefined;

  $: textareaClass = `input-field textarea-field__input ${monospace ? 'input-field--mono' : ''}`.trim();
</script>

<div class="textarea-field">
  <label class={labelClass} for={id}>{@html label}</label>
  <textarea
    class={textareaClass}
    {id}
    {rows}
    aria-label={ariaLabel}
    spellcheck={spellcheck}
    bind:value
    {disabled}
    aria-invalid={ariaInvalid}
  ></textarea>
</div>
