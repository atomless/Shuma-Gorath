<script>
  import { formatCompactNumber } from '../../domain/core/format.js';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import { deriveLatestScraplingEvidenceFromSummaries } from './monitoring-view-model.js';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import MetricStatCard from './primitives/MetricStatCard.svelte';

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
  export let operatorSnapshot = null;
  export let benchmarkResults = null;
  export let oversightHistory = null;
  export let oversightAgentStatus = null;

  const gameLoopSections = Object.freeze([
    {
      id: 'current-status',
      title: '',
      description: ''
    },
    {
      id: 'recent-loop-progress',
      title: 'Recent Loop Progress',
      description:
        'Bounded multi-loop benchmark progress and recent controller action history from the oversight decision ledger.'
    },
    {
      id: 'outcome-frontier',
      title: 'Outcome Frontier',
      description:
        'Suspicious non-human cost reduction and measured human-friction impact, kept separate instead of collapsed into one opaque score.'
    },
    {
      id: 'change-judgment',
      title: 'What The Loop Decided',
      description:
        'The machine-first benchmark decision, candidate action families, and latest oversight recommendation or apply outcome.'
    },
    {
      id: 'pressure-sits',
      title: 'Where The Pressure Sits',
      description:
        'A bounded preview of the benchmark families still carrying pressure, plus nearby recent config-change context from the operator snapshot.'
    },
    {
      id: 'trust-and-blockers',
      title: 'Trust And Blockers',
      description:
        'Evidence readiness, protected replay status, coverage state, and explicit blockers that explain how trustworthy the current conclusion is.'
    }
  ]);

  const titleAcronyms = Object.freeze({
    ai: 'AI',
    api: 'API',
    cdp: 'CDP',
    http: 'HTTP',
    id: 'ID',
    ip: 'IP',
    llm: 'LLM',
    pow: 'PoW',
    sim: 'Sim',
    ui: 'UI'
  });

  const zeroTargetSuppressionMetricIds = new Set([
    'suspicious_forwarded_request_rate',
    'suspicious_forwarded_byte_rate',
    'suspicious_forwarded_latency_share'
  ]);

  const zeroTargetSuppressionLabels = Object.freeze({
    suspicious_forwarded_request_rate: 'Non-Human Request Suppression',
    suspicious_forwarded_byte_rate: 'Non-Human Byte Suppression',
    suspicious_forwarded_latency_share: 'Non-Human Latency Suppression'
  });

  const knownGameLoopPolicyProfiles = Object.freeze({
    site_default_v1: {
      label: 'Mixed Site Default',
      activationStatus: 'strict_human_only_not_active',
      simOnlyTargetSummary:
        '10% suspicious-forwarded budgets remain mixed-site defaults here, not the strict human-only target.',
      humanCalibrationSummary:
        'Separate real-human traversal calibration remains required; adversary-sim alone cannot prove likely-human safety.',
      verifiedHandlingSummary:
        'Current machine-loop posture is still the mixed-site default, so verified-identity relaxation is not the strict reference target.'
    },
    human_only_private: {
      label: 'Human Only Private',
      activationStatus: 'strict_human_only_active',
      simOnlyTargetSummary:
        'Strict sim-only proof should drive known non-human leakage toward zero or equivalent fail-closed suppression.',
      humanCalibrationSummary:
        'Separate real-human traversal calibration is still required after the strict config is found.',
      verifiedHandlingSummary:
        'Strict loop must still deny or equivalently suppress verified non-human traffic; verified identity stays telemetry and attribution.'
    },
    humans_plus_verified_only: {
      label: 'Humans Plus Verified Only',
      activationStatus: 'later_relaxed_verified_sweep',
      simOnlyTargetSummary:
        'This is a later relaxed verified-only sweep, not the strict human-only reference loop.',
      humanCalibrationSummary:
        'Real-human traversal measurement remains a separate proof ring from adversary-sim traffic.',
      verifiedHandlingSummary:
        'Verified non-human traffic can be evaluated separately here without reopening general unverified non-human access.'
    }
  });

  const asRecord = (value) =>
    value && typeof value === 'object' ? value : {};
  const toArray = (value) => (Array.isArray(value) ? value : []);

  const humanizeToken = (value, mode = 'title') => {
    const normalized = String(value || '').trim().replace(/[-]+/g, '_');
    if (!normalized) return mode === 'sentence' ? 'not available' : 'Not Available';
    const words = normalized
      .split(/[_\s]+/)
      .filter(Boolean)
      .map((word) => {
        const lowered = word.toLowerCase();
        if (titleAcronyms[lowered]) return titleAcronyms[lowered];
        if (mode === 'sentence') return lowered;
        return lowered.charAt(0).toUpperCase() + lowered.slice(1);
      });
    return words.join(' ');
  };

  const resolveGameLoopPolicyProfile = (profileId) => {
    const normalized = String(profileId || '').trim();
    if (normalized && knownGameLoopPolicyProfiles[normalized]) {
      return {
        profileId: normalized,
        recognized: true,
        ...knownGameLoopPolicyProfiles[normalized]
      };
    }
    if (!normalized) {
      return {
        profileId: '',
        recognized: false,
        label: 'Not Available',
        activationStatus: 'objective_profile_not_available',
        simOnlyTargetSummary: 'Objective profile is not materialized yet.',
        humanCalibrationSummary:
          'Separate real-human traversal calibration remains required when evaluating likely-human safety.',
        verifiedHandlingSummary:
          'Verified-identity handling cannot be interpreted until the objective profile is available.'
      };
    }
    return {
      profileId: normalized,
      recognized: false,
      label: humanizeToken(normalized),
      activationStatus: 'custom_profile_active',
      simOnlyTargetSummary:
        'This profile is custom, so strict-loop target semantics must be read from the persisted profile and active proof plan.',
      humanCalibrationSummary:
        'Separate real-human traversal calibration remains required when evaluating likely-human safety.',
      verifiedHandlingSummary:
        'Verified-identity handling under this custom profile must be interpreted from the persisted posture matrix, not the legacy request-path stance alone.'
    };
  };

  const formatTimestamp = (value) => formatUnixSecondsLocal(value, '-');

  const formatNumber = (value, fallback = 'n/a') => {
    if (value === null || value === undefined || value === '') return fallback;
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return fallback;
    return formatCompactNumber(numeric, '0');
  };

  const asFiniteNumber = (value) => {
    const numeric = Number(value);
    return Number.isFinite(numeric) ? numeric : null;
  };

  const metricLooksRatio = (metricId = '') => /(rate|ratio|share|percent|mismatch)/i.test(metricId);

  const formatMetricValue = (metricId, value) => {
    if (value === null || value === undefined || value === '') return 'n/a';
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return 'n/a';
    if (metricLooksRatio(metricId)) return `${(numeric * 100).toFixed(1)}%`;
    if (Math.abs(numeric) >= 1000) return formatCompactNumber(numeric, '0');
    if (Number.isInteger(numeric)) return String(numeric);
    return numeric.toFixed(2);
  };

  const formatRatioPercent = (value, fallback = 'n/a') => {
    if (value === null || value === undefined || value === '') return fallback;
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return fallback;
    return `${(numeric * 100).toFixed(1)}%`;
  };

  const ratioToTarget = (current, target) => {
    const currentNumeric = asFiniteNumber(current);
    const targetNumeric = asFiniteNumber(target);
    if (currentNumeric === null || targetNumeric === null || targetNumeric <= 0) {
      return null;
    }
    return currentNumeric / targetNumeric;
  };

  const zeroTargetSuppressionRatio = (current) => {
    const currentNumeric = asFiniteNumber(current);
    if (currentNumeric === null) return null;
    return Math.max(0, Math.min(1, 1 - currentNumeric));
  };

  const clampPercent = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric <= 0) return 0;
    return Math.max(0, Math.min(100, numeric));
  };

  const formatTargetRatioText = (ratio) => {
    const numeric = Number(ratio);
    if (!Number.isFinite(numeric)) return 'Target usage unavailable';
    return `${(numeric * 100).toFixed(0)}% of target`;
  };

  const isZeroTargetSuppressionMetric = (metricId, target) =>
    zeroTargetSuppressionMetricIds.has(String(metricId || '').trim()) && asFiniteNumber(target) === 0;

  const formatLeakageMetricValue = (metricId, value) => {
    const formatted = formatMetricValue(metricId, value);
    return formatted === 'n/a' ? formatted : `${formatted} leakage`;
  };

  const formatSuppressionAchievementText = (ratio) => {
    const numeric = Number(ratio);
    if (!Number.isFinite(numeric)) return 'Suppression progress unavailable';
    return `${(numeric * 100).toFixed(1)}% non-human suppression achieved`;
  };

  const budgetMetricLabel = (metricId, zeroTargetSuppression) => {
    const normalized = String(metricId || '').trim();
    if (zeroTargetSuppression && zeroTargetSuppressionLabels[normalized]) {
      return zeroTargetSuppressionLabels[normalized];
    }
    return humanizeToken(normalized);
  };

  const categoryIdFromMetric = (metricId = '') => {
    const normalized = String(metricId || '').trim();
    return normalized.startsWith('category_posture_alignment:')
      ? normalized.slice('category_posture_alignment:'.length)
      : '';
  };

  const formatSignedMetricValue = (metricId, value) => {
    if (value === null || value === undefined || value === '') return 'n/a';
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return 'n/a';
    const prefix = numeric > 0 ? '+' : '';
    if (metricLooksRatio(metricId)) return `${prefix}${(numeric * 100).toFixed(1)}%`;
    if (Math.abs(numeric) >= 1000) return `${prefix}${formatCompactNumber(numeric, '0')}`;
    if (Number.isInteger(numeric)) return `${prefix}${numeric}`;
    return `${prefix}${numeric.toFixed(2)}`;
  };

  const findBenchmarkFamily = (familyId) =>
    toArray(benchmarkResults?.families).find((family) => family?.family_id === familyId) || null;

  const dedupeStrings = (values) => {
    const seen = new Set();
    const next = [];
    toArray(values).forEach((entry) => {
      const normalized = String(entry || '').trim();
      if (!normalized || seen.has(normalized)) return;
      seen.add(normalized);
      next.push(normalized);
    });
    return next;
  };

  $: latestHistoryRows = toArray(oversightHistory?.rows).slice(0, 6);
  $: latestHistoryRow = latestHistoryRows[0] || null;
  $: latestDecision = asRecord(oversightAgentStatus?.latest_decision);
  $: latestRecentRun = toArray(oversightAgentStatus?.recent_runs)[0] || null;
  $: episodeArchive = asRecord(oversightHistory?.episode_archive?.schema_version
    ? oversightHistory?.episode_archive
    : oversightAgentStatus?.episode_archive);
  $: episodeArchiveRows = toArray(episodeArchive?.rows);
  $: judgedCycleRows = episodeArchiveRows.filter((row) => {
    const outcome = String(row?.retain_or_rollback || '').trim();
    return outcome === 'retained' || outcome === 'rolled_back';
  });
  $: retainedCycleCount = judgedCycleRows.filter(
    (row) => String(row?.retain_or_rollback || '').trim() === 'retained'
  ).length;
  $: rolledBackCycleCount = judgedCycleRows.filter(
    (row) => String(row?.retain_or_rollback || '').trim() === 'rolled_back'
  ).length;
  $: currentObjectiveProfileId = String(operatorSnapshot?.objectives?.profile_id || '').trim();
  $: currentPolicyProfile = resolveGameLoopPolicyProfile(currentObjectiveProfileId);
  $: suspiciousOriginCostFamily = findBenchmarkFamily('suspicious_origin_cost');
  $: likelyHumanFrictionFamily = findBenchmarkFamily('likely_human_friction');
  $: pressureFamilies = toArray(benchmarkResults?.families).filter((family) =>
    family && (family.status === 'outside_budget' || family.status === 'near_limit')
  );
  $: recentChanges = toArray(operatorSnapshot?.recent_changes?.rows).slice(0, 3);
  $: categoryPostureTargets = new Map(
    toArray(operatorSnapshot?.objectives?.category_postures).map((row) => [
      String(row?.category_id || '').trim(),
      String(row?.posture || '').trim()
    ])
  );
  $: latestScraplingEvidence = deriveLatestScraplingEvidenceFromSummaries(
    toArray(operatorSnapshot?.adversary_sim?.recent_runs)
  );
  $: budgetUsageRows = [likelyHumanFrictionFamily, suspiciousOriginCostFamily]
    .flatMap((family) =>
      toArray(family?.metrics)
        .filter((metric) => metric && metric.target !== null && metric.target !== undefined)
        .map((metric) => {
          const zeroTargetSuppression = isZeroTargetSuppressionMetric(
            metric.metric_id,
            metric.target
          );
          const usageRatio = zeroTargetSuppression
            ? zeroTargetSuppressionRatio(metric.current)
            : ratioToTarget(metric.current, metric.target);
          return {
            metricId: metric.metric_id,
            label: budgetMetricLabel(metric.metric_id, zeroTargetSuppression),
            currentText: zeroTargetSuppression
              ? formatLeakageMetricValue(metric.metric_id, metric.current)
              : formatMetricValue(metric.metric_id, metric.current),
            targetText: zeroTargetSuppression
              ? formatLeakageMetricValue(metric.metric_id, metric.target)
              : formatMetricValue(metric.metric_id, metric.target),
            deltaText: formatSignedMetricValue(metric.metric_id, metric.delta),
            comparisonText:
              metric.comparison_delta !== null && metric.comparison_delta !== undefined
                ? formatSignedMetricValue(metric.metric_id, metric.comparison_delta)
                : '',
            usageText: zeroTargetSuppression
              ? formatSuppressionAchievementText(usageRatio)
              : formatTargetRatioText(usageRatio),
            meterPercent: clampPercent((usageRatio || 0) * 100),
            statusText: humanizeToken(metric.status, 'sentence')
          };
        })
    );
  $: categoryTargetRows = toArray(findBenchmarkFamily('non_human_category_posture')?.metrics)
    .map((metric) => {
      const categoryId = categoryIdFromMetric(metric?.metric_id);
      const targetPosture = categoryPostureTargets.get(categoryId) || '';
      const achievementRatio = ratioToTarget(metric?.current, metric?.target);
      return {
        categoryId,
        label: humanizeToken(categoryId),
        targetPostureText: humanizeToken(targetPosture),
        achievedText: formatRatioPercent(metric?.current),
        targetText: formatRatioPercent(metric?.target),
        achievementText: formatTargetRatioText(achievementRatio),
        meterPercent: clampPercent((achievementRatio || 0) * 100),
        statusText: humanizeToken(metric?.status, 'sentence'),
        capabilityText: humanizeToken(metric?.capability_gate, 'sentence'),
        basisText: humanizeToken(metric?.basis, 'sentence')
      };
    })
    .filter((row) => row.categoryId);
  $: currentStatusCards = [
    {
      title: 'Overall Status',
      valueId: 'game-loop-current-status-overall-status',
      value: humanizeToken(benchmarkResults?.overall_status),
      note: benchmarkResults?.coverage_status
        ? `Coverage ${humanizeToken(benchmarkResults.coverage_status, 'sentence')}`
        : 'Benchmark results not materialized yet.'
    },
    {
      title: 'Improvement',
      valueId: 'game-loop-current-status-improvement',
      value: humanizeToken(benchmarkResults?.improvement_status),
      note: benchmarkResults?.baseline_reference?.note || 'Awaiting a comparable prior-window reference.'
    },
    {
      title: 'Tuning Eligibility',
      valueId: 'game-loop-current-status-tuning-eligibility',
      value: humanizeToken(benchmarkResults?.tuning_eligibility?.status),
      note: benchmarkResults?.tuning_eligibility?.blockers?.length
        ? `${benchmarkResults.tuning_eligibility.blockers.length} blocker(s) active`
        : 'No explicit tuning blockers are currently active.'
    },
    {
      title: 'Latest Controller Action',
      valueId: 'game-loop-current-status-controller-action',
      value: humanizeToken(latestHistoryRow?.apply?.stage || latestDecision?.outcome),
      note: latestHistoryRow?.summary || latestDecision?.summary || 'No oversight decision has been recorded yet.'
    }
  ];
  $: trustBlockers = dedupeStrings([
    ...(currentObjectiveProfileId === 'human_only_private'
      ? []
      : ['strict_human_only_reference_not_active']),
    ...(benchmarkResults?.tuning_eligibility?.blockers || []),
    ...(benchmarkResults?.non_human_classification?.blockers || []),
    ...(benchmarkResults?.non_human_coverage?.blocking_reasons || []),
    ...(benchmarkResults?.replay_promotion?.eligibility_blockers || [])
  ]);
</script>

<section
  id="dashboard-panel-game-loop"
  class="dashboard-tab-panel"
  data-dashboard-tab-panel="game-loop"
  aria-labelledby="dashboard-tab-game-loop"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'false'}
  tabindex="-1"
>
  <TabStateMessage tab="game-loop" status={tabStatus} />

  {#each gameLoopSections as section (section.id)}
    <section class="section" data-game-loop-section={section.id}>
      {#if section.title}
        <h2>{section.title}</h2>
      {/if}
      {#if section.description}
        <p class="section-desc text-muted">{section.description}</p>
      {/if}

      {#if section.id === 'current-status'}
        <div class="stats-cards">
          {#each currentStatusCards as card (card.valueId)}
            <MetricStatCard title={card.title} valueId={card.valueId} value={card.value}>
              <p class="text-muted">{card.note}</p>
            </MetricStatCard>
          {/each}
        </div>
        <div class="panel panel-soft pad-md">
          <div class="status-rows">
            <div class="info-row">
              <span class="info-label text-muted">Runtime posture:</span>
              <span class="status-value">
                {humanizeToken(operatorSnapshot?.runtime_posture?.runtime_environment, 'sentence')}
                | fail {humanizeToken(operatorSnapshot?.runtime_posture?.fail_mode, 'sentence')}
                | shadow {operatorSnapshot?.runtime_posture?.shadow_mode ? 'on' : 'off'}
                | adversary sim {operatorSnapshot?.runtime_posture?.adversary_sim_available ? 'available' : 'unavailable'}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Traffic stance:</span>
              <span class="status-value">
                live {formatNumber(operatorSnapshot?.live_traffic?.total_requests, '0')} requests,
                {formatNumber(operatorSnapshot?.live_traffic?.forwarded_requests, '0')} forwarded,
                {formatNumber(operatorSnapshot?.live_traffic?.short_circuited_requests, '0')} short-circuited
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Adversary sim:</span>
              <span class="status-value">
                {formatNumber(operatorSnapshot?.adversary_sim?.total_requests, '0')} simulated requests,
                {formatNumber(operatorSnapshot?.adversary_sim?.forwarded_requests, '0')} forwarded,
                {toArray(operatorSnapshot?.adversary_sim?.recent_runs).length} recent run(s)
              </span>
            </div>
          </div>
        </div>
      {:else if section.id === 'recent-loop-progress'}
        <div class="panel panel-soft pad-md">
          <div class="status-rows">
            <div class="info-row">
              <span class="info-label text-muted">Baseline:</span>
              <span class="status-value">
                {humanizeToken(benchmarkResults?.baseline_reference?.status, 'sentence')}
                {#if benchmarkResults?.baseline_reference?.generated_at}
                  | {formatTimestamp(benchmarkResults.baseline_reference.generated_at)}
                {/if}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Oversight cadence:</span>
              <span class="status-value">
                {formatNumber(oversightAgentStatus?.periodic_trigger?.default_interval_seconds, '0')}s
                via <code>{oversightAgentStatus?.periodic_trigger?.surface || 'n/a'}</code>
              </span>
            </div>
            <div id="game-loop-progress-lineage" class="info-row">
              <span class="info-label text-muted">Cycle Lineage:</span>
              <span class="status-value">
                {formatNumber(judgedCycleRows.length, '0')} completed judged cycles
                | {formatNumber(retainedCycleCount, '0')} retained
                | {formatNumber(rolledBackCycleCount, '0')} rolled back
                | homeostasis {humanizeToken(episodeArchive?.homeostasis?.status, 'sentence')}
              </span>
            </div>
          </div>
        </div>
        <div id="game-loop-progress-history" class="panel panel-soft pad-md">
          {#if latestHistoryRows.length === 0}
            <p class="text-muted">No bounded loop history is available yet.</p>
          {:else}
            <ul class="metric-list">
              {#each latestHistoryRows as row (row.decision_id)}
                <li>
                  <strong>{formatTimestamp(row.recorded_at_ts)}</strong>:
                  {row.summary}
                  <br />
                  <span class="text-muted">
                    {humanizeToken(row.benchmark_overall_status)} |
                    {humanizeToken(row.improvement_status)} |
                    {humanizeToken(row.apply?.stage || row.outcome)}
                  </span>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else if section.id === 'outcome-frontier'}
        <div id="game-loop-outcome-frontier" class="panel panel-soft pad-md">
          <h3 class="caps-label">Budget Usage</h3>
          {#if budgetUsageRows.length === 0}
            <p class="text-muted">No numeric objective budgets are materialized yet.</p>
          {:else}
            <div id="game-loop-budget-usage" class="game-loop-meter-list">
              {#each budgetUsageRows as row (row.metricId)}
                <div class="game-loop-meter-row">
                  <p class="caps-label">{row.label}</p>
                  <div class="game-loop-meter" aria-hidden="true">
                    <span class="game-loop-meter__fill" style={`width: ${row.meterPercent}%;`}></span>
                  </div>
                  <p class="game-loop-meter-meta text-muted">
                    Current {row.currentText} | Target {row.targetText} | {row.usageText} | {row.statusText}
                    {#if row.comparisonText}
                      | vs prior {row.comparisonText}
                    {/if}
                  </p>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if section.id === 'change-judgment'}
        <div id="game-loop-change-judgment" class="panel panel-soft pad-md">
          <div class="status-rows">
            <div class="info-row">
              <span class="info-label text-muted">Benchmark decision:</span>
              <span class="status-value">{humanizeToken(benchmarkResults?.escalation_hint?.decision)}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Review status:</span>
              <span class="status-value">{humanizeToken(benchmarkResults?.escalation_hint?.review_status)}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Latest decision:</span>
              <span class="status-value">{latestHistoryRow?.summary || latestDecision?.summary || 'No decision recorded yet.'}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Latest run:</span>
              <span class="status-value">
                {#if latestRecentRun}
                  {humanizeToken(latestRecentRun.trigger_kind)} at {formatTimestamp(latestRecentRun.completed_at_ts)}
                {:else}
                  No recorded agent runs yet.
                {/if}
              </span>
            </div>
          </div>

          {#if benchmarkResults?.escalation_hint?.candidate_action_families?.length}
            <p class="text-muted">
              Candidate action families:
              {benchmarkResults.escalation_hint.candidate_action_families.map((family) => humanizeToken(family)).join(', ')}
            </p>
          {/if}

          {#if benchmarkResults?.escalation_hint?.blockers?.length}
            <p class="text-muted">
              Decision blockers:
              {benchmarkResults.escalation_hint.blockers.map((blocker) => humanizeToken(blocker, 'sentence')).join(', ')}
            </p>
          {/if}
        </div>
      {:else if section.id === 'pressure-sits'}
        <div class="stats-cards">
          <article class="card panel panel-border pad-md-b">
            <h3 class="caps-label">Benchmark Pressure</h3>
            {#if pressureFamilies.length === 0}
              <p class="text-muted">No benchmark families are currently near limit or outside budget.</p>
            {:else}
              <ul class="metric-list">
                {#each pressureFamilies as family (family.family_id)}
                  <li>
                    <strong>{humanizeToken(family.family_id)}</strong>:
                    {humanizeToken(family.status)} | {family.note}
                  </li>
                {/each}
              </ul>
            {/if}
          </article>
          <article id="game-loop-category-target-achievement" class="card panel panel-border pad-md-b">
            <h3 class="caps-label">Category Posture Achievement</h3>
            <p class="text-muted">
              These rows score category-level posture alignment. They do not prove whether Scrapling satisfied or failed its required defense-surface contract.
            </p>
            {#if categoryTargetRows.length === 0}
              <p class="text-muted">No category target-achievement rows are materialized yet.</p>
            {:else}
              <div class="game-loop-meter-list">
                {#each categoryTargetRows as row (row.categoryId)}
                  <div class="game-loop-meter-row">
                    <p class="caps-label">{row.label}</p>
                    <div class="game-loop-meter" aria-hidden="true">
                      <span class="game-loop-meter__fill" style={`width: ${row.meterPercent}%;`}></span>
                    </div>
                    <p class="game-loop-meter-meta text-muted">
                      Target {row.targetPostureText} | Achieved {row.achievedText} | Goal {row.targetText} | {row.achievementText} | {row.statusText}
                    </p>
                    <p class="game-loop-meter-meta text-muted">
                      Support {row.capabilityText} | Basis {row.basisText}
                    </p>
                  </div>
                {/each}
              </div>
            {/if}
          </article>
          <article class="card panel panel-border pad-md-b">
            <h3 class="caps-label">Recent Change Context</h3>
            {#if recentChanges.length === 0}
              <p class="text-muted">No recent config-change ledger rows are materialized yet.</p>
            {:else}
              <ul class="metric-list">
                {#each recentChanges as change, index (`${change.changed_at_ts}-${index}`)}
                  <li>
                    <strong>{formatTimestamp(change.changed_at_ts)}</strong>:
                    {change.change_summary || 'Change summary unavailable.'}
                  </li>
                {/each}
              </ul>
            {/if}
          </article>
        </div>
      {:else if section.id === 'trust-and-blockers'}
        <div id="game-loop-trust-blockers" class="panel panel-soft pad-md">
          <div class="status-rows">
            <div class="info-row">
              <span class="info-label text-muted">Policy Profile:</span>
              <span class="status-value">
                {currentPolicyProfile.label}
                | profile {currentPolicyProfile.profileId || 'not available'}
                | {humanizeToken(currentPolicyProfile.activationStatus, 'sentence')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Sim-Only Target:</span>
              <span class="status-value">{currentPolicyProfile.simOnlyTargetSummary}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Human Calibration:</span>
              <span class="status-value">{currentPolicyProfile.humanCalibrationSummary}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Classification:</span>
              <span class="status-value">
                {humanizeToken(benchmarkResults?.non_human_classification?.status)}
                | live receipts {formatNumber(benchmarkResults?.non_human_classification?.live_receipt_count, '0')}
                | sim receipts {formatNumber(benchmarkResults?.non_human_classification?.adversary_sim_receipt_count, '0')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Coverage:</span>
              <span class="status-value">{humanizeToken(benchmarkResults?.non_human_coverage?.overall_status)}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Protected Replay:</span>
              <span class="status-value">
                {humanizeToken(benchmarkResults?.replay_promotion?.availability)}
                | evidence {humanizeToken(benchmarkResults?.replay_promotion?.evidence_status, 'sentence')}
                | lineage {formatNumber(benchmarkResults?.replay_promotion?.protected_lineage_count, '0')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Tuning Eligibility:</span>
              <span class="status-value">
                {humanizeToken(benchmarkResults?.tuning_eligibility?.status)}
                | blockers {formatNumber(toArray(benchmarkResults?.tuning_eligibility?.blockers).length, '0')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Verified Identity:</span>
              <span class="status-value">
                {humanizeToken(operatorSnapshot?.verified_identity?.availability)}
                | alignment {humanizeToken(operatorSnapshot?.verified_identity?.taxonomy_alignment?.status)}
                | verified mode {humanizeToken(operatorSnapshot?.verified_identity?.effective_non_human_policy?.verified_identity_override_mode, 'sentence')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Verified Handling:</span>
              <span class="status-value">
                {currentPolicyProfile.verifiedHandlingSummary}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Latest Scrapling Surface Contract:</span>
              <span class="status-value">
                {#if latestScraplingEvidence?.ownedSurfaceCoverage}
                  {humanizeToken(latestScraplingEvidence.ownedSurfaceCoverage.overallStatus)}
                  | {formatNumber(latestScraplingEvidence.ownedSurfaceCoverage.satisfiedSurfaceCount, '0')}
                  / {formatNumber(latestScraplingEvidence.ownedSurfaceCoverage.requiredSurfaceCount, '0')}
                  required surfaces
                  | categories {latestScraplingEvidence.observedCategoryIds.map((value) => humanizeToken(value)).join(', ') || 'not available'}
                {:else}
                  No receipt-backed Scrapling surface-contract evidence is materialized yet.
                {/if}
              </span>
            </div>
          </div>

          {#if trustBlockers.length === 0}
            <p class="text-muted">No explicit trust blockers are currently active.</p>
          {:else}
            <ul class="metric-list">
              {#each trustBlockers as blocker}
                <li>{humanizeToken(blocker, 'sentence')}</li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>
  {/each}
</section>
