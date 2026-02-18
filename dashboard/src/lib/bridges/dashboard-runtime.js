let dashboardMounted = false;

export async function mountDashboardRuntime() {
  if (dashboardMounted) return;

  if (window.__SHUMA_DASHBOARD_RUNTIME_MOUNTED__) {
    dashboardMounted = true;
    return;
  }

  await import('../../../dashboard.js');
  window.__SHUMA_DASHBOARD_RUNTIME_MOUNTED__ = true;
  dashboardMounted = true;
}

export function unmountDashboardRuntime() {
  // Legacy runtime currently has no explicit unmount API in this commit.
}
