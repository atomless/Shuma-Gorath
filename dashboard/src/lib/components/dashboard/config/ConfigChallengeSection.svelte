<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';
  import NumericInputRow from '../primitives/NumericInputRow.svelte';

  export let writable = false;
  export let notABotDirty = false;
  export let challengePuzzleDirty = false;
  export let notABotEnabled = true;
  export let challengePuzzleEnabled = true;
  export let notABotScorePassMinFloor = 1;
  export let notABotScorePassMin = 7;
  export let notABotScoreFailMaxCap = 6;
  export let notABotScoreFailMax = 3;
  export let notABotPassScoreValid = true;
  export let notABotFailScoreValid = true;
</script>

<ConfigPanel writable={writable} dirty={notABotDirty}>
  <ConfigPanelHeading title="Challenge: Not-a-Bot">
    <label class="toggle-switch" for="not-a-bot-enabled-toggle">
      <input type="checkbox" id="not-a-bot-enabled-toggle" aria-label="Enable not-a-bot challenge routing" bind:checked={notABotEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Not-a-Bot is the low-friction checkbox style challenge routed to when the botness signal is above zero but below the instant ban criteria, and below the thresholds where the visitor will be routed to the Maze or Tarpit. When Shuma Gorath is optimally configured, human users should rarely be asked to complete either of the challenges. You may click here to preview <a id="preview-not-a-bot-link" href="/challenge/not-a-bot-checkbox" target="_blank" rel="noopener noreferrer">Not-a-Bot</a>, but shadow mode must be enabled. Advanced controls (<abbr title="Number Used Once">Nonce</abbr> <abbr title="Time To Live">TTL</abbr>, marker <abbr title="Time To Live">TTL</abbr>, and attempt limits) are available in Advanced <abbr title="JavaScript Object Notation">JSON</abbr>.</p>
  <div class="admin-controls">
    <NumericInputRow id="not-a-bot-score-pass-min" label={`Pass Score (${notABotScorePassMinFloor}-10)`} min={notABotScorePassMinFloor} max="10" step="1" inputmode="numeric" ariaLabel="Not-a-Bot pass score threshold" ariaInvalid={notABotPassScoreValid ? 'false' : 'true'} bind:value={notABotScorePassMin} />
    <p class="text-muted">Any scores above Fail and below Pass will be shown a tougher challenge.</p>
    <NumericInputRow id="not-a-bot-score-fail-max" label={`Fail Score (0-${notABotScoreFailMaxCap})`} min="0" max={notABotScoreFailMaxCap} step="1" inputmode="numeric" ariaLabel="Not-a-Bot fail score threshold" ariaInvalid={notABotFailScoreValid ? 'false' : 'true'} bind:value={notABotScoreFailMax} />
    <p class="text-muted">Scores below Fail route to Maze (if enabled), otherwise Block (403). Confirmed attacks (replay, tamper, or attempt-window abuse) route to tarpit when available, otherwise a short ban.</p>
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={challengePuzzleDirty}>
  <ConfigPanelHeading title="Challenge: Puzzle">
    <label class="toggle-switch" for="challenge-puzzle-enabled-toggle">
      <input type="checkbox" id="challenge-puzzle-enabled-toggle" aria-label="Enable challenge puzzle routing" bind:checked={challengePuzzleEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">The Puzzle Challenge is shown to visitors failing to attain a pass score on the not-a-bot challenge, but who also scored above the auto-ban, hard-fail level. It is designed to be significantly more costly for bots, while remaining relatively simple for humans. Very few humans should ever have to solve it. The Puzzle has deterministic solve verification (correct/incorrect) with strict signed-seed expiry and replay/timing enforcement. Advanced controls, including transform count (which increases the challenge difficulty), seed <abbr title="Time To Live">TTL</abbr>, and attempt limits, can be configured using the Advanced <abbr title="JavaScript Object Notation">JSON</abbr> input below. Wrong answers route to Maze. Confirmed attacks (replay, tamper, or attempt-window abuse) route to tarpit when available, otherwise a short ban. You may click here to preview the <a id="preview-challenge-puzzle-link" href="/challenge/puzzle" target="_blank" rel="noopener noreferrer">Puzzle</a>, but shadow mode must be enabled.</p>
</ConfigPanel>
