<script>
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let rows = [];
</script>

<SectionBlock
  {...$$restProps}
  title="Defense Breakdown"
  description="Per-defense furniture snapshot with recent trigger and handling context."
>
  {#if !loading && rows.length === 0}
    <p id="defense-trend-state-empty" class="text-muted">
      No defense breakdown rows are available for the current diagnostics snapshot.
    </p>
  {:else}
    <div id="defense-trend-blocks" class="stats-cards stats-cards--compact">
      {#each rows as row}
        <article class="panel panel-border pad-md-b">
          <h3>{row.label}</h3>
          <ul class="metric-list">
            {#each row.factRows as fact}
              <li><strong>{fact.label}:</strong> {fact.value}</li>
            {/each}
          </ul>
        </article>
      {/each}
    </div>
  {/if}
</SectionBlock>
