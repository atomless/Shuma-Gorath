<script>
  import { formatCompactNumber } from '../../../domain/core/format.js';
  import SectionBlock from '../primitives/SectionBlock.svelte';

  export let loading = false;
  export let trendRows = [];
</script>

<SectionBlock
  {...$$restProps}
  title="Defense Breakdown"
  description="Per-defense trigger and outcome breakdown grouped by source labels."
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
            {#if row.hasOutcomeBreakdown}
              <li><strong>Pass:</strong> {formatCompactNumber(row.passCount, '0')}</li>
              <li><strong>Fail:</strong> {formatCompactNumber(row.failCount, '0')}</li>
              <li><strong>Escalate:</strong> {formatCompactNumber(row.escalationCount, '0')}</li>
            {:else}
              <li><strong>Outcome Breakdown:</strong> n/a</li>
            {/if}
            <li><strong>Ban Outcomes:</strong> {formatCompactNumber(row.banOutcomeCount, '0')}</li>
            <li>
              <strong>Modes:</strong>
              {#if row.modeRows.length === 0}
                none
              {:else}
                {#each row.modeRows as mode, index}
                  {mode.label}:{formatCompactNumber(mode.count, '0')}{index < row.modeRows.length - 1 ? ', ' : ''}
                {/each}
              {/if}
            </li>
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
