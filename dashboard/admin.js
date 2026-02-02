// Admin controls for dashboard
async function banIp(endpoint, apikey, ip, reason = 'admin_ban', duration = 21600) {
  const resp = await fetch(endpoint + '/admin/ban', {
    method: 'POST',
    headers: { 'Authorization': 'Bearer ' + apikey, 'Content-Type': 'application/json' },
    body: JSON.stringify({ ip, reason, duration })
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
    headers: { 'Authorization': 'Bearer ' + apikey }
  });
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(`Unban failed: ${resp.status} ${text}`);
  }
  return await resp.text();
}

window.banIp = banIp;
window.unbanIp = unbanIp;
