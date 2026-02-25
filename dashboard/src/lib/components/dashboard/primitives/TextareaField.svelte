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
  export let showLineNumbers = false;

  $: textareaClass = `input-field textarea-field__input ${monospace ? 'input-field--mono' : ''}`.trim();
  $: lineCount = Math.max(1, String(value || '').split('\n').length);
  $: lineNumbers = Array.from({ length: lineCount }, (_value, index) => index + 1);

  let lineNumberGutter = null;

  const handleTextareaScroll = (event) => {
    if (!showLineNumbers || !lineNumberGutter) return;
    const target = event && event.currentTarget ? event.currentTarget : null;
    if (!target) return;
    lineNumberGutter.scrollTop = target.scrollTop;
  };
</script>

<div class="textarea-field">
  <label class={labelClass} for={id}>{@html label}</label>
  {#if showLineNumbers}
    <div class="textarea-field__with-lines">
      <pre class="textarea-field__line-numbers" aria-hidden="true" bind:this={lineNumberGutter}>{lineNumbers.join('\n')}</pre>
      <textarea
        class={textareaClass}
        {id}
        {rows}
        aria-label={ariaLabel}
        spellcheck={spellcheck}
        bind:value
        {disabled}
        aria-invalid={ariaInvalid}
        on:scroll={handleTextareaScroll}
      ></textarea>
    </div>
  {:else}
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
  {/if}
</div>
