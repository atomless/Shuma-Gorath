<script>
  import SectionBlock from '../primitives/SectionBlock.svelte';
  import TableEmptyRow from '../primitives/TableEmptyRow.svelte';
  import TableWrapper from '../primitives/TableWrapper.svelte';

  export let lines = [];
  export let maxLines = 200;
  export let wrapped = true;
</script>

{#if wrapped}
  <SectionBlock
    title="Raw Telemetry Feed"
    description="Rolling raw persisted external-traffic event rows from monitoring ingestion (newest first)."
  >
    <p id="monitoring-raw-feed-meta" class="control-desc text-muted">
      Retains the most recent {maxLines} lines. Each new line is prepended; overflow drops the oldest line.
    </p>
    <TableWrapper>
      <table id="monitoring-raw-feed" class="panel panel-border">
        <thead>
          <tr>
            <th class="caps-label">#</th>
            <th class="caps-label">Raw Line</th>
          </tr>
        </thead>
        <tbody>
          {#if lines.length === 0}
            <TableEmptyRow colspan={2}>No telemetry lines recorded yet.</TableEmptyRow>
          {:else}
            {#each lines as entry, index (entry.key)}
              <tr>
                <td>{index + 1}</td>
                <td><code>{entry.line}</code></td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </TableWrapper>
  </SectionBlock>
{/if}

{#if !wrapped}
  <p id="monitoring-raw-feed-meta" class="control-desc text-muted">
    Retains the most recent {maxLines} lines. Each new line is prepended; overflow drops the oldest line.
  </p>
  <TableWrapper>
    <table id="monitoring-raw-feed" class="panel panel-border">
      <thead>
        <tr>
          <th class="caps-label">#</th>
          <th class="caps-label">Raw Line</th>
        </tr>
      </thead>
      <tbody>
        {#if lines.length === 0}
          <TableEmptyRow colspan={2}>No telemetry lines recorded yet.</TableEmptyRow>
        {:else}
          {#each lines as entry, index (entry.key)}
            <tr>
              <td>{index + 1}</td>
              <td><code>{entry.line}</code></td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </TableWrapper>
{/if}
