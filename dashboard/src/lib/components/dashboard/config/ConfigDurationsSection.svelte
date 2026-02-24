<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import DurationInputRow from '../primitives/DurationInputRow.svelte';

  export let writable = false;
  export let durationsDirty = false;
  export let durHoneypotDays = 1;
  export let durHoneypotHours = 0;
  export let durHoneypotMinutes = 0;
  export let durRateLimitDays = 0;
  export let durRateLimitHours = 1;
  export let durRateLimitMinutes = 0;
  export let durBrowserDays = 0;
  export let durBrowserHours = 6;
  export let durBrowserMinutes = 0;
  export let durCdpDays = 0;
  export let durCdpHours = 12;
  export let durCdpMinutes = 0;
  export let durAdminDays = 0;
  export let durAdminHours = 6;
  export let durAdminMinutes = 0;
</script>

<ConfigPanel writable={writable} dirty={durationsDirty}>
  <h3>Ban Durations</h3>
  <p class="control-desc text-muted">Set ban length in days, hours, and minutes per trigger type. Longer bans increase deterrence but slow recovery from false positives.</p>
  <div class="duration-grid">
    <DurationInputRow label="Maze Threshold Exceeded" dayId="dur-honeypot-days" hourId="dur-honeypot-hours" minuteId="dur-honeypot-minutes" bind:days={durHoneypotDays} bind:hours={durHoneypotHours} bind:minutes={durHoneypotMinutes} />
    <DurationInputRow label="Rate Limit Exceeded" dayId="dur-rate-limit-days" hourId="dur-rate-limit-hours" minuteId="dur-rate-limit-minutes" bind:days={durRateLimitDays} bind:hours={durRateLimitHours} bind:minutes={durRateLimitMinutes} />
    <DurationInputRow label="Browser Automation Detected" dayId="dur-browser-days" hourId="dur-browser-hours" minuteId="dur-browser-minutes" bind:days={durBrowserDays} bind:hours={durBrowserHours} bind:minutes={durBrowserMinutes} />
    <DurationInputRow label='<abbr title="Chrome DevTools Protocol">CDP</abbr> Automation Detected' dayId="dur-cdp-days" hourId="dur-cdp-hours" minuteId="dur-cdp-minutes" bind:days={durCdpDays} bind:hours={durCdpHours} bind:minutes={durCdpMinutes} />
    <DurationInputRow label="Admin Manual Ban Default" dayId="dur-admin-days" hourId="dur-admin-hours" minuteId="dur-admin-minutes" bind:days={durAdminDays} bind:hours={durAdminHours} bind:minutes={durAdminMinutes} />
  </div>
</ConfigPanel>
