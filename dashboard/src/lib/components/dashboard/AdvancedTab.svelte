<script>
  import { onDestroy, onMount } from 'svelte';
  import ConfigAdvancedSection from './config/ConfigAdvancedSection.svelte';
  import ConfigExportSection from './config/ConfigExportSection.svelte';
  import SaveChangesBar from './primitives/SaveChangesBar.svelte';
  import TabStateMessage from './primitives/TabStateMessage.svelte';
  import { advancedConfigTemplatePaths } from '../../domain/config-schema.js';
  import {
    buildJsonFieldLineMap,
    buildTemplateFromPaths,
    normalizeJsonObjectForCompare,
    parseJsonObjectWithDiagnostics,
    resolveJsonFieldLine
  } from '../../domain/core/json-object.js';
  import {
    buildVariableInventoryGroups,
    deriveStatusSnapshot
  } from '../../domain/status.js';
  import { isAdminConfigWritable } from '../../domain/config-runtime.js';
  import { resolveDashboardAssetPath } from '../../runtime/dashboard-paths.js';

  export let managed = false;
  export let isActive = false;
  export let tabStatus = null;
  export let configSnapshot = null;
  export let configRuntimeSnapshot = null;
  export let configVersion = 0;
  export let dashboardBasePath = '/dashboard';
  export let onSaveConfig = null;
  export let onValidateConfig = null;
  export let noticeText = '';
  export let noticeKind = 'info';

  const STATUS_VAR_MEANINGS_ASSET = 'assets/status-var-meanings.json';
  const ADVANCED_VALIDATE_DEBOUNCE_MS = 350;
  const EXPORT_STATUS_RESET_MS = 4000;
  const EMPTY_VAR_MEANINGS = {};

  let statusVarMeanings = EMPTY_VAR_MEANINGS;
  let writable = false;
  let hasConfigSnapshot = false;
  let lastAppliedConfigVersion = -1;
  let deferredConfigApply = false;
  let savingAdvanced = false;
  let warnOnUnload = false;

  let advancedConfigJson = '{}';
  let advancedValidationPending = false;
  let advancedValidationError = '';
  let advancedValidationIssues = [];
  let advancedParseIssue = null;
  let advancedFieldLineMap = new Map();
  let advancedDisplayValidationIssues = [];
  let advancedValidationTimer = null;
  let advancedValidationRequestId = 0;
  let baselineAdvancedNormalized = '{}';
  let exportConfigStatus = '';
  let exportConfigStatusKind = 'info';
  let exportConfigStatusTimer = null;

  function normalizeStatusVarMeanings(value) {
    if (!value || typeof value !== 'object') return EMPTY_VAR_MEANINGS;
    return value;
  }

  async function loadStatusVarMeanings() {
    if (typeof window === 'undefined') return;
    try {
      const response = await fetch(
        resolveDashboardAssetPath(dashboardBasePath, STATUS_VAR_MEANINGS_ASSET),
        { credentials: 'same-origin' }
      );
      if (!response.ok) return;
      const parsed = await response.json();
      statusVarMeanings = normalizeStatusVarMeanings(parsed);
    } catch (_error) {}
  }

  const handleBeforeUnload = (event) => {
    if (!warnOnUnload) return;
    event.preventDefault();
    event.returnValue = '';
  };

  const clearExportStatusTimer = () => {
    if (exportConfigStatusTimer) {
      clearTimeout(exportConfigStatusTimer);
      exportConfigStatusTimer = null;
    }
  };

  const scheduleExportStatusReset = () => {
    clearExportStatusTimer();
    exportConfigStatusTimer = setTimeout(() => {
      exportConfigStatus = '';
      exportConfigStatusKind = 'info';
      exportConfigStatusTimer = null;
    }, EXPORT_STATUS_RESET_MS);
  };

  const clearAdvancedValidationTimer = () => {
    if (advancedValidationTimer) {
      clearTimeout(advancedValidationTimer);
      advancedValidationTimer = null;
    }
  };

  const resetAdvancedValidationState = () => {
    clearAdvancedValidationTimer();
    advancedValidationPending = false;
    advancedValidationError = '';
    advancedValidationIssues = [];
  };

  const normalizeAdvancedValidationIssues = (issues, fieldLineMap = new Map()) => {
    if (!Array.isArray(issues)) return [];
    return issues
      .filter((issue) => issue && typeof issue === 'object')
      .map((issue) => {
        const source = /** @type {Record<string, unknown>} */ (issue);
        const field = typeof source.field === 'string' ? source.field : '';
        const sourceLine = Number(source.line);
        const resolvedLine = Number.isInteger(sourceLine) && sourceLine > 0
          ? sourceLine
          : resolveJsonFieldLine(field, fieldLineMap);
        const sourceColumn = Number(source.column);
        return {
          field,
          message: typeof source.message === 'string' ? source.message : 'Invalid value.',
          expected: typeof source.expected === 'string' ? source.expected : '',
          received: Object.prototype.hasOwnProperty.call(source, 'received')
            ? source.received
            : undefined,
          line: Number.isInteger(resolvedLine) && resolvedLine > 0 ? resolvedLine : null,
          column: Number.isInteger(sourceColumn) && sourceColumn > 0 ? sourceColumn : null
        };
      });
  };

  async function runAdvancedServerValidation(advancedPatch, requestId, fieldLineMap) {
    if (typeof onValidateConfig !== 'function') {
      if (requestId !== advancedValidationRequestId) return;
      advancedValidationPending = false;
      advancedValidationError = '';
      advancedValidationIssues = [];
      return;
    }

    try {
      const result = await onValidateConfig(advancedPatch);
      if (requestId !== advancedValidationRequestId) return;
      const issues = normalizeAdvancedValidationIssues(result && result.issues, fieldLineMap);
      advancedValidationIssues = issues;
      advancedValidationError = '';
      advancedValidationPending = false;
    } catch (error) {
      if (requestId !== advancedValidationRequestId) return;
      advancedValidationIssues = [];
      advancedValidationPending = false;
      advancedValidationError = error && error.message
        ? String(error.message)
        : 'Unable to validate Advanced JSON right now.';
    }
  }

  function applyConfig(config = {}, runtime = {}) {
    hasConfigSnapshot = config && typeof config === 'object' && Object.keys(config).length > 0;
    writable = isAdminConfigWritable(runtime);

    const advancedTemplate = buildTemplateFromPaths(config, advancedConfigTemplatePaths || []);
    const advancedText = JSON.stringify(advancedTemplate, null, 2);
    advancedConfigJson = advancedText;
    baselineAdvancedNormalized = normalizeJsonObjectForCompare(advancedText) || '{}';

    resetAdvancedValidationState();
    clearExportStatusTimer();
    exportConfigStatus = '';
    exportConfigStatusKind = 'info';
  }

  const parseAdvancedPatchObject = () => {
    const parsed = parseJsonObjectWithDiagnostics(advancedConfigJson);
    if (!parsed.ok || !parsed.value) {
      const parseMessage = parsed.issue && typeof parsed.issue.message === 'string'
        ? parsed.issue.message
        : 'Advanced config JSON patch must be an object.';
      throw new Error(parseMessage);
    }
    return { ...parsed.value };
  };

  async function saveAdvancedConfig() {
    if (saveAdvancedDisabled || typeof onSaveConfig !== 'function') return;
    const patch = parseAdvancedPatchObject();
    if (Object.keys(patch).length === 0) return;

    savingAdvanced = true;
    try {
      const nextConfig = await onSaveConfig(patch, { successMessage: 'Advanced config saved' });
      if (nextConfig && typeof nextConfig === 'object') {
        applyConfig(
          nextConfig,
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    } finally {
      savingAdvanced = false;
    }
  }

  const downloadJsonFile = (filename, payload) => {
    if (typeof window === 'undefined' || typeof document === 'undefined') return false;
    const blob = new Blob([payload], { type: 'application/json' });
    const url = window.URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = filename;
    anchor.rel = 'noopener';
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.URL.revokeObjectURL(url);
    return true;
  };

  async function exportCurrentConfigJson(event) {
    if (event && typeof event.preventDefault === 'function') {
      event.preventDefault();
    }
    if (exportConfigDisabled) return;

    try {
      const text = JSON.stringify(advancedParseResult.value || {}, null, 2);
      const stamp = new Date().toISOString().replace(/[:.]/g, '-');
      const filename = `shuma-config-${stamp}.json`;
      const downloaded = downloadJsonFile(filename, text);
      let copied = false;
      if (
        typeof window !== 'undefined' &&
        window.isSecureContext === true &&
        typeof navigator !== 'undefined' &&
        navigator.clipboard
      ) {
        try {
          await navigator.clipboard.writeText(text);
          copied = true;
        } catch (_error) {}
      }

      if (downloaded && copied) {
        exportConfigStatus = 'Exported config JSON downloaded and copied to clipboard.';
      } else if (downloaded) {
        exportConfigStatus = 'Exported config JSON downloaded.';
      } else if (copied) {
        exportConfigStatus = 'Exported config JSON copied to clipboard.';
      } else {
        exportConfigStatus = 'Exported config JSON generated.';
      }
      exportConfigStatusKind = 'success';
      scheduleExportStatusReset();
    } catch (error) {
      exportConfigStatus = error && error.message
        ? error.message
        : 'Failed to export config JSON.';
      exportConfigStatusKind = 'error';
      scheduleExportStatusReset();
    }
  }

  onMount(() => {
    if (typeof window === 'undefined') return undefined;
    window.addEventListener('beforeunload', handleBeforeUnload);
    void loadStatusVarMeanings();
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  });

  onDestroy(() => {
    clearAdvancedValidationTimer();
    clearExportStatusTimer();
  });

  $: statusSnapshot = deriveStatusSnapshot(configSnapshot || {}, configRuntimeSnapshot || {});
  $: statusVariableGroups = buildVariableInventoryGroups(statusSnapshot, {
    varMeanings: statusVarMeanings
  });

  $: advancedParseResult = parseJsonObjectWithDiagnostics(advancedConfigJson);
  $: advancedNormalized = advancedParseResult.ok
    ? advancedParseResult.normalized
    : normalizeJsonObjectForCompare(advancedConfigJson);
  $: advancedShapeValid = advancedNormalized !== null;
  $: advancedParseIssue = advancedShapeValid ? null : advancedParseResult.issue;
  $: advancedFieldLineMap = advancedShapeValid
    ? buildJsonFieldLineMap(advancedConfigJson)
    : new Map();
  $: advancedDisplayValidationIssues = !advancedShapeValid && advancedParseIssue
    ? [advancedParseIssue]
    : advancedValidationIssues;
  $: advancedDirty = advancedShapeValid && advancedNormalized !== baselineAdvancedNormalized;
  $: advancedValid = advancedShapeValid
    && !advancedValidationPending
    && advancedValidationError === ''
    && advancedValidationIssues.length === 0;
  $: advancedInvalidMessage = !advancedShapeValid
    ? (advancedParseIssue && typeof advancedParseIssue.message === 'string'
      ? advancedParseIssue.message
      : 'Advanced JSON must be a valid JSON object.')
    : (advancedValidationError
      ? `Advanced JSON validation failed: ${advancedValidationError}`
      : (advancedValidationIssues.length > 0 ? 'Advanced JSON has invalid values.' : ''));

  $: {
    clearAdvancedValidationTimer();
    advancedValidationRequestId += 1;
    const requestId = advancedValidationRequestId;

    if (!writable || typeof onValidateConfig !== 'function' || !advancedDirty) {
      advancedValidationPending = false;
      advancedValidationError = '';
      advancedValidationIssues = [];
    } else if (!advancedShapeValid) {
      advancedValidationPending = false;
      advancedValidationError = '';
      advancedValidationIssues = [];
    } else {
      if (advancedParseResult.ok && advancedParseResult.value) {
        const advancedPatch = { ...advancedParseResult.value };
        const fieldLineMap = new Map(advancedFieldLineMap);
        advancedValidationPending = true;
        advancedValidationError = '';
        advancedValidationIssues = [];
        advancedValidationTimer = setTimeout(() => {
          void runAdvancedServerValidation(advancedPatch, requestId, fieldLineMap);
        }, ADVANCED_VALIDATE_DEBOUNCE_MS);
      } else {
        advancedValidationPending = false;
        advancedValidationError = '';
        advancedValidationIssues = [];
      }
    }
  }

  $: hasUnsavedChanges = advancedDirty;
  $: saveAdvancedDisabled = !writable || !hasUnsavedChanges || !advancedValid || savingAdvanced;
  $: saveAdvancedLabel = savingAdvanced ? 'Saving...' : 'Save advanced changes';
  $: saveAdvancedSummaryText = hasUnsavedChanges
    ? 'Advanced JSON has unsaved changes'
    : 'No unsaved changes';
  $: saveAdvancedInvalidText = !advancedValid
    ? 'Fix invalid Advanced JSON values before saving.'
    : '';
  $: exportConfigDisabled = !hasConfigSnapshot || !advancedShapeValid;
  $: warnOnUnload = writable && hasUnsavedChanges;

  $: {
    const nextVersion = Number(configVersion || 0);
    if (nextVersion !== lastAppliedConfigVersion) {
      lastAppliedConfigVersion = nextVersion;
      if (hasUnsavedChanges && !savingAdvanced) {
        deferredConfigApply = true;
      } else {
        applyConfig(
          configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
          configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
        );
      }
    }
  }

  $: if (deferredConfigApply && !hasUnsavedChanges && !savingAdvanced) {
    deferredConfigApply = false;
    applyConfig(
      configSnapshot && typeof configSnapshot === 'object' ? configSnapshot : {},
      configRuntimeSnapshot && typeof configRuntimeSnapshot === 'object' ? configRuntimeSnapshot : {}
    );
  }
</script>

<section
  id="dashboard-panel-advanced"
  class="admin-group"
  data-dashboard-tab-panel="advanced"
  aria-labelledby="dashboard-tab-advanced"
  hidden={managed ? !isActive : false}
  aria-hidden={managed ? (isActive ? 'false' : 'true') : 'true'}
>
  <TabStateMessage tab="advanced" status={tabStatus} noticeText={noticeText} noticeKind={noticeKind} />
  <div class="controls-grid controls-grid--config">
    <div class="control-group panel-soft pad-md status-inventory-group">
      <h3>Runtime Variable Inventory</h3>
      <p class="control-desc text-muted">
        Complete runtime snapshot of active configuration variables, grouped by concern.
        Rows with highlighted background are runtime admin-writable variables.
      </p>
      <div id="status-vars-groups" class="status-var-groups">
        {#if statusVariableGroups.length === 0}
          <p class="text-muted">No configuration snapshot loaded yet.</p>
        {:else}
          {#each statusVariableGroups as group}
            <section class="status-var-group">
              <h4 class="status-var-group-title">{@html group.title}</h4>
              <table class="status-vars-table">
                <colgroup>
                  <col class="status-vars-col status-vars-col--variable">
                  <col class="status-vars-col status-vars-col--value">
                  <col class="status-vars-col status-vars-col--meaning">
                </colgroup>
                <thead>
                  <tr>
                    <th scope="col">Variable</th>
                    <th scope="col">Current Value</th>
                    <th scope="col">Meaning</th>
                  </tr>
                </thead>
                <tbody>
                  {#each group.entries as entry}
                    <tr class={`status-var-row ${entry.isAdminWrite ? 'status-var-row--admin-write' : ''}`.trim()}>
                      <td><code>{entry.path}</code></td>
                      <td><code>{entry.valueText}</code></td>
                      <td>{@html entry.meaning}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </section>
          {/each}
        {/if}
      </div>
    </div>

    <ConfigExportSection
      bind:writable
      bind:exportConfigDisabled
      bind:exportConfigStatus
      bind:exportConfigStatusKind
      onExportCurrentConfigJson={exportCurrentConfigJson}
    />

    <ConfigAdvancedSection
      bind:writable
      bind:advancedDirty
      bind:advancedConfigJson
      bind:advancedValidationPending
      bind:advancedInvalidMessage
      advancedValidationIssues={advancedDisplayValidationIssues}
      bind:advancedValid
    />

    <SaveChangesBar
      containerId="advanced-save-bar"
      isHidden={!writable || !hasUnsavedChanges}
      summaryId="advanced-unsaved-summary"
      summaryText={saveAdvancedSummaryText}
      summaryClass="text-unsaved-changes"
      invalidId="advanced-invalid-summary"
      invalidText={saveAdvancedInvalidText}
      buttonId="save-advanced-config"
      buttonLabel={saveAdvancedLabel}
      buttonDisabled={saveAdvancedDisabled}
      onSave={saveAdvancedConfig}
    />
  </div>
</section>
