<script>
  import ConfigPanel from '../primitives/ConfigPanel.svelte';
  import ConfigPanelHeading from '../primitives/ConfigPanelHeading.svelte';
  import TextareaField from '../primitives/TextareaField.svelte';

  export let writable = false;
  export let geoScoringDirty = false;
  export let geoRoutingDirty = false;
  export let geoScoringEnabled = true;
  export let geoRoutingEnabled = true;
  export let geoRiskList = '';
  export let geoAllowList = '';
  export let geoChallengeList = '';
  export let geoMazeList = '';
  export let geoBlockList = '';
  export let geoRiskListValid = true;
  export let geoAllowListValid = true;
  export let geoChallengeListValid = true;
  export let geoMazeListValid = true;
  export let geoBlockListValid = true;
</script>

<ConfigPanel writable={writable} dirty={geoScoringDirty}>
  <ConfigPanelHeading title='<abbr title="Geolocation">GEO</abbr> Risk Based Scoring'>
    <label class="toggle-switch" for="geo-scoring-toggle">
      <input type="checkbox" id="geo-scoring-toggle" bind:checked={geoScoringEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will receive added botness risk to contribute to the combined score.</p>
  <div class="admin-controls geo-controls">
    <TextareaField id="geo-risk-list" label="Scoring Countries" rows="1" ariaLabel="Geolocation scoring countries" spellcheck={false} ariaInvalid={geoRiskListValid ? 'false' : 'true'} bind:value={geoRiskList} />
  </div>
</ConfigPanel>

<ConfigPanel writable={writable} dirty={geoRoutingDirty}>
  <ConfigPanelHeading title='<abbr title="Geolocation">GEO</abbr> Risk Based Routing'>
    <label class="toggle-switch" for="geo-routing-toggle">
      <input type="checkbox" id="geo-routing-toggle" bind:checked={geoRoutingEnabled}>
      <span class="toggle-slider"></span>
    </label>
  </ConfigPanelHeading>
  <p class="control-desc text-muted">Use <a href="https://www.iban.com/country-codes">2-letter country codes</a> to specify countries from where requests will be automatically routed. Precedence is Block &gt; Maze &gt; Challenge &gt; Allow.</p>
  <div class="admin-controls geo-controls">
    <TextareaField id="geo-block-list" label="Block Countries" rows="1" ariaLabel="Geolocation block countries" spellcheck={false} ariaInvalid={geoBlockListValid ? 'false' : 'true'} bind:value={geoBlockList} />
    <TextareaField id="geo-maze-list" label="Maze Countries" rows="1" ariaLabel="Geolocation maze countries" spellcheck={false} ariaInvalid={geoMazeListValid ? 'false' : 'true'} bind:value={geoMazeList} />
    <TextareaField id="geo-challenge-list" label="Challenge Countries" rows="1" ariaLabel="Geolocation challenge countries" spellcheck={false} ariaInvalid={geoChallengeListValid ? 'false' : 'true'} bind:value={geoChallengeList} />
    <TextareaField id="geo-allow-list" label="Allow Countries" rows="1" ariaLabel="Geolocation allow countries" spellcheck={false} ariaInvalid={geoAllowListValid ? 'false' : 'true'} bind:value={geoAllowList} />
  </div>
</ConfigPanel>
