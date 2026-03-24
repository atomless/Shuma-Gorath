<script>
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;

  const monitoringSections = Object.freeze([
    {
      id: 'current-status',
      title: 'Current Status',
      description:
        "Loop verdict, budget state, latest controller action, and any hard blockers for the live operator-selected stance will surface here."
    },
    {
      id: 'recent-loop-progress',
      title: 'Recent Loop Progress',
      description:
        'Bounded multi-loop benchmark progress and recent controller action history will land here once the loop-progress projection is wired in.'
    },
    {
      id: 'outcome-frontier',
      title: 'Outcome Frontier',
      description:
        'This section will compare suspicious non-human cost reduction against measured human-friction impact instead of collapsing the loop into one opaque score.'
    },
    {
      id: 'change-judgment',
      title: 'What The Loop Decided',
      description:
        'Controller recommendations, apply or rollback outcomes, and explicit refusal reasons will live here as a separate surface from observed traffic truth.'
    },
    {
      id: 'pressure-sits',
      title: 'Where The Pressure Sits',
      description:
        'Category-aware breakdowns will show where the remaining non-human problem is concentrated once the taxonomy and benchmark surfaces are projected.'
    },
    {
      id: 'trust-and-blockers',
      title: 'Trust And Blockers',
      description:
        'Evidence quality, tuning eligibility, protected-evidence readiness, and no-harm blockers will explain how trustworthy the current loop conclusion is.'
    }
  ]);
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
      <h2>Closed-Loop Accountability</h2>
      <p class="text-muted">
        Monitoring now frames how Shuma&apos;s closed feedback loop is performing against the current
        operator-selected live stance.
      </p>
      <p class="text-muted">
        Deep subsystem telemetry, recent raw traffic, and contributor-style freshness details stay
        in <a href="#diagnostics">Diagnostics</a>.
      </p>
    </div>
  </section>

  {#each monitoringSections as section (section.id)}
    <section class="section" data-monitoring-section={section.id}>
      <div class="panel panel-soft">
        <h2>{section.title}</h2>
        <p class="text-muted">{section.description}</p>
      </div>
    </section>
  {/each}
</section>
