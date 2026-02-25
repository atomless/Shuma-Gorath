<script>
  import { onDestroy, onMount } from 'svelte';
  import MonitoringTab from '$lib/components/dashboard/MonitoringTab.svelte';
  import {
    buildDashboardLoginPath,
    dashboardIndexPath,
    normalizeDashboardBasePath,
    resolveDashboardAssetPath
  } from '$lib/runtime/dashboard-paths.js';
  import { createDashboardRouteController } from '$lib/runtime/dashboard-route-controller.js';
  import {
    banDashboardIp,
    getDashboardEvents,
    getDashboardRobotsPreview,
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
    config: 'Loading config...',
    'rate-limiting': 'Loading rate limiting controls...',
    geo: 'Loading GEO controls...',
    fingerprinting: 'Loading fingerprinting controls...',
    robots: 'Loading robots policy...',
    tuning: 'Loading tuning values...'
  });
  const AUTO_REFRESH_INTERVAL_MS = 60000;
  const AUTO_REFRESH_TABS = new Set(['monitoring', 'ip-bans']);
  const AUTO_REFRESH_PREF_KEY = 'shuma_dashboard_auto_refresh_v1';

  const fallbackBasePath = normalizeDashboardBasePath();
  const dashboardBasePath = typeof data?.dashboardBasePath === 'string'
    ? data.dashboardBasePath
    : fallbackBasePath;
  const chartRuntimeSrc = typeof data?.chartRuntimeSrc === 'string'
    ? data.chartRuntimeSrc
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/vendor/chart-lite-1.0.0.min.js');
  const shumaImageLightSrc = typeof data?.shumaImageLightSrc === 'string'
    ? data.shumaImageLightSrc
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/shuma-gorath-pencil-light.png');
  const shumaImageDarkSrc = typeof data?.shumaImageDarkSrc === 'string'
    ? data.shumaImageDarkSrc
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/shuma-gorath-pencil-dark.png');

  const dashboardStore = createDashboardStore({ initialTab: 'monitoring' });

  let dashboardState = dashboardStore.getState();
  let runtimeTelemetry = dashboardStore.getRuntimeTelemetry();
  let storeUnsubscribe = () => {};
  let telemetryUnsubscribe = () => {};
  let runtimeReady = false;
  let runtimeError = '';
  let loggingOut = false;
  let savingGlobalTestMode = false;
  let autoRefreshEnabled = false;
  let adminMessageText = '';
  let adminMessageKind = 'info';
  let IpBansTabComponent = null;
  let StatusTabComponent = null;
  let ConfigTabComponent = null;
  let RateLimitingTabComponent = null;
  let GeoTabComponent = null;
  let FingerprintingTabComponent = null;
  let RobotsTabComponent = null;
  let TuningTabComponent = null;
  const tabLinks = {};

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
  $: hasConfigTestMode = typeof configSnapshot.test_mode === 'boolean';
  $: currentTestModeValue = hasConfigTestMode
    ? configSnapshot.test_mode === true
    : analyticsSnapshot.test_mode === true;
  $: testModeEnabled = currentTestModeValue;
  $: globalTestModeToggleDisabled =
    !runtimeReady ||
    loggingOut ||
    savingGlobalTestMode ||
    dashboardState?.session?.authenticated !== true ||
    configSnapshot.admin_config_write_enabled !== true;

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
    isAutoRefreshEnabled: () => autoRefreshEnabled === true,
    isAutoRefreshTab: (tab) => AUTO_REFRESH_TABS.has(normalizeTab(tab)),
    shouldRefreshOnActivate: ({ tab, store }) => {
      const normalized = normalizeTab(tab);
      if (AUTO_REFRESH_TABS.has(normalized)) return true;
      const state = store && typeof store.getState === 'function' ? store.getState() : null;
      const configSnapshot = state && state.snapshots ? state.snapshots.config : null;
      return !configSnapshot || Object.keys(configSnapshot).length === 0;
    },
    redirectToLogin
  });

  onMount(async () => {
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
        { default: loadedConfigTab },
        { default: loadedRateLimitingTab },
        { default: loadedGeoTab },
        { default: loadedFingerprintingTab },
        { default: loadedRobotsTab },
        { default: loadedTuningTab }
      ] = await Promise.all([
        import('$lib/components/dashboard/IpBansTab.svelte'),
        import('$lib/components/dashboard/StatusTab.svelte'),
        import('$lib/components/dashboard/ConfigTab.svelte'),
        import('$lib/components/dashboard/RateLimitingTab.svelte'),
        import('$lib/components/dashboard/GeoTab.svelte'),
        import('$lib/components/dashboard/FingerprintingTab.svelte'),
        import('$lib/components/dashboard/RobotsTab.svelte'),
        import('$lib/components/dashboard/TuningTab.svelte')
      ]);
      IpBansTabComponent = loadedIpBansTab;
      StatusTabComponent = loadedStatusTab;
      ConfigTabComponent = loadedConfigTab;
      RateLimitingTabComponent = loadedRateLimitingTab;
      GeoTabComponent = loadedGeoTab;
      FingerprintingTabComponent = loadedFingerprintingTab;
      RobotsTabComponent = loadedRobotsTab;
      TuningTabComponent = loadedTuningTab;

      const bootstrapped = await routeController.bootstrapRuntime({
        initialTab: normalizeTab(data?.initialHashTab || ''),
        chartRuntimeSrc,
        basePath: dashboardBasePath
      });
      runtimeReady = bootstrapped === true;
    } catch (error) {
      runtimeError = error && error.message ? error.message : 'Dashboard bootstrap failed.';
    }
  });

  onDestroy(() => {
    const runtimeWasMounted = routeController.getRuntimeMounted();
    routeController.dispose();
    storeUnsubscribe();
    telemetryUnsubscribe();
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

  function formatActionError(error, fallback = 'Action failed.') {
    if (error && typeof error.message === 'string' && error.message.trim()) {
      return error.message.trim();
    }
    return fallback;
  }

  async function onSaveConfig(patch, options = {}) {
    const successMessage = options && typeof options.successMessage === 'string'
      ? options.successMessage
      : 'Configuration saved';
    const shouldRefresh = options?.refresh !== false;
    setAdminMessage('Saving configuration...', 'info');
    try {
      const nextConfig = await updateDashboardConfig(patch || {});
      if (shouldRefresh) {
        await routeController.refreshTab(activeTabKey, 'config-save');
      }
      setAdminMessage(successMessage, 'success');
      return nextConfig;
    } catch (error) {
      const message = formatActionError(error, 'Failed to save configuration.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onValidateConfig(patch) {
    return validateDashboardConfigPatch(patch || {});
  }

  async function onBan(payload = {}) {
    const ip = String(payload.ip || '').trim();
    const duration = Number(payload.duration || 0);
    if (!ip || !Number.isFinite(duration) || duration <= 0) return;
    setAdminMessage(`Banning ${ip}...`, 'info');
    try {
      await banDashboardIp(ip, duration, 'manual_ban');
      await routeController.refreshTab('ip-bans', 'ban-save');
      setAdminMessage(`Banned ${ip} for ${duration}s`, 'success');
    } catch (error) {
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
      await unbanDashboardIp(ip);
      await routeController.refreshTab('ip-bans', 'unban-save');
      setAdminMessage(`Unbanned ${ip}`, 'success');
    } catch (error) {
      const message = formatActionError(error, 'Failed to unban Internet Protocol address.');
      setAdminMessage(`Error: ${message}`, 'error');
      throw error;
    }
  }

  async function onRobotsPreview(patch = null) {
    return getDashboardRobotsPreview(patch);
  }

  async function onFetchEventsRange(hours, options = {}) {
    return getDashboardEvents(hours, options || {});
  }

  async function onLogoutClick(event) {
    if (!routeController.getRuntimeMounted()) return;
    event.preventDefault();
    if (loggingOut) return;
    loggingOut = true;
    try {
      routeController.abortInFlightRefresh();
      await logoutDashboardSession();
      dashboardStore.setSession({ authenticated: false, csrfToken: '' });
      routeController.clearPolling();
      redirectToLogin();
    } finally {
      loggingOut = false;
    }
  }
</script>
<svelte:head>
  <title>Shuma-Gorath Dashboard</title>
</svelte:head>
<svelte:window on:hashchange={onWindowHashChange} />
<svelte:document on:visibilitychange={onDocumentVisibilityChange} />
<div class="container panel panel-border" data-dashboard-runtime-mode="native">
  <div id="test-mode-banner" class="test-mode-banner" class:hidden={!testModeEnabled}>
    TEST MODE ACTIVE - Logging only, no active defences
  </div>
  <div class="dashboard-global-control dashboard-test-mode-control">
    <label class="toggle-switch" for="global-test-mode-toggle">
      <input
        id="global-test-mode-toggle"
        type="checkbox"
        aria-label="Enable test mode"
        checked={currentTestModeValue}
        disabled={globalTestModeToggleDisabled}
        on:change={onGlobalTestModeToggleChange}
      >
      <span class="toggle-slider"></span>
    </label>
    <span class="dashboard-global-control-label" class:dashboard-global-control-label--disabled={globalTestModeToggleDisabled}>Test Mode</span>
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
      <img src={shumaImageDarkSrc} alt="Shuma-Gorath" class="shuma-gorath-img">
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
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
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
          dashboardBasePath={dashboardBasePath}
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
      {#if ConfigTabComponent}
        <svelte:component
          this={ConfigTabComponent}
          managed={true}
          isActive={activeTabKey === 'config'}
          tabStatus={tabStatus.config || {}}
          configSnapshot={snapshots.config}
          configVersion={snapshotVersions.config || 0}
          onSaveConfig={onSaveConfig}
          onValidateConfig={onValidateConfig}
        />
      {:else}
        <section
          id="dashboard-panel-config"
          class="admin-group"
          data-dashboard-tab-panel="config"
          aria-labelledby="dashboard-tab-config"
          hidden={activeTabKey !== 'config'}
          aria-hidden={activeTabKey === 'config' ? 'false' : 'true'}
        >
          <p class="message info">Loading config controls...</p>
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
