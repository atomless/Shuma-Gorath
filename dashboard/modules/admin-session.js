// @ts-check

export const create = (options = {}) => {
  const requestImpl =
    typeof options.request === 'function'
      ? options.request
      : fetch.bind(globalThis);
  const state = {
    authenticated: false,
    csrfToken: ''
  };

  const refreshUiState = () => {
    if (typeof options.refreshCoreActionButtonsState === 'function') {
      options.refreshCoreActionButtonsState();
    }
  };
  const resolveEndpoint = () => {
    if (typeof options.resolveAdminApiEndpoint !== 'function') return '';
    const resolved = options.resolveAdminApiEndpoint();
    if (!resolved) return '';
    if (typeof resolved === 'string') return resolved;
    if (typeof resolved.endpoint === 'string') return resolved.endpoint;
    return '';
  };

  const setAdminSession = (authenticated, csrfToken = '') => {
    state.authenticated = Boolean(authenticated);
    state.csrfToken = state.authenticated ? String(csrfToken || '') : '';
    refreshUiState();
  };

  const hasValidApiContext = () => state.authenticated;

  const getAdminContext = (messageTarget) => {
    const endpoint = resolveEndpoint();
    if (!endpoint) {
      if (messageTarget) {
        messageTarget.textContent = 'Unable to resolve admin API endpoint from the current page origin.';
        messageTarget.className = 'message error';
      }
      refreshUiState();
      return null;
    }

    if (!state.authenticated) {
      if (messageTarget) {
        messageTarget.textContent = 'Login required. Go to /dashboard/login.html.';
        messageTarget.className = 'message warning';
      }
      refreshUiState();
      return null;
    }

    refreshUiState();
    return { endpoint, apikey: '', sessionAuth: true, csrfToken: state.csrfToken };
  };

  const restoreAdminSession = async () => {
    const endpoint = resolveEndpoint();
    if (!endpoint) {
      setAdminSession(false);
      return false;
    }

    try {
      const response = await requestImpl(`${endpoint}/admin/session`, {
        method: 'GET',
        credentials: 'same-origin'
      });
      if (response.ok) {
        const data = await response.json().catch(() => ({}));
        if (data && data.authenticated === true) {
          setAdminSession(true, data.csrf_token || '');
          return true;
        }
        setAdminSession(false);
        return false;
      }
      setAdminSession(false);
      return false;
    } catch (_e) {
      setAdminSession(false);
      return false;
    }
  };

  const bindLogoutButton = (buttonId = 'logout-btn', messageId = 'admin-msg') => {
    const button = document.getElementById(buttonId);
    if (!button) return;
    button.onclick = async () => {
      const message = document.getElementById(messageId);
      const endpoint = resolveEndpoint();
      if (!endpoint) return;

      button.disabled = true;
      button.textContent = 'Logging out...';
      try {
        const headers = new Headers();
        if (state.csrfToken) {
          headers.set('X-Shuma-CSRF', state.csrfToken);
        }
        await requestImpl(`${endpoint}/admin/logout`, {
          method: 'POST',
          headers,
          credentials: 'same-origin'
        });
      } catch (_e) {}
      setAdminSession(false);
      if (message) {
        message.textContent = 'Logged out';
        message.className = 'message success';
      }
      button.textContent = 'Logout';
      refreshUiState();
      if (typeof options.redirectToLogin === 'function') {
        options.redirectToLogin();
      }
    };
  };

  return {
    hasValidApiContext,
    getAdminContext,
    restoreAdminSession,
    bindLogoutButton,
    getState: () => ({ ...state })
  };
};
