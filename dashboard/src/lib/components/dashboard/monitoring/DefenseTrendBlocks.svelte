<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let trendRows = [];
</script>

<SectionBlock
  title="Defense Trends"
  description="Per-defense trigger and outcome trends grouped by source labels."
>
  {#if !loading && trendRows.length === 0}
    <p id="defense-trend-state-empty" class="text-muted">
      No defense trend rows are available for the current event window.
    </p>
  {:else}
    <div id="defense-trend-blocks" class="stats-cards stats-cards--compact">
      {#each trendRows as row}
        <article class="panel panel-border pad-md-b">
          <h3>{row.label}</h3>
          <ul class="metric-list">
            <li><strong>Triggers:</strong> {formatCompactNumber(row.triggerCount, '0')}</li>
            <li><strong>Pass:</strong> {formatCompactNumber(row.passCount, '0')}</li>
            <li><strong>Fail:</strong> {formatCompactNumber(row.failCount, '0')}</li>
            <li><strong>Escalate:</strong> {formatCompactNumber(row.escalationCount, '0')}</li>
            <li><strong>Ban Outcomes:</strong> {formatCompactNumber(row.banOutcomeCount, '0')}</li>
            <li>
              <strong>Sources:</strong>
              {#if row.sourceRows.length === 0}
                none
              {:else}
                {#each row.sourceRows as source, index}
                  {source.label}:{formatCompactNumber(source.count, '0')}{index < row.sourceRows.length - 1 ? ', ' : ''}
                {/each}
              {/if}
            </li>
          </ul>
        </article>
      {/each}
    </div>
  {/if}
</SectionBlock>
