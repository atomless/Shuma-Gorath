<script>
  export let label = '';
  export let dayId = '';
  export let hourId = '';
  export let minuteId = '';
  export let days = 0;
  export let hours = 0;
  export let minutes = 0;
  export let ariaInvalid = undefined;
  export let onDaysInput = null;
  export let onHoursInput = null;
  export let onMinutesInput = null;

  const handleDaysInput = (event) => {
    if (typeof onDaysInput === 'function') {
      onDaysInput(event);
    }
  };

  const handleHoursInput = (event) => {
    if (typeof onHoursInput === 'function') {
      onHoursInput(event);
    }
  };

  const handleMinutesInput = (event) => {
    if (typeof onMinutesInput === 'function') {
      onMinutesInput(event);
    }
  };
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
        on:input={handleDaysInput}
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
        on:input={handleHoursInput}
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
        on:input={handleMinutesInput}
      />
      <span class="input-unit">mins</span>
    </label>
  </div>
</div>
