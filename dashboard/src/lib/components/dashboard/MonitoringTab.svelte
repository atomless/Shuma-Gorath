<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
</script>

<section
  id="dashboard-panel-monitoring"
  class="dashboard-tab-panel"
  data-dashboard-tab-panel="monitoring"
  aria-labelledby="dashboard-tab-monitoring"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'false'}
  tabindex="-1"
>
  <TabStateMessage tab="monitoring" status={tabStatus} />

  <section class="section">
    <div class="panel panel-soft">
      <h2>Monitoring Overhaul In Progress</h2>
      <p class="text-muted">
        This tab is intentionally clear while Shuma rebuilds Monitoring as the operator decision
        surface for traffic mix, attacker effectiveness, human friction, and shadow-versus-enforced
        truth.
      </p>
      <p class="text-muted">
        The previous subsystem-by-subsystem monitoring surface now lives in
        <a href="#diagnostics">Diagnostics</a>.
      </p>
    </div>
  </section>
</section>
