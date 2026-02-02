<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Bot Trap Dashboard</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div class="container">
    <h1>Bot Trap Dashboard</h1>
    <div class="config">
      <label>API Endpoint: <input id="endpoint" value="http://127.0.0.1:3000" size="30"></label>
      <label>API Key: <input id="apikey" value="changeme-supersecret" size="30" type="password"></label>
      <button id="refresh">Refresh</button>
    </div>
    <div class="stats">
      <h2>Ban Analytics</h2>
      <pre id="analytics">Loading...</pre>
    </div>
    <div class="events">
      <h2>Recent Events</h2>
      <table id="events">
        <thead>
          <tr><th>Time</th><th>Type</th><th>IP</th><th>Reason</th><th>Outcome</th><th>Admin</th></tr>
        </thead>
        <tbody></tbody>
      </table>
    </div>
    <div class="top-ips">
      <h2>Top IPs</h2>
      <ul id="top-ips"></ul>
    </div>
  </div>
  <script src="dashboard.js"></script>
</body>
</html>
