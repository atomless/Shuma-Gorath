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
  export let onFieldChange = null;

  const handleFieldChange = (field, event) => {
    if (typeof onFieldChange === 'function') {
      onFieldChange(field, event?.currentTarget?.value ?? '');
    }
  };
</script>

<ConfigPanel writable={writable} dirty={durationsDirty}>
  <h3>Ban Durations</h3>
  <p class="control-desc text-muted">Set ban length in days, hours, and minutes per trigger type. Longer bans increase deterrence but slow recovery from false positives.</p>
  <div class="duration-grid">
    <DurationInputRow label="Honeypot Hit" dayId="dur-honeypot-days" hourId="dur-honeypot-hours" minuteId="dur-honeypot-minutes" ariaInvalid={durHoneypotValid ? 'false' : 'true'} days={durHoneypotDays} hours={durHoneypotHours} minutes={durHoneypotMinutes} onDaysInput={(event) => handleFieldChange('durHoneypotDays', event)} onHoursInput={(event) => handleFieldChange('durHoneypotHours', event)} onMinutesInput={(event) => handleFieldChange('durHoneypotMinutes', event)} />
    <DurationInputRow label="IP Range Honeypot Policy" dayId="dur-ip-range-honeypot-days" hourId="dur-ip-range-honeypot-hours" minuteId="dur-ip-range-honeypot-minutes" ariaInvalid={durIpRangeHoneypotValid ? 'false' : 'true'} days={durIpRangeHoneypotDays} hours={durIpRangeHoneypotHours} minutes={durIpRangeHoneypotMinutes} onDaysInput={(event) => handleFieldChange('durIpRangeHoneypotDays', event)} onHoursInput={(event) => handleFieldChange('durIpRangeHoneypotHours', event)} onMinutesInput={(event) => handleFieldChange('durIpRangeHoneypotMinutes', event)} />
    <DurationInputRow label="Maze Threshold Exceeded" dayId="dur-maze-crawler-days" hourId="dur-maze-crawler-hours" minuteId="dur-maze-crawler-minutes" ariaInvalid={durMazeCrawlerValid ? 'false' : 'true'} days={durMazeCrawlerDays} hours={durMazeCrawlerHours} minutes={durMazeCrawlerMinutes} onDaysInput={(event) => handleFieldChange('durMazeCrawlerDays', event)} onHoursInput={(event) => handleFieldChange('durMazeCrawlerHours', event)} onMinutesInput={(event) => handleFieldChange('durMazeCrawlerMinutes', event)} />
    <DurationInputRow label="Rate Limit Exceeded" dayId="dur-rate-limit-days" hourId="dur-rate-limit-hours" minuteId="dur-rate-limit-minutes" ariaInvalid={durRateLimitValid ? 'false' : 'true'} days={durRateLimitDays} hours={durRateLimitHours} minutes={durRateLimitMinutes} onDaysInput={(event) => handleFieldChange('durRateLimitDays', event)} onHoursInput={(event) => handleFieldChange('durRateLimitHours', event)} onMinutesInput={(event) => handleFieldChange('durRateLimitMinutes', event)} />
    <DurationInputRow label='<abbr title="Chrome DevTools Protocol">CDP</abbr> Automation Detected' dayId="dur-cdp-days" hourId="dur-cdp-hours" minuteId="dur-cdp-minutes" ariaInvalid={durCdpValid ? 'false' : 'true'} days={durCdpDays} hours={durCdpHours} minutes={durCdpMinutes} onDaysInput={(event) => handleFieldChange('durCdpDays', event)} onHoursInput={(event) => handleFieldChange('durCdpHours', event)} onMinutesInput={(event) => handleFieldChange('durCdpMinutes', event)} />
    <DurationInputRow label="Edge Fingerprint Automation" dayId="dur-edge-fingerprint-days" hourId="dur-edge-fingerprint-hours" minuteId="dur-edge-fingerprint-minutes" ariaInvalid={durEdgeFingerprintValid ? 'false' : 'true'} days={durEdgeFingerprintDays} hours={durEdgeFingerprintHours} minutes={durEdgeFingerprintMinutes} onDaysInput={(event) => handleFieldChange('durEdgeFingerprintDays', event)} onHoursInput={(event) => handleFieldChange('durEdgeFingerprintHours', event)} onMinutesInput={(event) => handleFieldChange('durEdgeFingerprintMinutes', event)} />
    <DurationInputRow label="Tarpit Persistence" dayId="dur-tarpit-persistence-days" hourId="dur-tarpit-persistence-hours" minuteId="dur-tarpit-persistence-minutes" ariaInvalid={durTarpitPersistenceValid ? 'false' : 'true'} days={durTarpitPersistenceDays} hours={durTarpitPersistenceHours} minutes={durTarpitPersistenceMinutes} onDaysInput={(event) => handleFieldChange('durTarpitPersistenceDays', event)} onHoursInput={(event) => handleFieldChange('durTarpitPersistenceHours', event)} onMinutesInput={(event) => handleFieldChange('durTarpitPersistenceMinutes', event)} />
    <DurationInputRow label="Not-a-Bot Abuse" dayId="dur-not-a-bot-abuse-days" hourId="dur-not-a-bot-abuse-hours" minuteId="dur-not-a-bot-abuse-minutes" ariaInvalid={durNotABotAbuseValid ? 'false' : 'true'} days={durNotABotAbuseDays} hours={durNotABotAbuseHours} minutes={durNotABotAbuseMinutes} onDaysInput={(event) => handleFieldChange('durNotABotAbuseDays', event)} onHoursInput={(event) => handleFieldChange('durNotABotAbuseHours', event)} onMinutesInput={(event) => handleFieldChange('durNotABotAbuseMinutes', event)} />
    <DurationInputRow label="Challenge Puzzle Abuse" dayId="dur-challenge-puzzle-abuse-days" hourId="dur-challenge-puzzle-abuse-hours" minuteId="dur-challenge-puzzle-abuse-minutes" ariaInvalid={durChallengePuzzleAbuseValid ? 'false' : 'true'} days={durChallengePuzzleAbuseDays} hours={durChallengePuzzleAbuseHours} minutes={durChallengePuzzleAbuseMinutes} onDaysInput={(event) => handleFieldChange('durChallengePuzzleAbuseDays', event)} onHoursInput={(event) => handleFieldChange('durChallengePuzzleAbuseHours', event)} onMinutesInput={(event) => handleFieldChange('durChallengePuzzleAbuseMinutes', event)} />
    <DurationInputRow label="Admin Manual Ban Default" dayId="dur-admin-days" hourId="dur-admin-hours" minuteId="dur-admin-minutes" ariaInvalid={durAdminValid ? 'false' : 'true'} days={durAdminDays} hours={durAdminHours} minutes={durAdminMinutes} onDaysInput={(event) => handleFieldChange('durAdminDays', event)} onHoursInput={(event) => handleFieldChange('durAdminHours', event)} onMinutesInput={(event) => handleFieldChange('durAdminMinutes', event)} />
  </div>
</ConfigPanel>
