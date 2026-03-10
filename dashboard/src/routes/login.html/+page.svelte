<script>
  import { onMount, tick } from 'svelte';
  import {
    deriveDashboardBodyClassState,
    syncDashboardBodyClasses
  } from '$lib/runtime/dashboard-body-classes.js';
  import {
    dashboardIndexPath,
    normalizeDashboardBasePath,
    resolveDashboardAssetPath
  } from '$lib/runtime/dashboard-paths.js';

  export let data;
  let apiKey = '';
  let apiKeyInput = null;
  let messageText = '';
  let messageKind = 'info';
  let runtimeStateAvailable = false;
  const passwordManagerIdentity = 'admin';
  const dashboardBasePath = typeof data?.dashboardBasePath === 'string'
    ? data.dashboardBasePath
    : normalizeDashboardBasePath();
  const fallbackNextPath = dashboardIndexPath(dashboardBasePath);
  const faviconHref = typeof data?.faviconHref === 'string'
    ? data.faviconHref
    : resolveDashboardAssetPath(dashboardBasePath, 'assets/shuma-gorath-pencil-closed.png');
  let nextPath = fallbackNextPath;

  function safeNextPath(raw) {
    const fallback = fallbackNextPath;
    if (!raw) return fallback;
    try {
      const decoded = decodeURIComponent(raw);
      const url = new URL(decoded, window.location.origin);
      if (url.origin !== window.location.origin) return fallback;
      if (!url.pathname.startsWith(`${dashboardBasePath}/`)) return fallback;
      return `${url.pathname}${url.search}${url.hash}`;
    } catch (_e) {
      return fallback;
    }
  }

  async function getSessionState() {
    try {
      const resp = await fetch('/admin/session', { credentials: 'same-origin' });
      if (!resp.ok) return null;
      return await resp.json();
    } catch (_e) {
      return null;
    }
  }

  function setMessage(text, kind) {
    messageText = text;
    messageKind = kind;
  }

  function isLocalDevHost() {
    const host = String(window.location.hostname || '').toLowerCase();
    return host === 'localhost' || host === '127.0.0.1' || host === '::1' || host === '[::1]';
  }

  function normalizeRuntimeEnvironment(rawValue) {
    const normalized = String(rawValue || '').trim().toLowerCase();
    if (normalized === 'runtime-dev' || normalized === 'runtime-prod') {
      return normalized;
    }
    return '';
  }

  function syncLoginRootClasses(runtimeEnvironment = '') {
    if (typeof document === 'undefined') return;
    const classState = deriveDashboardBodyClassState(
      { runtime_environment: normalizeRuntimeEnvironment(runtimeEnvironment) },
      { backendConnectionState: 'disconnected' }
    );
    syncDashboardBodyClasses(document, classState);
  }

  async function focusApiKeyInput() {
    if (typeof document === 'undefined' || !apiKeyInput) return;
    await tick();
    apiKeyInput.focus();
  }

  function loginMessageFromQuery(params) {
    const errorCode = String(params.get('error') || '').trim().toLowerCase();
    if (!errorCode) return '';
    if (errorCode === 'invalid_key') {
      return 'Login failed. Check your key.';
    }
    if (errorCode === 'access_blocked') {
      if (isLocalDevHost()) {
        return 'Login blocked by local admin access policy. Check SHUMA_ADMIN_IP_ALLOWLIST.';
      }
      return 'Login failed.';
    }
    if (errorCode === 'rate_limited') {
      const retryAfter = String(params.get('retry_after') || '').trim();
      if (retryAfter && isLocalDevHost()) {
        return `Too many login attempts. Retry in ${retryAfter}s.`;
      }
      return 'Login temporarily unavailable. Retry shortly.';
    }
    if (errorCode === 'invalid_request') {
      return 'Login failed. Refresh and retry.';
    }
    return 'Login failed.';
  }

  onMount(async () => {
    syncLoginRootClasses('');
    const params = new URLSearchParams(window.location.search || '');
    nextPath = safeNextPath(params.get('next') || '');
    const queryMessage = loginMessageFromQuery(params);
    if (queryMessage) {
      setMessage(queryMessage, 'error');
    }
    const session = await getSessionState();
    const runtimeEnvironment = normalizeRuntimeEnvironment(session?.runtime_environment);
    runtimeStateAvailable = runtimeEnvironment.length > 0;
    if (!runtimeStateAvailable) {
      setMessage('Login unavailable while runtime state is unavailable. Refresh and retry.', 'error');
      return;
    }
    syncLoginRootClasses(runtimeEnvironment);
    if (session && session.authenticated === true && session.method === 'session') {
      window.location.replace(nextPath);
      return;
    }
    await focusApiKeyInput();
  });
</script>

<svelte:head>
  <title>Shuma-Gorath Dashboard Login</title>
  <link rel="icon" type="image/png" href={faviconHref}>
</svelte:head>

<main class="login-shell">
  <section class="login-card panel panel-border pad-md" aria-labelledby="login-title">
    <h1 id="login-title" class="hidden">Dashboard Login</h1>
    <form id="login-form" class="login-form" method="POST" action="/admin/login">
      <label class="control-label" for="username">Account</label>
      <input
        id="username"
        class="input-field"
        type="text"
        name="username"
        autocomplete="username"
        value={passwordManagerIdentity}
        readonly
        aria-readonly="true"
        spellcheck="false"
      >
      <input type="hidden" name="next" value={nextPath}>
      <label class="control-label" for="current-password">Enter your API key</label>
      <input
        id="current-password"
        class="input-field input-field--mono"
        type="password"
        name="password"
        autocomplete="current-password"
        spellcheck="false"
        autocapitalize="none"
        autocorrect="off"
        required
        aria-label="Application Programming Interface key"
        bind:value={apiKey}
        bind:this={apiKeyInput}
      >
      <button
        id="login-submit"
        class="btn btn-submit"
        type="submit"
        disabled={!runtimeStateAvailable}
      >
        Login
      </button>
    </form>
    <p id="login-msg" class={`message ${messageKind}`}>{messageText}</p>
  </section>
</main>
