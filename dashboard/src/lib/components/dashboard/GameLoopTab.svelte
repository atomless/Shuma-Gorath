<script>
  import { formatAdversarySimLaneLabel } from '../../domain/adversary-sim.js';
  import { formatCompactNumber } from '../../domain/core/format.js';
  import { formatUnixSecondsLocal } from '../../domain/core/date-time.js';
  import {
    deriveAdversaryRunRowsFromSummaries,
    deriveLlmSurfaceRowsFromRuntimeSummary,
    formatIdentityRealismSummary
  } from './monitoring-view-model.js';
  import TabStateMessage from './primitives/TabStateMessage.svelte';

  export let managed = false;
  export let isActive = true;
  export let tabStatus = null;
  export let operatorSnapshot = null;
  export let oversightHistory = null;
  export let oversightAgentStatus = null;

  const gameLoopSections = Object.freeze([
    {
      id: 'recent-rounds',
      title: 'Recent Rounds',
      description:
        'A simple recent history of judged rounds: who played, what move was tested, and whether the loop retained it or rolled it back.'
    },
    {
      id: 'adversary-cast',
      title: 'Adversaries In This Round',
      description:
        'This is the observer view. Simulator ground truth is visible here after the round, and recent recognition evaluation shows how Shuma categorized that traffic.'
    },
    {
      id: 'defence-cast',
      title: 'Defences In This Round',
      description:
        'This is the surface-native view. Each row stays with what the defence saw, how it responded, and whether the surface held or leaked.'
    }
  ]);

  const titleAcronyms = Object.freeze({
    ai: 'AI',
    api: 'API',
    cdp: 'CDP',
    http: 'HTTP',
    id: 'ID',
    ip: 'IP',
    js: 'JS',
    llm: 'LLM',
    pow: 'PoW',
    sim: 'Sim',
    ui: 'UI'
  });

  const asRecord = (value) => (value && typeof value === 'object' ? value : {});
  const toArray = (value) => (Array.isArray(value) ? value : []);

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

  const formatLaneList = (laneIds, fallback = 'Not available') => {
    const seen = new Set();
    const labels = toArray(laneIds)
      .map((laneId) => formatAdversarySimLaneLabel(laneId, ''))
      .filter((laneId) => {
        if (!laneId || seen.has(laneId)) return false;
        seen.add(laneId);
        return true;
      });
    return labels.length ? labels.join(', ') : fallback;
  };

  const summarizeRequiredRuns = (runs, fallback = 'No current mixed-attacker evidence set.') => {
    const labels = toArray(runs)
      .map((run) => {
        const laneLabel = formatAdversarySimLaneLabel(run?.lane, '');
        const statusLabel = humanizeToken(run?.status, 'sentence');
        return laneLabel && statusLabel ? `${laneLabel} ${statusLabel}` : '';
      })
      .filter(Boolean);
    return labels.length ? labels.join(' | ') : fallback;
  };

  const comparisonStatusText = (status) => {
    const normalized = String(status || '').trim();
    if (normalized === 'current_exact_match') return 'matched exactly';
    if (normalized === 'collapsed_to_unknown_non_human') return 'collapsed to unknown non-human';
    if (normalized === 'not_materialized') return 'not materialized';
    return humanizeToken(normalized, 'sentence');
  };

  const formatModeSummary = (modes) => {
    const labels = dedupeStrings(toArray(modes)).map((mode) => humanizeToken(mode));
    if (labels.length === 1) return labels[0];
    if (labels.length > 1) return 'Mixed activity';
    return 'No mode detail';
  };

  const formatObservedRunSummary = (run) => {
    if (!run) return 'No recent run detail is materialized yet.';
    const identitySummary =
      formatIdentityRealismSummary(run?.llmRuntimeSummary?.latestRealismReceipt)
      || formatIdentityRealismSummary(run?.latestScraplingRealismReceipt);
    const details = [
      formatModeSummary(run.observedFulfillmentModes),
      `${formatNumber(run.monitoringEventCount, '0')} monitoring events`,
      `${formatNumber(run.defenseDeltaCount, '0')} defence reactions`,
      `${formatNumber(run.banOutcomeCount, '0')} bans`
    ];
    const runtimeSummary = run?.llmRuntimeSummary;
    if (runtimeSummary?.provider || runtimeSummary?.modelId) {
      details.push(
        [humanizeToken(runtimeSummary.provider), runtimeSummary.modelId].filter(Boolean).join(' ')
      );
    }
    if (identitySummary) details.push(identitySummary);
    return details.filter(Boolean).join(' | ');
  };

  const formatRoundResultSummary = (row) => {
    const parts = dedupeStrings([
      row?.retain_or_rollback ? humanizeToken(row.retain_or_rollback) : '',
      row?.watch_window_result ? humanizeToken(row.watch_window_result) : ''
    ]);
    return parts.length ? parts.join(' | ') : 'No judged result';
  };

  const formatRoundMoveSummary = (row, historyRow) => {
    const patchFamily =
      row?.proposal?.patch_family ||
      historyRow?.proposal?.patch_family ||
      historyRow?.apply?.patch_family;
    return patchFamily ? humanizeToken(patchFamily) : 'No config move recorded';
  };

  const formatRoundNextState = (row, isLatest, currentSourceLabel, currentStatus) => {
    if (isLatest && currentSourceLabel && currentSourceLabel !== 'No active mixed-attacker evidence set') {
      return currentStatus
        ? `${currentSourceLabel} ${humanizeToken(currentStatus, 'sentence')}`
        : currentSourceLabel;
    }
    if (row?.homeostasis_break_status === 'triggered') return 'Homeostasis break triggered';
    return row?.cycle_judgment ? humanizeToken(row.cycle_judgment) : 'No next-state detail';
  };

  const formatReceiptObservation = (receipt) => {
    const source = asRecord(receipt);
    const attemptCount = asFiniteNumber(source.attemptCount);
    const requestLine = [source.sampleRequestMethod, source.sampleRequestPath].filter(Boolean).join(' ');
    const responseStatus = asFiniteNumber(source.sampleResponseStatus);
    const fragments = [];
    if (attemptCount !== null) fragments.push(`${formatNumber(attemptCount, '0')} attempts`);
    if (requestLine) fragments.push(`Saw ${requestLine}`);
    if (responseStatus !== null) fragments.push(`-> ${formatNumber(responseStatus, '0')}`);
    return fragments.length ? fragments.join(' | ') : 'No attempt materialized in this round.';
  };

  const shapeObserverRoundRunRow = (row) => {
    const source = asRecord(row);
    return {
      runId: String(source.run_id || '').trim(),
      lane: String(source.lane || 'none').trim() || 'none',
      profile: String(source.profile || 'unknown').trim() || 'unknown',
      observedFulfillmentModes: dedupeStrings(source.observed_fulfillment_modes),
      observedCategoryIds: dedupeStrings(source.observed_category_ids),
      monitoringEventCount: Number(source.monitoring_event_count || 0),
      defenseDeltaCount: Number(source.defense_delta_count || 0),
      banOutcomeCount: Number(source.ban_outcome_count || 0)
    };
  };

  const shapeObserverRoundSurfaceRow = (row) => {
    const source = asRecord(row);
    return {
      key: `${String(source.run_id || 'unknown').trim()}:${String(source.surface_id || 'surface').trim()}`,
      runId: String(source.run_id || '').trim(),
      surfaceId: String(source.surface_id || '').trim(),
      surfaceLabel: humanizeToken(source.surface_id),
      surfaceState: String(source.surface_state || '').trim(),
      coverageStatus: String(source.coverage_status || '').trim(),
      successContract: String(source.success_contract || '').trim(),
      dependencyKind: String(source.dependency_kind || '').trim(),
      dependencySurfaceIds: dedupeStrings(source.dependency_surface_ids),
      attemptCount: Number(source.attempt_count || 0),
      sampleRequestMethod: String(source.sample_request_method || '').trim(),
      sampleRequestPath: String(source.sample_request_path || '').trim(),
      sampleResponseStatus: asFiniteNumber(source.sample_response_status)
    };
  };

  const shapeRecentRunSurfaceRow = (runId, receipt) => {
    const source = asRecord(receipt);
    return {
      key: `${String(runId || 'unknown').trim()}:${String(source.surfaceId || 'surface').trim()}`,
      runId: String(runId || '').trim(),
      surfaceId: String(source.surfaceId || '').trim(),
      surfaceLabel: String(source.surfaceLabel || '').trim() || humanizeToken(source.surfaceId),
      surfaceState: String(source.surfaceState || '').trim(),
      coverageStatus: String(source.coverageStatus || '').trim(),
      successContract: String(source.successContract || '').trim(),
      dependencyKind: String(source.dependencyKind || '').trim(),
      dependencySurfaceIds: dedupeStrings(source.dependencySurfaceIds),
      attemptCount: Number(source.attemptCount || 0),
      sampleRequestMethod: String(source.sampleRequestMethod || '').trim(),
      sampleRequestPath: String(source.sampleRequestPath || '').trim(),
      sampleResponseStatus: asFiniteNumber(source.sampleResponseStatus)
    };
  };

  const shapeRecentRunSurfaceRows = (run) =>
    [
      ...toArray(run?.ownedSurfaceCoverage?.receipts).map((receipt) =>
        shapeRecentRunSurfaceRow(run?.runId, receipt)
      ),
      ...deriveLlmSurfaceRowsFromRuntimeSummary(run?.llmRuntimeSummary, run?.runId)
    ];

  const mergeSurfaceState = (current, next) => {
    const normalizedCurrent = String(current || '').trim();
    const normalizedNext = String(next || '').trim();
    if (normalizedCurrent === 'leaked' || normalizedNext === 'leaked') return 'leaked';
    if (normalizedCurrent === 'held' || normalizedNext === 'held') return 'held';
    return normalizedCurrent || normalizedNext;
  };

  const dedupeSurfaceRows = (rows) => {
    const rowsByKey = new Map();
    toArray(rows).forEach((row) => {
      const source = asRecord(row);
      const key = String(
        source.key ||
        `${String(source.runId || source.run_id || 'unknown').trim()}:${String(source.surfaceId || source.surface_id || 'surface').trim()}`
      ).trim();
      if (!key) return;
      const nextDependencySurfaceIds = dedupeStrings(
        source.dependencySurfaceIds || source.dependency_surface_ids
      );
      const nextAttemptCount = Number(source.attemptCount || source.attempt_count || 0);
      const nextSampleResponseStatus = asFiniteNumber(
        source.sampleResponseStatus ?? source.sample_response_status
      );
      if (!rowsByKey.has(key)) {
        rowsByKey.set(key, {
          key,
          runId: String(source.runId || source.run_id || '').trim(),
          surfaceId: String(source.surfaceId || source.surface_id || '').trim(),
          surfaceLabel: String(source.surfaceLabel || '').trim(),
          surfaceState: String(source.surfaceState || source.surface_state || '').trim(),
          coverageStatus: String(source.coverageStatus || source.coverage_status || '').trim(),
          successContract: String(source.successContract || source.success_contract || '').trim(),
          dependencyKind: String(source.dependencyKind || source.dependency_kind || '').trim(),
          dependencySurfaceIds: nextDependencySurfaceIds,
          attemptCount: nextAttemptCount,
          sampleRequestMethod: String(
            source.sampleRequestMethod || source.sample_request_method || ''
          ).trim(),
          sampleRequestPath: String(source.sampleRequestPath || source.sample_request_path || '').trim(),
          sampleResponseStatus: nextSampleResponseStatus
        });
        return;
      }
      const existing = rowsByKey.get(key);
      existing.surfaceLabel = existing.surfaceLabel || String(source.surfaceLabel || '').trim();
      existing.surfaceState = mergeSurfaceState(
        existing.surfaceState,
        source.surfaceState || source.surface_state
      );
      existing.coverageStatus =
        existing.coverageStatus || String(source.coverageStatus || source.coverage_status || '').trim();
      existing.successContract =
        existing.successContract || String(source.successContract || source.success_contract || '').trim();
      existing.dependencyKind =
        existing.dependencyKind || String(source.dependencyKind || source.dependency_kind || '').trim();
      existing.dependencySurfaceIds = dedupeStrings([
        ...existing.dependencySurfaceIds,
        ...nextDependencySurfaceIds
      ]);
      existing.attemptCount += nextAttemptCount;
      if (!existing.sampleRequestMethod) {
        existing.sampleRequestMethod = String(
          source.sampleRequestMethod || source.sample_request_method || ''
        ).trim();
      }
      if (!existing.sampleRequestPath) {
        existing.sampleRequestPath = String(
          source.sampleRequestPath || source.sample_request_path || ''
        ).trim();
      }
      if (existing.sampleResponseStatus === null) {
        existing.sampleResponseStatus = nextSampleResponseStatus;
      }
    });
    return Array.from(rowsByKey.values());
  };

  $: oversightHistoryRows = toArray(oversightHistory?.rows);
  $: historyRowByEpisodeId = new Map(
    oversightHistoryRows
      .flatMap((row) => {
        const keys = dedupeStrings([row?.episode_id, row?.decision_id]);
        return keys.map((key) => [key, row]);
      })
      .filter(([key]) => key)
  );
  $: candidateWindowStatus = asRecord(oversightAgentStatus?.candidate_window);
  $: continuationRunStatus = asRecord(oversightAgentStatus?.continuation_run);
  $: episodeArchive = asRecord(
    oversightHistory?.episode_archive?.schema_version
      ? oversightHistory?.episode_archive
      : oversightAgentStatus?.episode_archive
  );
  $: episodeArchiveRows = toArray(episodeArchive?.rows);
  $: judgedCycleRows = episodeArchiveRows.filter((row) => {
    const outcome = String(row?.retain_or_rollback || '').trim();
    return outcome === 'retained' || outcome === 'rolled_back';
  });
  $: latestJudgedEpisodeRow = judgedCycleRows[0] || null;
  $: currentMixedEvidenceSource =
    toArray(candidateWindowStatus?.required_runs).length > 0
      ? 'candidate_window'
      : toArray(continuationRunStatus?.required_runs).length > 0
        ? 'continuation_run'
        : '';
  $: currentMixedEvidenceRuns =
    currentMixedEvidenceSource === 'candidate_window'
      ? toArray(candidateWindowStatus?.required_runs)
      : currentMixedEvidenceSource === 'continuation_run'
        ? toArray(continuationRunStatus?.required_runs)
        : [];
  $: currentMixedEvidenceSourceLabel =
    currentMixedEvidenceSource === 'candidate_window'
      ? 'Candidate window'
      : currentMixedEvidenceSource === 'continuation_run'
        ? 'Loop continuation'
        : 'No active mixed-attacker evidence set';
  $: currentMixedEvidenceStatus =
    currentMixedEvidenceSource === 'candidate_window'
      ? candidateWindowStatus?.status
      : currentMixedEvidenceSource === 'continuation_run'
        ? continuationRunStatus?.status
        : '';
  $: currentMixedEvidenceLaneSummary = summarizeRequiredRuns(currentMixedEvidenceRuns);

  $: recognitionEvaluation = asRecord(operatorSnapshot?.non_human_traffic?.recognition_evaluation);
  $: recognitionComparisonRows = toArray(recognitionEvaluation?.comparison_rows);
  $: recognitionComparisonByCategoryId = new Map(
    recognitionComparisonRows
      .map((row) => [String(row?.category_id || '').trim(), row])
      .filter(([categoryId]) => categoryId)
  );
  $: simulatorGroundTruth = asRecord(recognitionEvaluation?.simulator_ground_truth);
  $: simulatorGroundTruthCategoryRows = toArray(simulatorGroundTruth?.categories);
  $: simulatorGroundTruthByCategoryId = new Map(
    simulatorGroundTruthCategoryRows
      .map((row) => [String(row?.category_id || '').trim(), row])
      .filter(([categoryId]) => categoryId)
  );

  $: observerRoundArchive = asRecord(oversightHistory?.observer_round_archive);
  $: observerRoundArchiveRows = toArray(observerRoundArchive?.rows);
  $: observerRoundByEpisodeId = new Map(
    observerRoundArchiveRows
      .map((row) => [String(row?.episode_id || '').trim(), row])
      .filter(([episodeId]) => episodeId)
  );
  $: selectedObserverRound =
    observerRoundByEpisodeId.get(String(latestJudgedEpisodeRow?.episode_id || '').trim()) || null;
  $: selectedObserverRoundMissing = Boolean(latestJudgedEpisodeRow && !selectedObserverRound);
  $: selectedRoundMissingRunIds = dedupeStrings(selectedObserverRound?.missing_run_ids);
  $: selectedRoundLaneRunRows = toArray(selectedObserverRound?.run_rows).map(shapeObserverRoundRunRow);

  $: recentObservedRunRows = deriveAdversaryRunRowsFromSummaries(
    toArray(operatorSnapshot?.adversary_sim?.recent_runs),
    []
  ).runRows;
  $: recentObservedRunById = new Map(
    recentObservedRunRows
      .map((run) => [String(run?.runId || '').trim(), run])
      .filter(([runId]) => runId)
  );
  $: currentMixedEvidenceRunIds = dedupeStrings(
    currentMixedEvidenceRuns.map((run) => String(run?.follow_on_run_id || '').trim())
  );
  $: currentMixedEvidenceObservedRunRows = currentMixedEvidenceRunIds
    .map((runId) => recentObservedRunById.get(runId) || null)
    .filter(Boolean);
  $: currentMixedEvidenceMissingRunIds = currentMixedEvidenceRunIds.filter(
    (runId) => !recentObservedRunById.has(runId)
  );
  $: currentMixedEvidenceSurfaceRows = currentMixedEvidenceObservedRunRows.flatMap((run) =>
    shapeRecentRunSurfaceRows(run)
  );
  $: latestRecentObservedRun = recentObservedRunRows[0] || null;
  $: latestRecentObservedRunTs =
    asFiniteNumber(latestRecentObservedRun?.lastTs) ??
    asFiniteNumber(latestRecentObservedRun?.firstTs) ??
    0;
  $: latestJudgedEpisodeCompletedAt = asFiniteNumber(latestJudgedEpisodeRow?.completed_at_ts) ?? 0;
  $: latestRecentRunOutranksJudgedRound = Boolean(
    latestRecentObservedRun &&
      (!latestJudgedEpisodeRow || latestRecentObservedRunTs > latestJudgedEpisodeCompletedAt)
  );
  $: latestRecentObservedRunSurfaceRows = latestRecentObservedRun
    ? shapeRecentRunSurfaceRows(latestRecentObservedRun)
    : [];
  $: selectedRoundCastContext = (() => {
    if (currentMixedEvidenceObservedRunRows.length > 0) {
      return {
        sourceKind: 'current_mixed_evidence',
        sourceText: `Showing current mixed-attacker evidence: ${currentMixedEvidenceLaneSummary}.`,
        runRows: currentMixedEvidenceObservedRunRows,
        surfaceRows: currentMixedEvidenceSurfaceRows,
        missingRunIds: currentMixedEvidenceMissingRunIds,
        archiveMissing: false
      };
    }
    if (latestRecentRunOutranksJudgedRound && latestRecentObservedRun) {
      return {
        sourceKind: 'latest_recent_run',
        sourceText: latestJudgedEpisodeRow
          ? `Showing the latest exact recent sim run: ${formatAdversarySimLaneLabel(latestRecentObservedRun.lane, humanizeToken(latestRecentObservedRun.lane))}. Judged history remains above.`
          : `Showing the latest exact recent sim run: ${formatAdversarySimLaneLabel(latestRecentObservedRun.lane, humanizeToken(latestRecentObservedRun.lane))}.`,
        runRows: [latestRecentObservedRun],
        surfaceRows: latestRecentObservedRunSurfaceRows,
        missingRunIds: [],
        archiveMissing: false
      };
    }
    if (selectedObserverRound) {
      return {
        sourceKind: 'latest_judged_round',
        sourceText: 'Showing the latest completed judged round.',
        runRows: selectedRoundLaneRunRows,
        surfaceRows: [
          ...toArray(selectedObserverRound?.scrapling_surface_rows).map(
            shapeObserverRoundSurfaceRow
          ),
          ...toArray(selectedObserverRound?.llm_surface_rows).map(shapeObserverRoundSurfaceRow)
        ],
        missingRunIds: selectedRoundMissingRunIds,
        archiveMissing: false
      };
    }
    if (selectedObserverRoundMissing) {
      return {
        sourceKind: 'judged_archive_missing',
        sourceText: 'The latest judged round is recorded, but its durable observer archive is still unavailable.',
        runRows: [],
        surfaceRows: [],
        missingRunIds: [],
        archiveMissing: true
      };
    }
    return {
      sourceKind: 'none',
      sourceText: '',
      runRows: [],
      surfaceRows: [],
      missingRunIds: [],
      archiveMissing: false
    };
  })();
  $: selectedRoundCastSourceText = String(selectedRoundCastContext?.sourceText || '').trim();
  $: selectedRoundCastMissingRunIds = dedupeStrings(selectedRoundCastContext?.missingRunIds);
  $: selectedRoundCastArchiveMissing = selectedRoundCastContext?.archiveMissing === true;
  $: selectedRoundCastRunRows = toArray(selectedRoundCastContext?.runRows);
  $: selectedRoundCastSurfaceRows = dedupeSurfaceRows(selectedRoundCastContext?.surfaceRows);

  $: recentRoundRows = judgedCycleRows.slice(0, 4).map((row, index) => {
    const historyRow =
      historyRowByEpisodeId.get(String(row?.episode_id || '').trim()) ||
      historyRowByEpisodeId.get(String(row?.decision_id || '').trim()) ||
      null;
    return {
      episodeId: String(row?.episode_id || historyRow?.decision_id || `round-${index}`),
      completedText: formatTimestamp(row?.completed_at_ts || historyRow?.recorded_at_ts),
      lanesText: formatLaneList(row?.judged_lane_ids, 'No judged lanes recorded'),
      resultText: formatRoundResultSummary(row),
      moveText: formatRoundMoveSummary(row, historyRow),
      nextText: formatRoundNextState(
        row,
        index === 0,
        currentMixedEvidenceSourceLabel,
        currentMixedEvidenceStatus
      ),
      noteText: row?.proposal?.note || historyRow?.summary || ''
    };
  });

  $: adversaryCastRows = (() => {
    const rows = [];
    const seen = new Set();
    selectedRoundCastRunRows.forEach((run) => {
      const categoryIds = dedupeStrings(run?.observedCategoryIds);
      if (categoryIds.length === 0) {
        const key = `${String(run?.runId || run?.lane || 'unknown')}:observer-category-unavailable`;
        if (!seen.has(key)) {
          seen.add(key);
          rows.push({
            key,
            categoryLabel: 'Category truth unavailable',
            laneText: formatAdversarySimLaneLabel(run?.lane, humanizeToken(run?.lane)),
            activityText: formatObservedRunSummary(run),
            shumaCallText: 'Recent recognition evaluation unavailable',
            recognitionText: 'not materialized',
            noteText:
              'This exact observer run did not preserve a lane-owned simulator category label, so the page will not guess one.'
          });
        }
        return;
      }
      categoryIds.forEach((categoryId) => {
        const key = `${String(run?.runId || run?.lane || 'unknown')}:${categoryId || 'uncategorized'}`;
        if (seen.has(key)) return;
        seen.add(key);
        const comparison = recognitionComparisonByCategoryId.get(categoryId) || null;
        const groundTruth = simulatorGroundTruthByCategoryId.get(categoryId) || null;
        rows.push({
          key,
          categoryLabel:
            groundTruth?.category_label ||
            comparison?.category_label ||
            humanizeToken(categoryId || run?.lane),
          laneText: formatAdversarySimLaneLabel(run?.lane, humanizeToken(run?.lane)),
          activityText: formatObservedRunSummary(run),
          shumaCallText: comparison?.inferred_category_label
            ? `Recent recognition evaluation inferred ${comparison.inferred_category_label}`
            : 'Recent recognition evaluation not materialized',
          recognitionText: comparison
            ? comparisonStatusText(comparison.comparison_status)
            : 'not materialized',
          noteText:
            comparison?.note ||
            (groundTruth?.evidence_references?.length
              ? `Simulator ground truth preserved for ${formatNumber(groundTruth.recent_run_count, '0')} recent run(s).`
              : '')
        });
      });
    });
    return rows;
  })();

  $: defenceCastRows = selectedRoundCastSurfaceRows.map((row) => ({
    key: row.key,
    surfaceLabel: row.surfaceLabel || humanizeToken(row.surfaceId),
    observationText: formatReceiptObservation(row),
    outcomeText: row.surfaceState
      ? humanizeToken(row.surfaceState, 'sentence')
      : 'state unavailable',
    noteText: dedupeStrings([
      row.coverageStatus ? `coverage ${humanizeToken(row.coverageStatus, 'sentence')}` : '',
      row.successContract ? `success contract ${humanizeToken(row.successContract, 'sentence')}` : '',
      row.dependencyKind ? `${humanizeToken(row.dependencyKind, 'sentence')} surface` : ''
    ]).join(' | ') || 'No additional surface detail.'
  }));
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
      <h2>{section.title}</h2>
      <p class="section-desc text-muted">{section.description}</p>

      {#if section.id === 'recent-rounds'}
        <div id="game-loop-round-history" class="panel panel-soft pad-md">
          <p class="caps-label">Recent Rounds</p>
          <p class="text-muted">
            Each row is a judged round: who played, what move was tested, and whether the loop kept going.
          </p>
          {#if recentRoundRows.length === 0}
            <p class="text-muted">No completed judged rounds are materialized yet.</p>
          {:else}
            <ul class="metric-list">
              {#each recentRoundRows as row (row.episodeId)}
                <li>
                  <strong>{row.completedText}</strong>: {row.lanesText}
                  <br />
                  <span class="text-muted">{row.resultText} | {row.moveText} | {row.nextText}</span>
                  {#if row.noteText}
                    <br />
                    <span class="text-muted">{row.noteText}</span>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else if section.id === 'adversary-cast'}
        <div id="game-loop-adversary-cast" class="panel panel-soft pad-md">
          <p class="caps-label">Adversaries In This Round</p>
          <p class="text-muted">
            The simulator ground truth is visible here after the round. Runtime defences still never read those labels directly.
          </p>
          {#if selectedRoundCastSourceText}
            <p class="text-muted">{selectedRoundCastSourceText}</p>
          {/if}
          {#if selectedRoundCastMissingRunIds.length > 0}
            <p class="text-muted">
              Missing exact run receipts: {selectedRoundCastMissingRunIds.join(', ')}.
            </p>
          {/if}
          {#if adversaryCastRows.length === 0}
            <p class="text-muted">
              {#if selectedRoundCastArchiveMissing}
                The judged round is recorded, but no durable observer-round archive was materialized for it yet.
              {:else if selectedRoundCastMissingRunIds.length}
                The current observer evidence is only partially materialized, so the page will not guess the missing adversaries.
              {:else}
                No recent adversary cast is materialized yet.
              {/if}
            </p>
          {:else}
            <ul class="metric-list">
              {#each adversaryCastRows as row (row.key)}
                <li>
                  <strong>{row.categoryLabel}</strong>
                  {#if row.laneText}
                    via {row.laneText}
                  {/if}
                  <br />
                  <span class="text-muted">{row.activityText}</span>
                  <br />
                  <span class="text-muted">{row.shumaCallText} | {row.recognitionText}</span>
                  {#if row.noteText}
                    <br />
                    <span class="text-muted">{row.noteText}</span>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else if section.id === 'defence-cast'}
        <div id="game-loop-defence-cast" class="panel panel-soft pad-md">
          <p class="caps-label">Defences In This Round</p>
          <p class="text-muted">
            This stays surface-native. It shows what each defence observed and how that surface fared, without simulator labels becoming defence truth.
          </p>
          {#if selectedRoundCastSourceText}
            <p class="text-muted">{selectedRoundCastSourceText}</p>
          {/if}
          {#if selectedRoundCastMissingRunIds.length > 0}
            <p class="text-muted">
              Missing exact run receipts: {selectedRoundCastMissingRunIds.join(', ')}.
            </p>
          {/if}
          {#if defenceCastRows.length === 0}
            <p class="text-muted">
              {#if selectedRoundCastArchiveMissing}
                The judged round is recorded, but no durable observer-round archive was materialized for it yet.
              {:else if selectedRoundCastMissingRunIds.length}
                The current observer evidence is only partially materialized, so the page will not guess the missing defence cast.
              {:else}
                No defence-surface view is materialized for the selected round yet.
              {/if}
            </p>
          {:else}
            <ul class="metric-list">
              {#each defenceCastRows as row (row.key)}
                <li>
                  <strong>{row.surfaceLabel}</strong>
                  <br />
                  <span class="text-muted">{row.observationText}</span>
                  <br />
                  <span class="text-muted">{row.outcomeText}</span>
                  {#if row.noteText}
                    <br />
                    <span class="text-muted">{row.noteText}</span>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </section>
  {/each}
</section>
