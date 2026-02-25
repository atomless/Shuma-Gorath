<script>
  export let label = '';
  export let dayId = '';
  export let hourId = '';
  export let minuteId = '';
  export let days = 0;
  export let hours = 0;
  export let minutes = 0;
  export let ariaInvalid = undefined;
</script>

<div class="duration-row">
  <label class="control-label" for={dayId}>{@html label}</label>
  <div class="duration-inputs">
    <label class="duration-input" for={dayId}>
      <input
        id={dayId}
        class="input-field"
        type="number"
        min="0"
        max="365"
        step="1"
        inputmode="numeric"
        bind:value={days}
        aria-invalid={ariaInvalid}
      />
      <span class="input-unit">days</span>
    </label>
    <label class="duration-input" for={hourId}>
      <input
        id={hourId}
        class="input-field"
        type="number"
        min="0"
        max="23"
        step="1"
        inputmode="numeric"
        bind:value={hours}
        aria-invalid={ariaInvalid}
      />
      <span class="input-unit">hrs</span>
    </label>
    <label class="duration-input" for={minuteId}>
      <input
        id={minuteId}
        class="input-field"
        type="number"
        min="0"
        max="59"
        step="1"
        inputmode="numeric"
        bind:value={minutes}
        aria-invalid={ariaInvalid}
      />
      <span class="input-unit">mins</span>
    </label>
  </div>
</div>
