const DEFAULT_BASE_URL = "http://127.0.0.1:3000";
const DEFAULT_API_KEY = "changeme-dev-only-api-key";
const DEFAULT_FORWARDED_SECRET = "changeme-dev-only-ip-secret";

function authHeaders() {
  return {
    Authorization: `Bearer ${process.env.SHUMA_API_KEY || DEFAULT_API_KEY}`
  };
}

function forwardedHeaders(ip) {
  const headers = {
    "X-Forwarded-For": ip
  };
  const secret = process.env.SHUMA_FORWARDED_IP_SECRET || DEFAULT_FORWARDED_SECRET;
  if (secret) {
    headers["X-Shuma-Forwarded-Secret"] = secret;
  }
  return headers;
}

async function request(baseURL, path, options = {}) {
  const response = await fetch(`${baseURL}${path}`, options);
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`Seed request failed: ${options.method || "GET"} ${path} -> ${response.status} ${text}`);
  }
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

async function seedDashboardData() {
  const baseURL = process.env.SHUMA_BASE_URL || DEFAULT_BASE_URL;
  const now = Date.now();
  const banIp = `203.0.113.${(now % 200) + 20}`;
  const cdpIp = `198.51.100.${(Math.floor(now / 7) % 200) + 20}`;

  await request(baseURL, "/admin/config", {
    method: "POST",
    headers: {
      ...authHeaders(),
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ test_mode: false })
  });

  await request(baseURL, "/admin/ban", {
    method: "POST",
    headers: {
      ...authHeaders(),
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      ip: banIp,
      duration: 3600
    })
  });

  await request(baseURL, "/cdp-report", {
    method: "POST",
    headers: {
      ...forwardedHeaders(cdpIp),
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      cdp_detected: true,
      score: 0.92,
      checks: ["webdriver", "runtime_enable"]
    })
  });

  await request(baseURL, "/admin/analytics", {
    headers: authHeaders()
  });
  await request(baseURL, "/admin/events?hours=24", {
    headers: authHeaders()
  });
  const events = await request(baseURL, "/admin/events?hours=24", {
    headers: authHeaders()
  });

  if (!events || !Array.isArray(events.recent_events) || events.recent_events.length === 0) {
    throw new Error("Seed verification failed: no recent events available");
  }

  return {
    banIp,
    cdpIp,
    eventCount: events.recent_events.length
  };
}

module.exports = { seedDashboardData };

if (require.main === module) {
  seedDashboardData()
    .then((result) => {
      process.stdout.write(
        `Dashboard seed complete: banIp=${result.banIp}, cdpIp=${result.cdpIp}, events=${result.eventCount}\n`
      );
    })
    .catch((err) => {
      process.stderr.write(`Dashboard seed failed: ${err.message}\n`);
      process.exit(1);
    });
}
