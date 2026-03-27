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
      title: 'Origin Leakage And Human Cost',
      description:
        'These guardrails measure suspicious-origin leakage and measured human friction. They do not, by themselves, prove total attacker defeat.'
    },
    {
      id: 'change-judgment',
      title: 'Loop Actionability',
      description:
        'The machine-first decision state, bounded move readiness, and whether the controller actually applied a config move or stayed blocked.'
    },
    {
      id: 'pressure-sits',
      title: 'Board State',
      description:
        'Terrain breach progress, surface-contract satisfaction, recognition evaluation, and recent change context shown as separate truths.'
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
  const terminalLoopOutcomes = new Set(['config_ring_exhausted', 'code_evolution_referral']);

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
    if (value === null || value === undefined || value === '') return null;
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

  const formatCategoryAchievedText = (current) =>
    asFiniteNumber(current) === null ? 'Unscored' : formatRatioPercent(current);

  const formatCategoryAchievementText = (ratio, unscored) =>
    unscored ? 'Unscored pending exact evidence' : formatTargetRatioText(ratio);

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

  const firstPrefixedValue = (values, prefix) =>
    dedupeStrings(values).find((entry) => entry.startsWith(prefix)) || '';

  const removePrefix = (value, prefix) =>
    String(value || '').startsWith(prefix) ? String(value || '').slice(prefix.length) : String(value || '');

  const formatLocusSample = (locus) => {
    const source = asRecord(locus);
    const method = String(source.sample_request_method || '').trim();
    const path = String(source.sample_request_path || '').trim();
    const status = asFiniteNumber(source.sample_response_status);
    const parts = [];
    const requestLine = [method, path].filter(Boolean).join(' ');
    if (requestLine) parts.push(requestLine);
    if (status !== null) parts.push(`-> ${formatNumber(status, '0')}`);
    return parts.join(' ') || 'Sample unavailable';
  };

  const humanizeList = (values, mode = 'sentence') => {
    const items = dedupeStrings(values).map((value) => humanizeToken(value, mode));
    return items.length ? items.join(', ') : 'not available';
  };

  const joinStatusSummary = (parts, fallback) => {
    const normalized = dedupeStrings(parts.filter(Boolean));
    return normalized.length ? normalized.join(' | ') : fallback;
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
  $: recognitionEvaluation = asRecord(operatorSnapshot?.non_human_traffic?.recognition_evaluation);
  $: suspiciousOriginCostFamily = findBenchmarkFamily('suspicious_origin_cost');
  $: likelyHumanFrictionFamily = findBenchmarkFamily('likely_human_friction');
  $: exploitProgressFamily = findBenchmarkFamily('scrapling_exploit_progress');
  $: evidenceQuality = asRecord(benchmarkResults?.escalation_hint?.evidence_quality);
  $: urgencySummary = asRecord(benchmarkResults?.urgency);
  $: recentChanges = toArray(operatorSnapshot?.recent_changes?.rows).slice(0, 3);
  $: latestMoveOutcome = String(
    latestHistoryRow?.outcome || latestDecision?.outcome || benchmarkResults?.escalation_hint?.decision || ''
  ).trim();
  $: latestApplyStage = String(latestHistoryRow?.apply?.stage || '').trim();
  $: selectedRepairSurface = String(
    latestHistoryRow?.proposal?.patch_family ||
      benchmarkResults?.escalation_hint?.family_guidance?.[0]?.family ||
      benchmarkResults?.escalation_hint?.candidate_action_families?.[0] ||
      ''
  ).trim();
  $: configRingBlockedToken = firstPrefixedValue(
    [
      ...toArray(benchmarkResults?.tuning_eligibility?.blockers),
      ...toArray(benchmarkResults?.escalation_hint?.blockers),
      ...toArray(latestHistoryRow?.refusal_reasons)
    ],
    'config_ring_exhausted:'
  );
  $: configRingBlockedFamily = removePrefix(configRingBlockedToken, 'config_ring_exhausted:');
  $: configRingStatus =
    latestMoveOutcome === 'config_ring_exhausted' || configRingBlockedToken
      ? 'exhausted'
      : selectedRepairSurface
        ? 'bounded_ring_available'
        : 'not_evaluated';
  $: codeEvolutionStatus =
    latestMoveOutcome === 'code_evolution_referral' ||
    benchmarkResults?.escalation_hint?.decision === 'code_evolution_candidate'
      ? 'required'
      : 'not_required';
  $: loopActionabilityDecisionToken = terminalLoopOutcomes.has(latestMoveOutcome)
    ? latestMoveOutcome
    : benchmarkResults?.escalation_hint?.decision;
  $: loopActionabilityApplyText = latestApplyStage
    ? humanizeToken(latestApplyStage, 'sentence')
    : 'No Config Move Applied';
  $: loopActionabilityText = dedupeStrings([
    humanizeToken(benchmarkResults?.tuning_eligibility?.status),
    humanizeToken(loopActionabilityDecisionToken, 'sentence'),
    loopActionabilityApplyText
  ]).join(' | ');
  $: loopActionabilityNote = benchmarkResults?.tuning_eligibility?.blockers?.length
    ? `Blocked by ${benchmarkResults.tuning_eligibility.blockers.length} active guardrail(s) or readiness blocker(s).`
    : latestHistoryRow?.summary ||
      latestDecision?.summary ||
      benchmarkResults?.escalation_hint?.note ||
      'No bounded config move has been applied yet.';
  $: breachLoci = (() => {
    const escalationLoci = toArray(benchmarkResults?.escalation_hint?.breach_loci);
    if (escalationLoci.length > 0) return escalationLoci;
    const evidenceLoci = toArray(evidenceQuality?.breach_loci);
    if (evidenceLoci.length > 0) return evidenceLoci;
    return toArray(exploitProgressFamily?.exploit_loci);
  })();
  $: homeostasisSummary = asRecord(episodeArchive?.homeostasis);
  $: homeostasisBreakReasons = dedupeStrings([
    ...toArray(homeostasisSummary?.break_reasons),
    ...toArray(urgencySummary?.homeostasis_break_reasons)
  ]);
  $: homeostasisBreakStatus = String(
    homeostasisSummary?.break_status || urgencySummary?.homeostasis_break_status || ''
  ).trim();
  $: exploitUrgencyValue = humanizeToken(urgencySummary?.exploit_short_window_status);
  $: exploitUrgencyNote = joinStatusSummary(
    [
      urgencySummary?.exploit_short_window_status
        ? `Short window ${humanizeToken(urgencySummary?.exploit_short_window_status, 'sentence')}`
        : '',
      urgencySummary?.exploit_long_window_status
        ? `Trend ${humanizeToken(urgencySummary?.exploit_long_window_status, 'sentence')}`
        : '',
      homeostasisBreakStatus
        ? `Homeostasis ${humanizeToken(homeostasisBreakStatus, 'sentence')}`
        : ''
    ],
    urgencySummary?.note || 'Exploit urgency is not materialized yet.'
  );
  $: humanFrictionUrgencyValue = humanizeToken(urgencySummary?.likely_human_short_window_status);
  $: humanFrictionUrgencyNote = joinStatusSummary(
    [
      urgencySummary?.likely_human_short_window_status
        ? `Short window ${humanizeToken(
            urgencySummary?.likely_human_short_window_status,
            'sentence'
          )}`
        : '',
      urgencySummary?.likely_human_long_window_status
        ? `Trend ${humanizeToken(urgencySummary?.likely_human_long_window_status, 'sentence')}`
        : ''
    ],
    'Human-friction urgency is not materialized yet.'
  );
  $: restartBaseline = asRecord(homeostasisSummary?.restart_baseline);
  $: categoryPostureTargets = new Map(
    toArray(operatorSnapshot?.objectives?.category_postures).map((row) => [
      String(row?.category_id || '').trim(),
      String(row?.posture || '').trim()
    ])
  );
  $: latestScraplingEvidence = deriveLatestScraplingEvidenceFromSummaries(
    toArray(operatorSnapshot?.adversary_sim?.recent_runs)
  );
  $: surfaceContractCoverage = asRecord(latestScraplingEvidence?.ownedSurfaceCoverage);
  $: surfaceContractBlockingLabels = (() => {
    const blockingReceipts = toArray(surfaceContractCoverage?.receipts).filter(
      (receipt) => receipt && receipt.satisfied !== true
    );
    if (blockingReceipts.length > 0) {
      return blockingReceipts.map((receipt) => {
        const label = String(receipt?.surfaceLabel || '').trim() || humanizeToken(receipt?.surfaceId);
        const stateLabel = String(receipt?.surfaceStateLabel || '').trim();
        const dependencyLabel = String(receipt?.dependencyLabel || '').trim();
        if (stateLabel && dependencyLabel) {
          return `${label} (${stateLabel} | ${dependencyLabel})`;
        }
        return stateLabel ? `${label} (${stateLabel})` : label;
      });
    }
    return toArray(surfaceContractCoverage?.blockingSurfaceIds).map((surfaceId) => humanizeToken(surfaceId));
  })();
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
      const currentValue = asFiniteNumber(metric?.current);
      const isUnscored = currentValue === null;
      const achievementRatio = ratioToTarget(metric?.current, metric?.target);
      return {
        categoryId,
        label: humanizeToken(categoryId),
        targetPostureText: humanizeToken(targetPosture),
        achievedText: formatCategoryAchievedText(metric?.current),
        targetText: formatRatioPercent(metric?.target),
        achievementText: formatCategoryAchievementText(achievementRatio, isUnscored),
        meterPercent: isUnscored ? null : clampPercent((achievementRatio || 0) * 100),
        isUnscored,
        statusText: humanizeToken(metric?.status, 'sentence'),
        capabilityText: humanizeToken(metric?.capability_gate, 'sentence'),
        basisText: humanizeToken(metric?.basis, 'sentence')
      };
    })
    .filter((row) => row.categoryId);
  $: exploitProgressRows = toArray(exploitProgressFamily?.metrics)
    .map((metric) => ({
      metricId: metric?.metric_id,
      label: humanizeToken(metric?.metric_id),
      currentText: formatMetricValue(metric?.metric_id, metric?.current),
      targetText: formatMetricValue(metric?.metric_id, metric?.target),
      deltaText: formatSignedMetricValue(metric?.metric_id, metric?.delta),
      comparisonText:
        metric?.comparison_delta !== null && metric?.comparison_delta !== undefined
          ? formatSignedMetricValue(metric?.metric_id, metric?.comparison_delta)
          : '',
      statusText: humanizeToken(metric?.status, 'sentence')
    }))
    .filter((row) => row.metricId);
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
      title: 'Loop Actionability',
      valueId: 'game-loop-current-status-loop-actionability',
      value: loopActionabilityText,
      note: loopActionabilityNote
    },
    {
      title: 'Terrain Breach Progress',
      valueId: 'game-loop-current-status-exploit-progress',
      value: humanizeToken(exploitProgressFamily?.status),
      note:
        exploitProgressFamily?.note ||
        'No receipt-backed exploit-progress summary is materialized yet.'
    },
    {
      title: 'Evidence Quality',
      valueId: 'game-loop-current-status-evidence-quality',
      value: humanizeToken(evidenceQuality?.status),
      note:
        evidenceQuality?.note ||
        'Diagnosis confidence and evidence quality have not been materialized yet.'
    },
    {
      title: 'Exploit Urgency',
      valueId: 'game-loop-current-status-exploit-urgency',
      value: exploitUrgencyValue,
      note: exploitUrgencyNote
    },
    {
      title: 'Human Friction Urgency',
      valueId: 'game-loop-current-status-human-friction-urgency',
      value: humanFrictionUrgencyValue,
      note: humanFrictionUrgencyNote
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
        <div class="stats-cards stats-cards--summary">
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
            <div id="game-loop-progress-break-state" class="info-row">
              <span class="info-label text-muted">Homeostasis Break:</span>
              <span class="status-value">
                {humanizeToken(homeostasisBreakStatus, 'sentence')}
                {#if homeostasisBreakReasons.length}
                  | reasons {homeostasisBreakReasons.map((reason) => humanizeToken(reason, 'sentence')).join(', ')}
                {/if}
                {#if restartBaseline?.source}
                  | restart baseline {humanizeToken(restartBaseline.source, 'sentence')}
                  {#if restartBaseline.generated_at}
                    @ {formatTimestamp(restartBaseline.generated_at)}
                  {/if}
                {/if}
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
          <h3 class="caps-label">Origin Leakage And Human Cost</h3>
          <p class="text-muted">
            These rows are guardrails. They can be fully inside budget even while terrain breach progress is still non-zero elsewhere on the board.
          </p>
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
              <span class="info-label text-muted">Loop Actionability:</span>
              <span class="status-value">{loopActionabilityText}</span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Judge State:</span>
              <span class="status-value">
                {humanizeToken(benchmarkResults?.overall_status)}
                | improvement {humanizeToken(benchmarkResults?.improvement_status, 'sentence')}
                | coverage {humanizeToken(benchmarkResults?.coverage_status, 'sentence')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Urgency Split:</span>
              <span class="status-value">
                Exploit urgency {humanizeToken(urgencySummary?.exploit_short_window_status, 'sentence')}
                | trend {humanizeToken(urgencySummary?.exploit_long_window_status, 'sentence')}
                | Human friction urgency {humanizeToken(
                  urgencySummary?.likely_human_short_window_status,
                  'sentence'
                )}
                | trend {humanizeToken(urgencySummary?.likely_human_long_window_status, 'sentence')}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Evidence Quality:</span>
              <span class="status-value">
                {humanizeToken(evidenceQuality?.status)}
                | attribution {humanizeToken(evidenceQuality?.attribution_status)}
                | locality {humanizeToken(evidenceQuality?.locality_status)}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Diagnosis:</span>
              <span class="status-value">
                {humanizeToken(evidenceQuality?.diagnosis_confidence)}
                | problem class {humanizeToken(benchmarkResults?.escalation_hint?.problem_class)}
                | repair surface {humanizeToken(selectedRepairSurface)}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Move Or Escalation:</span>
              <span class="status-value">
                {humanizeToken(latestMoveOutcome)}
                | benchmark {humanizeToken(benchmarkResults?.escalation_hint?.decision)}
                | review {humanizeToken(benchmarkResults?.escalation_hint?.review_status)}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Config Ring:</span>
              <span class="status-value">
                {humanizeToken(configRingStatus)}
                {#if configRingBlockedFamily}
                  | exhausted at {humanizeToken(configRingBlockedFamily)}
                {/if}
              </span>
            </div>
            <div class="info-row">
              <span class="info-label text-muted">Code Evolution:</span>
              <span class="status-value">{humanizeToken(codeEvolutionStatus)}</span>
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
          <article id="game-loop-exploit-progress" class="card panel panel-border pad-md-b">
            <h3 class="caps-label">Terrain Breach Progress</h3>
            <p class="text-muted">
              This surface tracks where Scrapling actually advanced through the defended terrain. It is separate from origin leakage budgets and separate from category posture scoring.
            </p>
            <div class="status-rows">
              <div class="info-row">
                <span class="info-label text-muted">Status:</span>
                <span class="status-value">
                  {humanizeToken(exploitProgressFamily?.status)}
                  | comparison {humanizeToken(exploitProgressFamily?.comparison_status, 'sentence')}
                </span>
              </div>
              <div class="info-row">
                <span class="info-label text-muted">Note:</span>
                <span class="status-value">{exploitProgressFamily?.note || 'Exploit progress is not materialized yet.'}</span>
              </div>
            </div>
            {#if breachLoci.length}
              <div id="game-loop-breach-loci">
                <p class="caps-label">Named Breach Loci</p>
                <ul class="metric-list">
                  {#each breachLoci as locus (`${locus.locus_id}-${locus.sample_request_path}`)}
                    <li>
                      <strong>{locus.locus_label || humanizeToken(locus.locus_id)}</strong>:
                      {humanizeToken(locus.evidence_status, 'sentence')} |
                      {humanizeToken(locus.stage_id, 'sentence')} |
                      {formatNumber(locus.attempt_count, '0')} attempts
                      <br />
                      <span class="text-muted">
                        Host cost {humanizeList(locus.cost_channel_ids)}
                        | repair {humanizeList(locus.repair_family_candidates)}
                        | {formatLocusSample(locus)}
                      </span>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
            {#if exploitProgressRows.length}
              <ul class="metric-list">
                {#each exploitProgressRows as row (row.metricId)}
                  <li>
                    <strong>{row.label}</strong>:
                    Current {row.currentText} | Goal {row.targetText} | {row.statusText}
                    {#if row.comparisonText}
                      | vs prior {row.comparisonText}
                    {/if}
                  </li>
                {/each}
              </ul>
            {/if}
          </article>
          <article id="game-loop-surface-contract" class="card panel panel-border pad-md-b">
            <h3 class="caps-label">Surface Contract Satisfaction</h3>
            <p class="text-muted">
              This surface asks whether Scrapling satisfied the required defense-surface contract for the latest run. It is not the same as category posture achievement.
            </p>
            {#if latestScraplingEvidence?.ownedSurfaceCoverage}
              <div class="status-rows">
                <div class="info-row">
                  <span class="info-label text-muted">Status:</span>
                  <span class="status-value">
                    {humanizeToken(surfaceContractCoverage.overallStatus)}
                    | {formatNumber(surfaceContractCoverage.satisfiedSurfaceCount, '0')}
                    / {formatNumber(surfaceContractCoverage.requiredSurfaceCount, '0')}
                    required surfaces
                  </span>
                </div>
                <div class="info-row">
                  <span class="info-label text-muted">Blocking Surfaces:</span>
                  <span class="status-value">
                    {surfaceContractBlockingLabels.length
                      ? surfaceContractBlockingLabels.join(', ')
                      : 'None'}
                  </span>
                </div>
              </div>
            {:else}
              <p class="text-muted">No receipt-backed Scrapling surface-contract evidence is materialized yet.</p>
            {/if}
          </article>
          <article id="game-loop-recognition-evaluation" class="card panel panel-border pad-md-b">
            <h3 class="caps-label">Recognition Evaluation</h3>
            <p class="text-muted">
              These rows are the recognition side quest. They compare Shuma-side category inference against simulator-known intent after the fact and must not drive bounded tuning or runtime restriction directly.
            </p>
            <div class="status-rows">
              <div class="info-row">
                <span class="info-label text-muted">Recognition Status:</span>
                <span class="status-value">
                  {humanizeToken(recognitionEvaluation?.comparison_status)}
                  | readiness {humanizeToken(recognitionEvaluation?.readiness?.status)}
                  | coverage {humanizeToken(recognitionEvaluation?.coverage?.overall_status)}
                </span>
              </div>
              <div class="info-row">
                <span class="info-label text-muted">Recognition Summary:</span>
                <span class="status-value">
                  exact {formatNumber(recognitionEvaluation?.current_exact_match_count, '0')}
                  | collapsed unknown {formatNumber(recognitionEvaluation?.collapsed_to_unknown_count, '0')}
                  | not materialized {formatNumber(recognitionEvaluation?.not_materialized_count, '0')}
                </span>
              </div>
            </div>
            {#if categoryTargetRows.length === 0}
              <p class="text-muted">No recognition-evaluation rows are materialized yet.</p>
            {:else}
              <div class="game-loop-meter-list">
                {#each categoryTargetRows as row (row.categoryId)}
                  <div class="game-loop-meter-row">
                    <p class="caps-label">{row.label}</p>
                    <div class="game-loop-meter" aria-hidden="true">
                      {#if row.meterPercent !== null}
                        <span class="game-loop-meter__fill" style={`width: ${row.meterPercent}%;`}></span>
                      {/if}
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
              <span class="info-label text-muted">Judge Path:</span>
              <span class="status-value">
                Sim and real traffic must share Shuma-side scoring truth; simulator metadata does not count as category truth.
              </span>
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
