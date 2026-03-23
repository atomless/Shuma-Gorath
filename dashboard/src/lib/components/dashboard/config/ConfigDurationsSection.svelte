<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import DurationInputRow from '../primitives/DurationInputRow.svelte';

  export let writable = false;
  export let durationsDirty = false;
  export let durHoneypotDays = 1;
  export let durHoneypotHours = 0;
  export let durHoneypotMinutes = 0;
  export let durIpRangeHoneypotDays = 1;
  export let durIpRangeHoneypotHours = 0;
  export let durIpRangeHoneypotMinutes = 0;
  export let durMazeCrawlerDays = 1;
  export let durMazeCrawlerHours = 0;
  export let durMazeCrawlerMinutes = 0;
  export let durRateLimitDays = 0;
  export let durRateLimitHours = 1;
  export let durRateLimitMinutes = 0;
  export let durCdpDays = 0;
  export let durCdpHours = 12;
  export let durCdpMinutes = 0;
  export let durEdgeFingerprintDays = 0;
  export let durEdgeFingerprintHours = 12;
  export let durEdgeFingerprintMinutes = 0;
  export let durTarpitPersistenceDays = 0;
  export let durTarpitPersistenceHours = 0;
  export let durTarpitPersistenceMinutes = 10;
  export let durNotABotAbuseDays = 0;
  export let durNotABotAbuseHours = 0;
  export let durNotABotAbuseMinutes = 10;
  export let durChallengePuzzleAbuseDays = 0;
  export let durChallengePuzzleAbuseHours = 0;
  export let durChallengePuzzleAbuseMinutes = 10;
  export let durAdminDays = 0;
  export let durAdminHours = 6;
  export let durAdminMinutes = 0;
  export let durHoneypotValid = true;
  export let durIpRangeHoneypotValid = true;
  export let durMazeCrawlerValid = true;
  export let durRateLimitValid = true;
  export let durCdpValid = true;
  export let durEdgeFingerprintValid = true;
  export let durTarpitPersistenceValid = true;
  export let durNotABotAbuseValid = true;
  export let durChallengePuzzleAbuseValid = true;
  export let durAdminValid = true;
</script>

<ConfigPanel writable={writable} dirty={durationsDirty}>
  <h3>Ban Durations</h3>
  <p class="control-desc text-muted">Set ban length in days, hours, and minutes per trigger type. Longer bans increase deterrence but slow recovery from false positives.</p>
  <div class="duration-grid">
    <DurationInputRow label="Honeypot Hit" dayId="dur-honeypot-days" hourId="dur-honeypot-hours" minuteId="dur-honeypot-minutes" ariaInvalid={durHoneypotValid ? 'false' : 'true'} bind:days={durHoneypotDays} bind:hours={durHoneypotHours} bind:minutes={durHoneypotMinutes} />
    <DurationInputRow label="IP Range Honeypot Policy" dayId="dur-ip-range-honeypot-days" hourId="dur-ip-range-honeypot-hours" minuteId="dur-ip-range-honeypot-minutes" ariaInvalid={durIpRangeHoneypotValid ? 'false' : 'true'} bind:days={durIpRangeHoneypotDays} bind:hours={durIpRangeHoneypotHours} bind:minutes={durIpRangeHoneypotMinutes} />
    <DurationInputRow label="Maze Threshold Exceeded" dayId="dur-maze-crawler-days" hourId="dur-maze-crawler-hours" minuteId="dur-maze-crawler-minutes" ariaInvalid={durMazeCrawlerValid ? 'false' : 'true'} bind:days={durMazeCrawlerDays} bind:hours={durMazeCrawlerHours} bind:minutes={durMazeCrawlerMinutes} />
    <DurationInputRow label="Rate Limit Exceeded" dayId="dur-rate-limit-days" hourId="dur-rate-limit-hours" minuteId="dur-rate-limit-minutes" ariaInvalid={durRateLimitValid ? 'false' : 'true'} bind:days={durRateLimitDays} bind:hours={durRateLimitHours} bind:minutes={durRateLimitMinutes} />
    <DurationInputRow label='<abbr title="Chrome DevTools Protocol">CDP</abbr> Automation Detected' dayId="dur-cdp-days" hourId="dur-cdp-hours" minuteId="dur-cdp-minutes" ariaInvalid={durCdpValid ? 'false' : 'true'} bind:days={durCdpDays} bind:hours={durCdpHours} bind:minutes={durCdpMinutes} />
    <DurationInputRow label="Edge Fingerprint Automation" dayId="dur-edge-fingerprint-days" hourId="dur-edge-fingerprint-hours" minuteId="dur-edge-fingerprint-minutes" ariaInvalid={durEdgeFingerprintValid ? 'false' : 'true'} bind:days={durEdgeFingerprintDays} bind:hours={durEdgeFingerprintHours} bind:minutes={durEdgeFingerprintMinutes} />
    <DurationInputRow label="Tarpit Persistence" dayId="dur-tarpit-persistence-days" hourId="dur-tarpit-persistence-hours" minuteId="dur-tarpit-persistence-minutes" ariaInvalid={durTarpitPersistenceValid ? 'false' : 'true'} bind:days={durTarpitPersistenceDays} bind:hours={durTarpitPersistenceHours} bind:minutes={durTarpitPersistenceMinutes} />
    <DurationInputRow label="Not-a-Bot Abuse" dayId="dur-not-a-bot-abuse-days" hourId="dur-not-a-bot-abuse-hours" minuteId="dur-not-a-bot-abuse-minutes" ariaInvalid={durNotABotAbuseValid ? 'false' : 'true'} bind:days={durNotABotAbuseDays} bind:hours={durNotABotAbuseHours} bind:minutes={durNotABotAbuseMinutes} />
    <DurationInputRow label="Challenge Puzzle Abuse" dayId="dur-challenge-puzzle-abuse-days" hourId="dur-challenge-puzzle-abuse-hours" minuteId="dur-challenge-puzzle-abuse-minutes" ariaInvalid={durChallengePuzzleAbuseValid ? 'false' : 'true'} bind:days={durChallengePuzzleAbuseDays} bind:hours={durChallengePuzzleAbuseHours} bind:minutes={durChallengePuzzleAbuseMinutes} />
    <DurationInputRow label="Admin Manual Ban Default" dayId="dur-admin-days" hourId="dur-admin-hours" minuteId="dur-admin-minutes" ariaInvalid={durAdminValid ? 'false' : 'true'} bind:days={durAdminDays} bind:hours={durAdminHours} bind:minutes={durAdminMinutes} />
  </div>
</ConfigPanel>
