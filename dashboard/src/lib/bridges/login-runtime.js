let loginMounted = false;

export async function mountLoginRuntime() {
  if (loginMounted) return;

  if (window.__SHUMA_LOGIN_RUNTIME_MOUNTED__) {
    loginMounted = true;
    return;
  }

  await import('../../../login.js');
  window.__SHUMA_LOGIN_RUNTIME_MOUNTED__ = true;
  loginMounted = true;
}

export function unmountLoginRuntime() {
  // Legacy runtime currently has no explicit unmount API in this commit.
}
