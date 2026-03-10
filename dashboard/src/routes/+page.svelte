<script>
  import { onDestroy, onMount } from 'svelte';
  import MonitoringTab from '$lib/components/dashboard/MonitoringTab.svelte';
  import {
    buildDashboardLoginPath,
    dashboardIndexPath,
    normalizeDashboardBasePath,
    resolveDashboardAssetPath
  } from '$lib/runtime/dashboard-paths.js';
  import {
    clearDashboardBodyClasses,
    deriveDashboardBodyClassState,
    syncDashboardBodyClasses
  } from '$lib/runtime/dashboard-body-classes.js';
  import {
    deriveAdversarySimControlState,
    normalizeAdversarySimStatus
  } from '$lib/runtime/dashboard-adversary-sim.js';
  import { createDashboardRouteController } from '$lib/runtime/dashboard-route-controller.js';
  import {
    banDashboardIp,
    controlDashboardAdversarySim,
    getDashboardEvents,
    getDashboardRobotsPreview,
    getDashboardAdversarySimStatus,
    getDashboardSessionState,
    logoutDashboardSession,
    mountDashboardApp,
    refreshDashboardTab,
    setDashboardActiveTab,
    unbanDashboardIp,
    unmountDashboardApp,
    validateDashboardConfigPatch,
    updateDashboardConfig,
    restoreDashboardSession
  } from '$lib/runtime/dashboard-native-runtime.js';
  import {
    createDashboardStore,
    DASHBOARD_TABS,
    normalizeTab
  } from '$lib/state/dashboard-store.js';

  export let data;
  const TAB_LOADING_MESSAGES = Object.freeze({
    monitoring: 'Loading monitoring data...',
    'ip-bans': 'Loading ban list...',
    status: 'Loading status signals...',
    verification: 'Loading verification controls...',
    traps: 'Loading trap controls...',
    advanced: 'Loading advanced controls...',
    'rate-limiting': 'Loading rate limiting controls...',
    geo: 'Loading GEO controls...',
    fingerprinting: 'Loading fingerprinting controls...',
    robots: 'Loading robots policy...',
    tuning: 'Loading tuning values...'
  });
  const AUTO_REFRESH_INTERVAL_MS = 1000;
  const AUTO_REFRESH_TABS = new Set(['monitoring', 'ip-bans']);
  const AUTO_REFRESH_PREF_KEY = 'shuma_dashboard_auto_refresh_v1';
  const DASHBOARD_LOADED_CLASS = 'dashboard-loaded';
  const ACTIVE_DIRTY_CONFIG_SAVE_BAR_SELECTOR =
    '#dashboard-admin-section [data-dashboard-tab-panel][aria-hidden="false"] .config-save-bar:not(.hidden)';

  const fallbackBasePath = normalizeDashboardBasePath();
  const dashboardBasePath = typeof data?.dashboardBasePath === 'string'
    ? data.dashboardBasePath
    : fallbackBasePath;
  const shumaImageSrc = typeof data?.shumaImageSrc === 'string'
    ? data.shumaImageSrc
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/shuma-gorath-pencil.png');
  const faviconHref = resolveDashboardAssetPath(
    dashboardBasePath,
    'assets/shuma-gorath-pencil-closed.png'
  );

  const dashboardStore = createDashboardStore({ initialTab: 'monitoring' });

  let dashboardState = dashboardStore.getState();
  let runtimeTelemetry = dashboardStore.getRuntimeTelemetry();
  let storeUnsubscribe = () => {};
  let telemetryUnsubscribe = () => {};
  let runtimeReady = false;
  let dashboardLoaded = false;
  let runtimeError = '';
  let loggingOut = false;
  let suppressBeforeUnloadPrompt = false;
  let savingGlobalTestMode = false;
  let savingGlobalAdversarySim = false;
  let autoRefreshEnabled = false;
  let authExpiryTimer = null;
  let authExpiryAtSeconds = 0;
  let adminMessageText = '';
  let adminMessageKind = 'info';
  let adversarySimStatus = {};
  let adversarySimStatusPollTimer = null;
  let adversarySimStatusRequestInFlight = null;
  let IpBansTabComponent = null;
  let StatusTabComponent = null;
  let VerificationTabComponent = null;
  let TrapsTabComponent = null;
  let AdvancedTabComponent = null;
  let RateLimitingTabComponent = null;
  let GeoTabComponent = null;
  let FingerprintingTabComponent = null;
  let RobotsTabComponent = null;
  let TuningTabComponent = null;
  const tabLinks = {};
  let rootRuntimeClassHint = '';

  function normalizeRuntimeClassHint(value) {
    const normalized = String(value || '').trim().toLowerCase();
    if (normalized === 'runtime-dev' || normalized === 'runtime-prod') {
      return normalized;
    }
    return '';
  }

  function readRootRuntimeClassHint(doc = null) {
    const targetDocument = doc || (typeof document !== 'undefined' ? document : null);
    const classList = targetDocument?.documentElement?.classList;
    if (!classList || typeof classList.contains !== 'function') return '';
    if (classList.contains('runtime-dev')) return 'runtime-dev';
    if (classList.contains('runtime-prod')) return 'runtime-prod';
    return '';
  }

  const describeGlobalControlDisabledState = ({
    runtimeReady,
    loggingOut,
    saving,
    authenticated,
    adminConfigWritable,
    unavailableMessage
  }) => {
    if (runtimeReady !== true) {
      return 'Waiting for the dashboard to finish loading.';
    }
    if (loggingOut === true) {
      return 'Logging out. Controls are temporarily disabled.';
    }
    if (saving === true) {
      return 'A change is already in progress. Wait for it to complete.';
    }
    if (authenticated !== true) {
      return 'Log in to use this control.';
    }
    if (adminConfigWritable !== true) {
      return 'Unavailable because config writes are disabled in this deployment.';
    }
    return String(unavailableMessage || '').trim();
  };

  if (typeof document !== 'undefined') {
    rootRuntimeClassHint = readRootRuntimeClassHint(document);
  }

  $: activeTabKey = normalizeTab(dashboardState.activeTab);
  $: tabStatus = dashboardState?.tabStatus || {};
  $: activeTabStatus = tabStatus[activeTabKey] || {};
  $: autoRefreshSupported = AUTO_REFRESH_TABS.has(activeTabKey);
  $: refreshNowDisabled =
    !runtimeReady || activeTabStatus.loading === true || autoRefreshSupported !== true;
  $: refreshModeText = autoRefreshSupported
    ? (autoRefreshEnabled
      ? `Auto refresh ON (${Math.floor(AUTO_REFRESH_INTERVAL_MS / 1000)}s cadence)`
      : 'Auto refresh OFF (manual)')
    : 'Manual updates only on this tab';
  $: lastUpdatedText = activeTabStatus.updatedAt
    ? `Last updated: ${new Date(activeTabStatus.updatedAt).toLocaleString()}`
    : 'Last updated: not updated yet';
  $: snapshots = dashboardState?.snapshots || {};
  $: snapshotVersions = dashboardState?.snapshotVersions || {};
  $: analyticsSnapshot = snapshots.analytics || {};
  $: configSnapshot = snapshots.config || {};
  $: hasConfigSnapshot = Object.keys(configSnapshot).length > 0;
  $: hasConfigTestMode = typeof configSnapshot.test_mode === 'boolean';
  $: currentTestModeValue = hasConfigTestMode
    ? configSnapshot.test_mode === true
    : analyticsSnapshot.test_mode === true;
  $: testModeEnabled = currentTestModeValue;
  $: backendConnectionState = String(runtimeTelemetry?.connection?.state || 'disconnected')
    .trim()
    .toLowerCase();
  $: backendConnectionTransitionReason = String(runtimeTelemetry?.connection?.lastTransitionReason || '')
    .trim()
    .toLowerCase();
  $: hasConnectionStateSettled = backendConnectionTransitionReason !== 'boot_disconnected';
  $: lostConnectionVisible = dashboardLoaded && backendConnectionState === 'disconnected';
  $: sessionRuntimeClassHint = normalizeRuntimeClassHint(dashboardState?.session?.runtimeEnvironment || '');
  $: runtimeClassHint = sessionRuntimeClassHint || rootRuntimeClassHint;
  $: bodyClassState = deriveDashboardBodyClassState(configSnapshot, {
    backendConnectionState,
    runtimeClassHint
  });
  $: if (typeof document !== 'undefined') {
    syncDashboardBodyClasses(document, bodyClassState);
  }
  $: if (
    typeof document !== 'undefined' &&
    dashboardLoaded !== true &&
    runtimeReady === true &&
    hasConnectionStateSettled === true
  ) {
    const classList = document?.documentElement?.classList;
    if (classList && typeof classList.add === 'function') {
      classList.add(DASHBOARD_LOADED_CLASS);
      dashboardLoaded = true;
    }
  }
  $: normalizedAdversarySimStatus = normalizeAdversarySimStatus(adversarySimStatus);
  $: adversarySimToggleEnabled = normalizedAdversarySimStatus.enabled;
  $: adversarySimControlState = deriveAdversarySimControlState({
    configSnapshot,
    adversarySimStatus
  });
  $: adversarySimRuntimeEnvironment = adversarySimControlState.runtimeEnvironment;
  $: adversarySimSurfaceAvailable = adversarySimControlState.surfaceAvailable;
  $: adversarySimControlAvailable = adversarySimControlState.controlAvailable;
  $: frontierProviderCount = Number.isFinite(Number(configSnapshot.frontier_provider_count))
    ? Math.max(0, Math.floor(Number(configSnapshot.frontier_provider_count)))
    : 0;
  $: globalTestModeToggleDisabled =
    !runtimeReady ||
    loggingOut ||
    savingGlobalTestMode ||
    dashboardState?.session?.authenticated !== true ||
    configSnapshot.admin_config_write_enabled !== true;
  $: globalAdversarySimToggleDisabled =
    !runtimeReady ||
    loggingOut ||
    savingGlobalAdversarySim ||
    dashboardState?.session?.authenticated !== true ||
    configSnapshot.admin_config_write_enabled !== true ||
    !adversarySimControlAvailable;
  $: globalTestModeToggleDisabledReason = globalTestModeToggleDisabled
    ? describeGlobalControlDisabledState({
      runtimeReady,
      loggingOut,
      saving: savingGlobalTestMode,
      authenticated: dashboardState?.session?.authenticated,
      adminConfigWritable: configSnapshot.admin_config_write_enabled,
      unavailableMessage: ''
    })
    : '';
  $: globalAdversarySimToggleDisabledReason = globalAdversarySimToggleDisabled
    ? describeGlobalControlDisabledState({
      runtimeReady,
      loggingOut,
      saving: savingGlobalAdversarySim,
      authenticated: dashboardState?.session?.authenticated,
      adminConfigWritable: configSnapshot.admin_config_write_enabled,
      unavailableMessage: adversarySimControlAvailable
        ? ''
        : 'Unavailable because adversary simulation control requires the simulation surface to be active in this deployment.'
    })
    : '';
  $: adversarySimRetentionHours = Math.max(0, Number(normalizedAdversarySimStatus.historyRetentionHours || 0));
  $: adversarySimCleanupCommand =
    String(normalizedAdversarySimStatus.historyCleanupCommand || '').trim() || 'make telemetry-clean';
  $: adversarySimGenerationDiagnostics = normalizedAdversarySimStatus.generationDiagnostics || {};
  $: adversarySimLifecycleCopy = normalizedAdversarySimStatus.generationActive
        ? (
          String(adversarySimGenerationDiagnostics.health || '') === 'ok'
            ? 'Generation active. Auto-off stops new simulation traffic only; retained telemetry stays visible.'
            : `Generation active. ${
              String(adversarySimGenerationDiagnostics.recommendedAction || '').trim() ||
              'No observable traffic yet. Check supervisor diagnostics for stalled heartbeat state.'
            }`
        )
    : normalizedAdversarySimStatus.historicalDataVisible
      ? `Generation inactive. Retained telemetry remains visible for ${adversarySimRetentionHours}h or until ${adversarySimCleanupCommand} is run.`
      : 'Generation inactive.';

  function registerTabLink(node, tab) {
    let key = normalizeTab(tab);
    tabLinks[key] = node;
    return {
      update(nextTab) {
        delete tabLinks[key];
        key = normalizeTab(nextTab);
        tabLinks[key] = node;
      },
      destroy() {
        delete tabLinks[key];
      }
    };
  }

  function focusTab(tab) {
    const node = tabLinks[normalizeTab(tab)];
    if (node && typeof node.focus === 'function') {
      node.focus();
      return true;
    }
    return false;
  }

  function readAutoRefreshPreference() {
    if (typeof window === 'undefined') return false;
    try {
      return window.localStorage.getItem(AUTO_REFRESH_PREF_KEY) === '1';
    } catch (_error) {
      return false;
    }
  }

  function writeAutoRefreshPreference(nextEnabled) {
    if (typeof window === 'undefined') return;
    try {
      window.localStorage.setItem(AUTO_REFRESH_PREF_KEY, nextEnabled ? '1' : '0');
    } catch (_error) {}
  }

  function resolveLoginRedirectPath() {
    if (typeof window === 'undefined') {
      return buildDashboardLoginPath({ basePath: dashboardBasePath });
    }
    const pathname = String(window.location?.pathname || dashboardIndexPath(dashboardBasePath));
    const search = String(window.location?.search || '');
    const hash = String(window.location?.hash || '');
    return buildDashboardLoginPath({
      basePath: dashboardBasePath,
      nextPath: `${pathname}${search}${hash}`
    });
  }

  function redirectToLogin() {
    if (typeof window === 'undefined') return;
    window.location.replace(resolveLoginRedirectPath());
  }

  function hasVisibleUnsavedConfigChanges(doc = null) {
    const targetDocument = doc || (typeof document !== 'undefined' ? document : null);
    if (!targetDocument || typeof targetDocument.querySelector !== 'function') return false;
    return Boolean(targetDocument.querySelector(ACTIVE_DIRTY_CONFIG_SAVE_BAR_SELECTOR));
  }

  function confirmDiscardUnsavedConfigChanges(win = null, doc = null) {
    if (!hasVisibleUnsavedConfigChanges(doc)) return true;
    const targetWindow = win || (typeof window !== 'undefined' ? window : null);
    if (!targetWindow || typeof targetWindow.confirm !== 'function') return false;
    return targetWindow.confirm(
      'You have unsaved configuration changes. Press OK to discard them and log out, or Cancel to stay on this page.'
    );
  }

  function handleConfirmedLogoutBeforeUnload(event) {
    if (suppressBeforeUnloadPrompt !== true) return;
    if (event && typeof event.stopImmediatePropagation === 'function') {
      event.stopImmediatePropagation();
    }
  }

  function clearAuthExpiryTimer() {
    if (authExpiryTimer) {
      clearTimeout(authExpiryTimer);
      authExpiryTimer = null;
    }
    authExpiryAtSeconds = 0;
  }

  function scheduleAuthExpiryRedirect(session = null) {
    clearAuthExpiryTimer();
    const snapshot = session && typeof session === 'object'
      ? session
      : getDashboardSessionState();
    if (!snapshot || snapshot.authenticated !== true) return;
    const expiresAtSeconds = Number(snapshot.expiresAt || 0);
    if (!Number.isFinite(expiresAtSeconds) || expiresAtSeconds <= 0) return;
    if (authExpiryTimer && authExpiryAtSeconds === Math.floor(expiresAtSeconds)) return;
    const delayMs = Math.max(0, (Math.floor(expiresAtSeconds) * 1000) - Date.now());
    authExpiryAtSeconds = Math.floor(expiresAtSeconds);
    authExpiryTimer = setTimeout(() => {
      authExpiryTimer = null;
      authExpiryAtSeconds = 0;
      redirectToLogin();
    }, delayMs);
  }

  const routeController = createDashboardRouteController({
    tabs: DASHBOARD_TABS,
    normalizeTab,
    tabLoadingMessages: TAB_LOADING_MESSAGES,
    store: dashboardStore,
    mountDashboardApp,
    restoreDashboardSession,
    getDashboardSessionState,
    setDashboardActiveTab,
    refreshDashboardTab,
    selectRefreshInterval: (tab) =>
      AUTO_REFRESH_TABS.has(normalizeTab(tab)) ? AUTO_REFRESH_INTERVAL_MS : 0,
    setPollingContext: (tab, intervalMs) => dashboardStore.setPollingContext(tab, intervalMs),
    recordPollingSkip: (reason, tab, intervalMs) =>
      dashboardStore.recordPollingSkip(reason, tab, intervalMs),
    recordPollingResume: (reason, tab, intervalMs) =>
      dashboardStore.recordPollingResume(reason, tab, intervalMs),
    recordRefreshMetrics: (metrics) => dashboardStore.recordRefreshMetrics(metrics),
    isAuthenticated: () => dashboardStore.getState().session.authenticated === true,
    onBootstrapSession: (session) => {
      scheduleAuthExpiryRedirect(session);
    },
    isAutoRefreshEnabled: () => autoRefreshEnabled === true,
    isAutoRefreshTab: (tab) => AUTO_REFRESH_TABS.has(normalizeTab(tab)),
    shouldRefreshOnActivate: ({ tab, store }) => {
      const normalized = normalizeTab(tab);
      if (AUTO_REFRESH_TABS.has(normalized)) return true;
      const state = store && typeof store.getState === 'function' ? store.getState() : null;
      const configSnapshot = state && state.snapshots ? state.snapshots.config : null;
      if (!configSnapshot || Object.keys(configSnapshot).length === 0) {
        return true;
      }
      if (normalized === 'status') {
        const monitoringSnapshot = state && state.snapshots ? state.snapshots.monitoring : null;
        const monitoringFreshness = state && state.snapshots ? state.snapshots.monitoringFreshness : null;
        const hasRetentionHealth =
          monitoringSnapshot &&
          typeof monitoringSnapshot === 'object' &&
          monitoringSnapshot.retention_health &&
          typeof monitoringSnapshot.retention_health === 'object';
        const hasFreshness =
          monitoringFreshness &&
          typeof monitoringFreshness === 'object' &&
          String(monitoringFreshness.state || '').trim().length > 0;
        return !(hasRetentionHealth && hasFreshness);
      }
      return false;
    },
    redirectToLogin
  });

  onMount(async () => {
    if (typeof window !== 'undefined') {
      window.addEventListener('beforeunload', handleConfirmedLogoutBeforeUnload, true);
    }
    if (typeof document !== 'undefined') {
      const classList = document?.documentElement?.classList;
      dashboardLoaded =
        !!classList &&
        typeof classList.contains === 'function' &&
        classList.contains(DASHBOARD_LOADED_CLASS);
    }
    autoRefreshEnabled = readAutoRefreshPreference();
    routeController.setMounted(true);
    storeUnsubscribe = dashboardStore.subscribe((value) => {
      dashboardState = value;
    });
    telemetryUnsubscribe = dashboardStore.runtimeTelemetryStore.subscribe((value) => {
      runtimeTelemetry = value;
    });

    try {
      const [
        { default: loadedIpBansTab },
        { default: loadedStatusTab },
        { default: loadedVerificationTab },
        { default: loadedTrapsTab },
        { default: loadedAdvancedTab },
        { default: loadedRateLimitingTab },
        { default: loadedGeoTab },
        { default: loadedFingerprintingTab },
        { default: loadedRobotsTab },
        { default: loadedTuningTab }
      ] = await Promise.all([
        import('$lib/components/dashboard/IpBansTab.svelte'),
        import('$lib/components/dashboard/StatusTab.svelte'),
        import('$lib/components/dashboard/VerificationTab.svelte'),
        import('$lib/components/dashboard/TrapsTab.svelte'),
        import('$lib/components/dashboard/AdvancedTab.svelte'),
        import('$lib/components/dashboard/RateLimitingTab.svelte'),
        import('$lib/components/dashboard/GeoTab.svelte'),
        import('$lib/components/dashboard/FingerprintingTab.svelte'),
        import('$lib/components/dashboard/RobotsTab.svelte'),
        import('$lib/components/dashboard/TuningTab.svelte')
      ]);
      IpBansTabComponent = loadedIpBansTab;
      StatusTabComponent = loadedStatusTab;
      VerificationTabComponent = loadedVerificationTab;
      TrapsTabComponent = loadedTrapsTab;
      AdvancedTabComponent = loadedAdvancedTab;
      RateLimitingTabComponent = loadedRateLimitingTab;
      GeoTabComponent = loadedGeoTab;
      FingerprintingTabComponent = loadedFingerprintingTab;
      RobotsTabComponent = loadedRobotsTab;
      TuningTabComponent = loadedTuningTab;

      const bootstrapped = await routeController.bootstrapRuntime({
        initialTab: normalizeTab(data?.initialHashTab || ''),
        basePath: dashboardBasePath
      });
      runtimeReady = bootstrapped === true;
      const bootstrapAdversarySimControlState = deriveAdversarySimControlState({
        configSnapshot: dashboardStore.getState()?.snapshots?.config || {},
        adversarySimStatus
      });
      if (runtimeReady && bootstrapAdversarySimControlState.controlAvailable) {
        await refreshAdversarySimStatus('bootstrap');
      }
    } catch (error) {
      runtimeError = error && error.message ? error.message : 'Dashboard bootstrap failed.';
    }
  });

  onDestroy(() => {
    const runtimeWasMounted = routeController.getRuntimeMounted();
    routeController.dispose();
    storeUnsubscribe();
    telemetryUnsubscribe();
    clearAdversarySimStatusPollTimer();
    clearAuthExpiryTimer();
    suppressBeforeUnloadPrompt = false;
    if (typeof window !== 'undefined') {
      window.removeEventListener('beforeunload', handleConfirmedLogoutBeforeUnload, true);
    }
    if (typeof document !== 'undefined') {
      const classList = document?.documentElement?.classList;
      if (classList && typeof classList.remove === 'function') {
        classList.remove(DASHBOARD_LOADED_CLASS);
      }
      dashboardLoaded = false;
      clearDashboardBodyClasses(document);
    }
    if (runtimeWasMounted) {
      unmountDashboardApp();
    }
  });

  function onTabClick(event, tab) {
    event.preventDefault();
    void routeController.applyActiveTab(tab, { reason: 'click', syncHash: true });
  }

  function onTabKeydown(event, tab) {
    const target = routeController.keyNavTarget(tab, event.key);
    if (!target) return;
    event.preventDefault();
    void routeController.applyActiveTab(target, { reason: 'keyboard', syncHash: true });
    setTimeout(() => {
      focusTab(target);
    }, 0);
  }

  function onWindowHashChange() {
    if (!routeController.getRuntimeMounted()) return;
    routeController.syncFromHash('hashchange');
  }

  $: if (runtimeReady && routeController.getRuntimeMounted()) {
    const isAuthenticated = dashboardState?.session?.authenticated === true;
    if (isAuthenticated) {
      scheduleAuthExpiryRedirect();
    } else if (!loggingOut) {
      clearAuthExpiryTimer();
      redirectToLogin();
    }
  }

  function onDocumentVisibilityChange() {
    routeController.handleVisibilityChange();
  }

  function setAdminMessage(text = '', kind = 'info') {
    adminMessageText = String(text || '');
    adminMessageKind = String(kind || 'info');
  }

  function onAutoRefreshToggle(event) {
    const checked = event && event.currentTarget && event.currentTarget.checked === true;
    autoRefreshEnabled = checked;
    writeAutoRefreshPreference(checked);
    routeController.schedulePolling('auto-refresh-toggle');
    if (checked && autoRefreshSupported && runtimeReady) {
      void routeController.refreshTab(activeTabKey, 'manual-refresh');
    }
  }

  async function onRefreshNow(event) {
    if (event && typeof event.preventDefault === 'function') {
      event.preventDefault();
    }
    if (refreshNowDisabled || !autoRefreshSupported) return;
    await routeController.refreshTab(activeTabKey, 'manual-refresh');
  }

  async function onGlobalTestModeToggleChange(event) {
    const target = event && event.currentTarget ? event.currentTarget : null;
    const nextValue = target && target.checked === true;
    const previousValue = currentTestModeValue;
    if (globalTestModeToggleDisabled || typeof onSaveConfig !== 'function') {
      if (target) target.checked = previousValue;
      return;
    }
    if (nextValue === previousValue) return;

    savingGlobalTestMode = true;
    try {
      const nextConfig = await onSaveConfig(
        { test_mode: nextValue },
        {
          successMessage: `Test mode ${nextValue ? 'enabled (logging only)' : 'disabled (blocking active)'}`,
          refresh: false
        }
      );
      const persistedValue =
        nextConfig && typeof nextConfig === 'object'
          ? nextConfig.test_mode === true
          : nextValue;
      if (target) target.checked = persistedValue;
    } catch (_error) {
      if (target) target.checked = previousValue;
    } finally {
      savingGlobalTestMode = false;
    }
  }

  function clearAdversarySimStatusPollTimer() {
    if (adversarySimStatusPollTimer) {
      clearInterval(adversarySimStatusPollTimer);
      adversarySimStatusPollTimer = null;
    }
    adversarySimStatusRequestInFlight = null;
  }

  function syncAdversarySimTimers() {
    const shouldPollStatus =
      routeController.getRuntimeMounted() &&
      runtimeReady &&
      (normalizedAdversarySimStatus.enabled === true ||
        normalizedAdversarySimStatus.phase === 'running' ||
        normalizedAdversarySimStatus.phase === 'stopping');

    if (shouldPollStatus) {
      if (!adversarySimStatusPollTimer) {
        adversarySimStatusPollTimer = setInterval(() => {
          void refreshAdversarySimStatus('poll');
        }, 1000);
      }
    } else {
      clearAdversarySimStatusPollTimer();
    }
  }

  async function refreshAdversarySimStatus(_reason = 'manual') {
    if (!routeController.getRuntimeMounted() || !runtimeReady) return;
    if (adversarySimStatusRequestInFlight) {
      return adversarySimStatusRequestInFlight;
    }
    adversarySimStatusRequestInFlight = (async () => {
      try {
        const status = await getDashboardAdversarySimStatus({
          telemetry: {
            tab: 'status',
            reason: _reason,
            source: 'adversary-sim-status'
          }
        });
        adversarySimStatus = status && typeof status === 'object' ? status : {};
      } catch (error) {
        if (error && Number(error.status) === 404) {
          adversarySimStatus = {
            runtime_environment: configSnapshot.runtime_environment,
            adversary_sim_available: false,
            adversary_sim_enabled: false
          };
        }
      } finally {
        adversarySimStatusRequestInFlight = null;
      }
      syncAdversarySimTimers();
    })();
    return adversarySimStatusRequestInFlight;
  }

  async function onGlobalAdversarySimToggleChange(event) {
    const target = event && event.currentTarget ? event.currentTarget : null;
    const nextValue = target && target.checked === true;
    const previousValue = normalizedAdversarySimStatus.enabled === true;
    if (globalAdversarySimToggleDisabled) {
      if (target) target.checked = previousValue;
      return;
    }
    if (nextValue === previousValue) return;
    if (nextValue && frontierProviderCount === 0) {
      const continueWithoutFrontier = typeof window !== 'undefined'
        ? window.confirm(
          'No frontier model provider keys are configured. Press OK to continue without frontier calls, or Cancel to add keys to .env.local and restart make dev.'
        )
        : false;
      if (!continueWithoutFrontier) {
        if (target) target.checked = previousValue;
        setAdminMessage(
          'Frontier provider keys are missing. Add SHUMA_FRONTIER_*_API_KEY values to .env.local, restart make dev, then toggle again (or continue without frontier calls).',
          'warning'
        );
        return;
      }
      setAdminMessage(
        'Continuing without frontier provider calls for this run.',
        'warning'
      );
    }

    const previousStatusSnapshot =
      adversarySimStatus && typeof adversarySimStatus === 'object'
        ? { ...adversarySimStatus }
        : {};
    savingGlobalAdversarySim = true;
    try {
      const result = await withRefreshedSessionOnAuthError(
        () => controlDashboardAdversarySim(nextValue, {
          telemetry: {
            tab: 'status',
            reason: 'adversary-sim-toggle',
            source: 'adversary-sim-control'
          }
        })
      );
      const status = result && result.status ? result.status : {};
      const normalizedStatus = normalizeAdversarySimStatus(status);
      adversarySimStatus = status && typeof status === 'object' ? status : {};
      if (target) target.checked = normalizedStatus.enabled;
      setAdminMessage(
        nextValue
          ? 'Adversary simulation run started.'
          : 'Adversary simulation run stopped.',
        'success'
      );
    } catch (error) {
      adversarySimStatus = previousStatusSnapshot;
      if (target) target.checked = previousValue;
      if (isAuthSessionExpiredError(error)) {
        setAdminMessage(
          'Adversary simulation control session expired. Redirecting to login...',
          'warning'
        );
        redirectToLogin();
        return;
      }
      const message = formatActionError(error, 'Failed to toggle adversary simulation.');
      setAdminMessage(`Error: ${message}`, 'error');
    } finally {
      savingGlobalAdversarySim = false;
      try {
        await refreshAdversarySimStatus('toggle');
      } catch (_error) {}
    }
  }

  function formatActionError(error, fallback = 'Action failed.') {
    if (error && typeof error.message === 'string' && error.message.trim()) {
      return error.message.trim();
    }
    return fallback;
  }

  function isAuthSessionExpiredError(error) {
    const status = Number(error?.status || 0);
    if (status === 401) return true;
    if (status !== 403) return false;
    const message = String(error?.message || '').trim().toLowerCase();
    if (!message) return false;
    return (
      message.includes('csrf') ||
      message.includes('session') ||
      message.includes('trust boundary') ||
      message.includes('unauthorized')
    );
  }

  async function withRefreshedSessionOnAuthError(action) {
    if (typeof action !== 'function') {
      throw new Error('Action callback is required.');
    }
    try {
      return await action();
    } catch (error) {
      if (!isAuthSessionExpiredError(error)) throw error;
      const restored = await restoreDashboardSession();
      if (restored !== true) throw error;
      return action();
    }
  }

  async function onSaveConfig(patch, options = {}) {
    const successMessage = options && typeof options.successMessage === 'string'
      ? options.successMessage
      : 'Configuration saved';
    const shouldRefresh = options?.refresh !== false;
    setAdminMessage('Saving configuration...', 'info');
    try {
      const nextConfig = await withRefreshedSessionOnAuthError(
        () => updateDashboardConfig(patch || {}, {
          telemetry: {
            tab: activeTabKey,
            reason: 'config-save',
            source: 'config-update'
          }
        })
      );
      if (shouldRefresh) {
        await routeController.refreshTab(activeTabKey, 'config-save');
      }
      setAdminMessage(successMessage, 'success');
      return nextConfig;
    } catch (error) {
      if (isAuthSessionExpiredError(error)) {
        setAdminMessage('Configuration save session expired. Redirecting to login...', 'warning');
        redirectToLogin();
        throw error;
      }
      const message = formatActionError(error, 'Failed to save configuration.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onValidateConfig(patch) {
    return validateDashboardConfigPatch(patch || {}, {
      telemetry: {
        tab: activeTabKey,
        reason: 'config-validate',
        source: 'config-validation'
      }
    });
  }

  async function onBan(payload = {}) {
    const ip = String(payload.ip || '').trim();
    const duration = Number(payload.duration || 0);
    if (!ip || !Number.isFinite(duration) || duration <= 0) return;
    setAdminMessage(`Banning ${ip}...`, 'info');
    try {
      await withRefreshedSessionOnAuthError(
        () => banDashboardIp(ip, duration, 'manual_ban', {
          telemetry: {
            tab: 'ip-bans',
            reason: 'manual-ban',
            source: 'ban-control'
          }
        })
      );
      await routeController.refreshTab('ip-bans', 'ban-save');
      setAdminMessage(`Banned ${ip} for ${duration}s`, 'success');
    } catch (error) {
      if (isAuthSessionExpiredError(error)) {
        setAdminMessage('Ban action session expired. Redirecting to login...', 'warning');
        redirectToLogin();
        throw error;
      }
      const message = formatActionError(error, 'Failed to ban Internet Protocol address.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onUnban(payload = {}) {
    const ip = String(payload.ip || '').trim();
    if (!ip) return;
    setAdminMessage(`Unbanning ${ip}...`, 'info');
    try {
      await withRefreshedSessionOnAuthError(() => unbanDashboardIp(ip, {
        telemetry: {
          tab: 'ip-bans',
          reason: 'manual-unban',
          source: 'ban-control'
        }
      }));
      await routeController.refreshTab('ip-bans', 'unban-save');
      setAdminMessage(`Unbanned ${ip}`, 'success');
    } catch (error) {
      if (isAuthSessionExpiredError(error)) {
        setAdminMessage('Unban action session expired. Redirecting to login...', 'warning');
        redirectToLogin();
        throw error;
      }
      const message = formatActionError(error, 'Failed to unban Internet Protocol address.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onRobotsPreview(patch = null) {
    return getDashboardRobotsPreview(patch, {
      telemetry: {
        tab: 'robots',
        reason: 'preview',
        source: 'robots-preview'
      }
    });
  }

  async function onFetchEventsRange(hours, options = {}) {
    const requestOptions = options && typeof options === 'object'
      ? { ...options }
      : {};
    requestOptions.telemetry = {
      tab: 'monitoring',
      reason: 'range-fetch',
      source: 'monitoring-range',
      ...(requestOptions.telemetry && typeof requestOptions.telemetry === 'object'
        ? requestOptions.telemetry
        : {})
    };
    return getDashboardEvents(hours, requestOptions);
  }

  async function onLogoutClick(event) {
    if (!routeController.getRuntimeMounted()) return;
    event.preventDefault();
    if (loggingOut) return;
    const hasUnsavedConfigChanges = hasVisibleUnsavedConfigChanges();
    if (!confirmDiscardUnsavedConfigChanges()) return;
    let redirectingToLogin = false;
    loggingOut = true;
    try {
      suppressBeforeUnloadPrompt = hasUnsavedConfigChanges;
      routeController.abortInFlightRefresh();
      clearAdversarySimStatusPollTimer();
      await logoutDashboardSession();
      dashboardStore.setSession({ authenticated: false, csrfToken: '' });
      routeController.clearPolling();
      redirectingToLogin = true;
      redirectToLogin();
    } finally {
      if (!redirectingToLogin) {
        suppressBeforeUnloadPrompt = false;
      }
      loggingOut = false;
    }
  }
</script>
<svelte:head>
  <title>Shuma-Gorath Dashboard</title>
  <link rel="icon" type="image/png" href={faviconHref}>
</svelte:head>
<svelte:window on:hashchange={onWindowHashChange} />
<svelte:document on:visibilitychange={onDocumentVisibilityChange} />
<div id="lost-connection" aria-live="polite" aria-hidden={lostConnectionVisible ? 'false' : 'true'}>
  <div id="connection-status">offline!</div>
</div>
<div class="container panel panel-border" data-dashboard-runtime-mode="native">
  <div id="test-mode-banner" class="test-mode-banner" class:hidden={!testModeEnabled}>
    TEST MODE ACTIVE - Logging only, no active defences
  </div>
  <div class="dashboard-global-control dashboard-test-mode-control">
    <label class="toggle-switch" for="global-test-mode-toggle" title={globalTestModeToggleDisabledReason}>
      <input
        id="global-test-mode-toggle"
        type="checkbox"
        aria-label="Enable test mode"
        checked={currentTestModeValue}
        disabled={globalTestModeToggleDisabled}
        title={globalTestModeToggleDisabledReason}
        on:change={onGlobalTestModeToggleChange}
      >
      <span class="toggle-slider"></span>
    </label>
    <span class="dashboard-global-control-label" class:dashboard-global-control-label--disabled={globalTestModeToggleDisabled} title={globalTestModeToggleDisabledReason}>Test Mode</span>
  </div>
  <div class="dashboard-global-control dashboard-adversary-sim-control">
    <label class="toggle-switch" for="global-adversary-sim-toggle" title={globalAdversarySimToggleDisabledReason}>
      <input
        id="global-adversary-sim-toggle"
        type="checkbox"
        aria-label="Enable adversary simulation"
        checked={adversarySimToggleEnabled}
        disabled={globalAdversarySimToggleDisabled}
        title={globalAdversarySimToggleDisabledReason}
        on:change={onGlobalAdversarySimToggleChange}
      >
      <span class="toggle-slider"></span>
    </label>
    <span class="dashboard-global-control-label" class:dashboard-global-control-label--disabled={globalAdversarySimToggleDisabled} title={globalAdversarySimToggleDisabledReason}>Adversary Sim</span>
  </div>
  <div id="adversary-sim-lifecycle-copy" class="dashboard-adversary-sim-hint text-muted">
    <p class="dashboard-global-control-copy-block">{adversarySimLifecycleCopy}</p>
  </div>
  <button
    id="logout-btn"
    class="btn btn-subtle dashboard-logout"
    aria-label="Log out of admin session"
    disabled={loggingOut || dashboardState.session.authenticated !== true}
    on:click={onLogoutClick}
  >Logout</button>
  <header>
    <div class="shuma-image-wrapper">
      <img src={shumaImageSrc} alt="Shuma-Gorath" class="shuma-gorath-img">
    </div>
    <h1>Shuma-Gorath</h1>
    <p class="subtitle text-muted"><a href="https://read.dukeupress.edu/books/book/27/Staying-with-the-TroubleMaking-Kin-in-the" target="_blank">Chthulucene</a> Bot Defence</p>
    <nav class="dashboard-tabs" aria-label="Dashboard sections">
      {#each DASHBOARD_TABS as tab}
        {@const tabKey = normalizeTab(tab)}
        {@const selected = activeTabKey === tabKey}
        <a
          id={`dashboard-tab-${tab}`}
          class="dashboard-tab-link"
          class:active={selected}
          data-dashboard-tab-link={tab}
          href={`#${tab}`}
          role="tab"
          aria-selected={selected ? 'true' : 'false'}
          aria-controls={`dashboard-panel-${tab}`}
          tabindex={selected ? 0 : -1}
          on:click={(event) => onTabClick(event, tab)}
          on:keydown={(event) => onTabKeydown(event, tab)}
          use:registerTabLink={tab}
        >
          {#if tab === 'ip-bans'}
            <abbr title="Internet Protocol">IP</abbr>&nbsp;Bans
          {:else if tab === 'rate-limiting'}
            Rate Limiting
          {:else if tab === 'geo'}
            <abbr title="Geolocation">GEO</abbr>
          {:else if tab === 'robots'}
            Robots.txt
          {:else}
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          {/if}
        </a>
      {/each}
    </nav>
    {#if autoRefreshSupported}
      <div id="dashboard-refresh-controls" class="dashboard-refresh-controls">
        <div class="dashboard-refresh-meta">
          <span id="last-updated" class="text-muted">{lastUpdatedText}</span>
          {#if !autoRefreshEnabled}
            <button
              id="refresh-now-btn"
              class="btn btn-subtle"
              aria-label="Refresh now"
              title="Refresh now"
              disabled={refreshNowDisabled}
              on:click={onRefreshNow}
            >↻</button>
          {/if}
        </div>
        <div class="dashboard-refresh-auto">
          <span id="refresh-mode" class="text-muted">{refreshModeText}</span>
          <div class="toggle-row dashboard-refresh-toggle">
            <label class="toggle-switch" for="auto-refresh-toggle">
              <input
                id="auto-refresh-toggle"
                type="checkbox"
                aria-label="Enable automatic refresh for current tab"
                checked={autoRefreshEnabled}
                on:change={onAutoRefreshToggle}
              >
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>
      </div>
    {/if}
  </header>

  <MonitoringTab
    managed={true}
    isActive={activeTabKey === 'monitoring'}
    autoRefreshEnabled={autoRefreshEnabled}
    tabStatus={tabStatus.monitoring || {}}
    analyticsSnapshot={snapshots.analytics}
    eventsSnapshot={snapshots.events}
    bansSnapshot={snapshots.bans}
    mazeSnapshot={snapshots.maze}
    cdpSnapshot={snapshots.cdp}
    cdpEventsSnapshot={snapshots.cdpEvents}
    monitoringSnapshot={snapshots.monitoring}
    monitoringFreshnessSnapshot={snapshots.monitoringFreshness}
    configSnapshot={snapshots.config}
    onFetchEventsRange={onFetchEventsRange}
  />

  <div
    id="dashboard-admin-section"
    class="section admin-section"
    hidden={activeTabKey === 'monitoring'}
    aria-hidden={activeTabKey === 'monitoring' ? 'true' : 'false'}
  >
    <div class="admin-groups">
      {#if IpBansTabComponent}
        <svelte:component
          this={IpBansTabComponent}
          managed={true}
          isActive={activeTabKey === 'ip-bans'}
          tabStatus={tabStatus['ip-bans'] || {}}
          bansSnapshot={snapshots.bans}
          ipBansFreshnessSnapshot={snapshots.ipBansFreshness}
          ipRangeSuggestionsSnapshot={snapshots.ipRangeSuggestions}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          ipRangeSuggestionsVersion={snapshotVersions.ipRangeSuggestions || 0}
          onSaveConfig={onSaveConfig}
          onBan={onBan}
          onUnban={onUnban}
        />
      {:else}
        <section
          id="dashboard-panel-ip-bans"
          class="admin-group"
          data-dashboard-tab-panel="ip-bans"
          aria-labelledby="dashboard-tab-ip-bans"
          hidden={activeTabKey !== 'ip-bans'}
          aria-hidden={activeTabKey === 'ip-bans' ? 'false' : 'true'}
        >
          <p class="message info">Loading ban controls...</p>
        </section>
      {/if}
      {#if StatusTabComponent}
        <svelte:component
          this={StatusTabComponent}
          managed={true}
          isActive={activeTabKey === 'status'}
          runtimeTelemetry={runtimeTelemetry}
          tabStatus={tabStatus.status || {}}
          configSnapshot={snapshots.config}
          monitoringSnapshot={snapshots.monitoring}
        />
      {:else}
        <section
          id="dashboard-panel-status"
          class="admin-group"
          data-dashboard-tab-panel="status"
          aria-labelledby="dashboard-tab-status"
          hidden={activeTabKey !== 'status'}
          aria-hidden={activeTabKey === 'status' ? 'false' : 'true'}
        >
          <p class="message info">Loading status signals...</p>
        </section>
      {/if}
      {#if VerificationTabComponent}
        <svelte:component
          this={VerificationTabComponent}
          managed={true}
          isActive={activeTabKey === 'verification'}
          tabStatus={tabStatus.verification || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-verification"
          class="admin-group"
          data-dashboard-tab-panel="verification"
          aria-labelledby="dashboard-tab-verification"
          hidden={activeTabKey !== 'verification'}
          aria-hidden={activeTabKey === 'verification' ? 'false' : 'true'}
        >
          <p class="message info">Loading verification controls...</p>
        </section>
      {/if}
      {#if TrapsTabComponent}
        <svelte:component
          this={TrapsTabComponent}
          managed={true}
          isActive={activeTabKey === 'traps'}
          tabStatus={tabStatus.traps || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-traps"
          class="admin-group"
          data-dashboard-tab-panel="traps"
          aria-labelledby="dashboard-tab-traps"
          hidden={activeTabKey !== 'traps'}
          aria-hidden={activeTabKey === 'traps' ? 'false' : 'true'}
        >
          <p class="message info">Loading trap controls...</p>
        </section>
      {/if}
      {#if RateLimitingTabComponent}
        <svelte:component
          this={RateLimitingTabComponent}
          managed={true}
          isActive={activeTabKey === 'rate-limiting'}
          tabStatus={tabStatus['rate-limiting'] || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-rate-limiting"
          class="admin-group"
          data-dashboard-tab-panel="rate-limiting"
          aria-labelledby="dashboard-tab-rate-limiting"
          hidden={activeTabKey !== 'rate-limiting'}
          aria-hidden={activeTabKey === 'rate-limiting' ? 'false' : 'true'}
        >
          <p class="message info">Loading rate limiting controls...</p>
        </section>
      {/if}
      {#if GeoTabComponent}
        <svelte:component
          this={GeoTabComponent}
          managed={true}
          isActive={activeTabKey === 'geo'}
          tabStatus={tabStatus.geo || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-geo"
          class="admin-group"
          data-dashboard-tab-panel="geo"
          aria-labelledby="dashboard-tab-geo"
          hidden={activeTabKey !== 'geo'}
          aria-hidden={activeTabKey === 'geo' ? 'false' : 'true'}
        >
          <p class="message info">Loading GEO controls...</p>
        </section>
      {/if}
      {#if FingerprintingTabComponent}
        <svelte:component
          this={FingerprintingTabComponent}
          managed={true}
          isActive={activeTabKey === 'fingerprinting'}
          tabStatus={tabStatus.fingerprinting || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          cdpSnapshot={snapshots.cdp}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-fingerprinting"
          class="admin-group"
          data-dashboard-tab-panel="fingerprinting"
          aria-labelledby="dashboard-tab-fingerprinting"
          hidden={activeTabKey !== 'fingerprinting'}
          aria-hidden={activeTabKey === 'fingerprinting' ? 'false' : 'true'}
        >
          <p class="message info">Loading fingerprinting controls...</p>
        </section>
      {/if}
      {#if RobotsTabComponent}
        <svelte:component
          this={RobotsTabComponent}
          managed={true}
          isActive={activeTabKey === 'robots'}
          tabStatus={tabStatus.robots || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
          onFetchRobotsPreview={onRobotsPreview}
        />
      {:else}
        <section
          id="dashboard-panel-robots"
          class="admin-group"
          data-dashboard-tab-panel="robots"
          aria-labelledby="dashboard-tab-robots"
          hidden={activeTabKey !== 'robots'}
          aria-hidden={activeTabKey === 'robots' ? 'false' : 'true'}
        >
          <p class="message info">Loading robots policy...</p>
        </section>
      {/if}
      {#if TuningTabComponent}
        <svelte:component
          this={TuningTabComponent}
          managed={true}
          isActive={activeTabKey === 'tuning'}
          tabStatus={tabStatus.tuning || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
        />
      {:else}
        <section
          id="dashboard-panel-tuning"
          class="admin-group"
          data-dashboard-tab-panel="tuning"
          aria-labelledby="dashboard-tab-tuning"
          hidden={activeTabKey !== 'tuning'}
          aria-hidden={activeTabKey === 'tuning' ? 'false' : 'true'}
        >
          <p class="message info">Loading tuning controls...</p>
        </section>
      {/if}
      {#if AdvancedTabComponent}
        <svelte:component
          this={AdvancedTabComponent}
          managed={true}
          isActive={activeTabKey === 'advanced'}
          tabStatus={tabStatus.advanced || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          dashboardBasePath={dashboardBasePath}
          onSaveConfig={onSaveConfig}
          onValidateConfig={onValidateConfig}
        />
      {:else}
        <section
          id="dashboard-panel-advanced"
          class="admin-group"
          data-dashboard-tab-panel="advanced"
          aria-labelledby="dashboard-tab-advanced"
          hidden={activeTabKey !== 'advanced'}
          aria-hidden={activeTabKey === 'advanced' ? 'false' : 'true'}
        >
          <p class="message info">Loading advanced controls...</p>
        </section>
      {/if}
    </div>
    <div id="admin-msg" class={`message ${adminMessageKind}`}>{adminMessageText}</div>
  </div>

  {#if runtimeError}
    <p class="message error">{runtimeError}</p>
  {/if}
  {#if !runtimeReady && !runtimeError}
    <p class="message info">Loading dashboard runtime...</p>
  {/if}
</div>
