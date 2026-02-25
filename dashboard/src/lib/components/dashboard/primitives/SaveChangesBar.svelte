<script>
  export let containerId = '';
  export let isHidden = true;
  export let summaryId = '';
  export let summaryText = '';
  export let summaryClass = 'text-muted';
  export let invalidId = '';
  export let invalidText = '';
  export let buttonId = '';
  export let buttonLabel = 'Save all changes';
  export let buttonDisabled = false;
  export let onSave = null;

  const handleSave = (event) => {
    if (typeof onSave === 'function') {
      void onSave(event);
    }
  };
</script>

<div
  id={containerId}
  class="config-save-bar panel panel-border"
  class:hidden={isHidden}
>
  <div class="config-save-bar__meta">
    <span id={summaryId} class={summaryClass}>{summaryText}</span>
    {#if invalidText}
      <span id={invalidId} class="config-save-bar__warning">{invalidText}</span>
    {/if}
    <button id={buttonId} class="btn btn-submit" disabled={buttonDisabled} on:click={handleSave}>
      {buttonLabel}
    </button>
  </div>
</div>
