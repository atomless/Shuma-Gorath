// Admin controls for dashboard
function authHeaders(apikey, includeJson = false) {
  const headers = {};
  const trimmed = (apikey || '').trim();
  if (trimmed) {
    headers.Authorization = 'Bearer ' + trimmed;
  }
  if (includeJson) {
    headers['Content-Type'] = 'application/json';
  }
  return headers;
}

async function banIp(endpoint, apikey, ip, duration = 21600) {
  const resp = await fetch(endpoint + '/admin/ban', {
    method: 'POST',
    headers: authHeaders(apikey, true),
    body: JSON.stringify({ ip, reason: 'manual_ban', duration })
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Ban failed: ${resp.status} ${text}`);
  }
  return await resp.json();
}

async function unbanIp(endpoint, apikey, ip) {
  const resp = await fetch(endpoint + `/admin/unban?ip=${encodeURIComponent(ip)}`, {
    method: 'POST',
    headers: authHeaders(apikey)
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Unban failed: ${resp.status} ${text}`);
  }
  return await resp.text();
}

async function getConfig(endpoint, apikey) {
  const resp = await fetch(endpoint + '/admin/config', {
    method: 'GET',
    headers: authHeaders(apikey)
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Get config failed: ${resp.status} ${text}`);
  }
  return await resp.json();
}

async function updateConfig(endpoint, apikey, config) {
  const resp = await fetch(endpoint + '/admin/config', {
    method: 'POST',
    headers: authHeaders(apikey, true),
    body: JSON.stringify(config)
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Update config failed: ${resp.status} ${text}`);
  }
  return await resp.json();
}

async function setTestMode(endpoint, apikey, enabled) {
  return await updateConfig(endpoint, apikey, { test_mode: enabled });
}

window.banIp = banIp;
window.unbanIp = unbanIp;
window.getConfig = getConfig;
window.updateConfig = updateConfig;
window.setTestMode = setTestMode;
