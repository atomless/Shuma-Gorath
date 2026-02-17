// @ts-check

import * as format from './core/format.js';
import * as domModule from './core/dom.js';

export const create = (options = {}) => {
  const escapeHtml = typeof options.escapeHtml === 'function'
    ? options.escapeHtml
    : format.escapeHtml;
  const onQuickUnban = typeof options.onQuickUnban === 'function'
    ? options.onQuickUnban
    : async () => {};
  const domCache = domModule.createCache({ document });
  const query = domCache.query;
  const queryAll = domCache.queryAll;
  const byId = domCache.byId;

  const updateBansTable = (bans) => {
    const tbody = query('#bans-table tbody');
    if (!tbody) return;
    domModule.setHtml(tbody, '');

    if (!Array.isArray(bans) || bans.length === 0) {
      domModule.setHtml(
        tbody,
        '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No active bans</td></tr>'
      );
      return;
    }

    for (const ban of bans) {
      const tr = document.createElement('tr');
      const now = Math.floor(Date.now() / 1000);
      const isExpired = Number(ban.expires || 0) < now;
      const bannedAt = ban.banned_at ? new Date(ban.banned_at * 1000).toLocaleString() : '-';
      const expiresAt = new Date(Number(ban.expires || 0) * 1000).toLocaleString();
      const safeIp = escapeHtml(ban.ip || '-');
      const safeReason = escapeHtml(ban.reason || 'unknown');
      const signals = (ban.fingerprint && Array.isArray(ban.fingerprint.signals))
        ? ban.fingerprint.signals
        : [];
      const signalBadges = signals.length
        ? signals.map((signal) => `<span class="ban-signal-badge">${escapeHtml(signal)}</span>`).join('')
        : '<span class="text-muted">none</span>';
      const detailsId = `ban-detail-${String(ban.ip || 'unknown').replace(/[^a-zA-Z0-9]/g, '-')}`;

      tr.innerHTML = `
      <td><code>${safeIp}</code></td>
      <td>${safeReason}</td>
      <td>${bannedAt}</td>
      <td class="${isExpired ? 'expired' : ''}">${isExpired ? 'Expired' : expiresAt}</td>
      <td>${signalBadges}</td>
      <td class="ban-action-cell">
        <button class="ban-details-toggle" data-target="${detailsId}">Details</button>
        <button class="unban-quick" data-ip="${ban.ip}">Unban</button>
      </td>
    `;
      tbody.appendChild(tr);

      const detailRow = document.createElement('tr');
      detailRow.id = detailsId;
      detailRow.className = 'ban-detail-row hidden';
      const score =
        ban.fingerprint && typeof ban.fingerprint.score === 'number' ? ban.fingerprint.score : null;
      const summary = ban.fingerprint && ban.fingerprint.summary
        ? ban.fingerprint.summary
        : 'No additional fingerprint details.';
      const safeSummary = escapeHtml(summary);
      detailRow.innerHTML = `
      <td colspan="6">
        <div class="ban-detail-content">
          <div><strong>Score:</strong> ${score === null ? 'n/a' : score}</div>
          <div><strong>Summary:</strong> ${safeSummary}</div>
        </div>
      </td>
    `;
      tbody.appendChild(detailRow);
    }

    queryAll('.ban-details-toggle').forEach((btn) => {
      btn.onclick = () => {
        const target = byId(btn.dataset.target);
        if (!target) return;
        target.classList.toggle('hidden');
        btn.textContent = target.classList.contains('hidden') ? 'Details' : 'Hide';
      };
    });

    queryAll('.unban-quick').forEach((btn) => {
      btn.onclick = async () => {
        const ip = btn.dataset.ip || '';
        await onQuickUnban(ip);
      };
    });
  };

  const updateEventsTable = (events) => {
    const tbody = query('#events tbody');
    if (!tbody) return;
    domModule.setHtml(tbody, '');

    if (!Array.isArray(events) || events.length === 0) {
      domModule.setHtml(
        tbody,
        '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No recent events</td></tr>'
      );
      return;
    }

    for (const ev of events) {
      const tr = document.createElement('tr');
      const eventClass = String(ev.event || '').toLowerCase().replace(/[^a-z_]/g, '');
      const safeEvent = escapeHtml(ev.event || '-');
      const safeIp = escapeHtml(ev.ip || '-');
      const safeReason = escapeHtml(ev.reason || '-');
      const safeOutcome = escapeHtml(ev.outcome || '-');
      const safeAdmin = escapeHtml(ev.admin || '-');
      tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><span class="badge ${eventClass}">${safeEvent}</span></td>
      <td><code>${safeIp}</code></td>
      <td>${safeReason}</td>
      <td>${safeOutcome}</td>
      <td>${safeAdmin}</td>
    `;
      tbody.appendChild(tr);
    }
  };

  const extractCdpField = (text, key) => {
    const match = new RegExp(`${key}=([^\\s]+)`, 'i').exec(text || '');
    return match ? match[1] : '-';
  };

  const updateCdpEventsTable = (events) => {
    const tbody = query('#cdp-events tbody');
    if (!tbody) return;
    domModule.setHtml(tbody, '');

    const cdpEvents = Array.isArray(events) ? events : [];
    if (cdpEvents.length === 0) {
      domModule.setHtml(
        tbody,
        '<tr><td colspan="6" style="text-align: center; color: #6b7280;">No CDP detections or auto-bans in the selected window</td></tr>'
      );
      return;
    }

    for (const ev of cdpEvents) {
      const reason = ev.reason || '';
      const reasonLower = reason.toLowerCase();
      const outcome = ev.outcome || '-';
      const isBan = reasonLower === 'cdp_automation';
      const tierSource = isBan ? outcome : reason;
      const tier = extractCdpField(tierSource, 'tier').toUpperCase();
      const score = extractCdpField(tierSource, 'score');
      const details = isBan
        ? `Auto-ban: ${outcome}`
        : (outcome.toLowerCase().startsWith('checks:') ? outcome.replace(/^checks:/i, 'Checks: ') : outcome);

      const tr = document.createElement('tr');
      const safeIp = escapeHtml(ev.ip || '-');
      const safeTier = escapeHtml(tier);
      const safeScore = escapeHtml(score);
      const safeDetails = escapeHtml(details);
      tr.innerHTML = `
      <td>${new Date(ev.ts * 1000).toLocaleString()}</td>
      <td><code>${safeIp}</code></td>
      <td><span class="badge ${isBan ? 'ban' : 'challenge'}">${isBan ? 'BAN' : 'DETECTION'}</span></td>
      <td>${safeTier}</td>
      <td>${safeScore}</td>
      <td>${safeDetails}</td>
    `;
      tbody.appendChild(tr);
    }
  };

  const updateCdpTotals = (cdpData) => {
    const detections = cdpData?.stats?.total_detections ?? 0;
    const autoBans = cdpData?.stats?.auto_bans ?? 0;
    const fingerprintEvents =
      (cdpData?.fingerprint_stats?.ua_client_hint_mismatch ?? 0) +
      (cdpData?.fingerprint_stats?.ua_transport_mismatch ?? 0) +
      (cdpData?.fingerprint_stats?.temporal_transition ?? 0);
    const fingerprintFlowViolations = cdpData?.fingerprint_stats?.flow_violation ?? 0;

    domModule.setText(byId('cdp-total-detections'), format.formatNumber(detections, '0'));
    domModule.setText(byId('cdp-total-auto-bans'), format.formatNumber(autoBans, '0'));
    domModule.setText(byId('cdp-fp-events'), format.formatNumber(fingerprintEvents, '0'));
    domModule.setText(byId('cdp-fp-flow-violations'), format.formatNumber(fingerprintFlowViolations, '0'));
  };

  return {
    updateBansTable,
    updateEventsTable,
    updateCdpEventsTable,
    updateCdpTotals,
    _extractCdpField: extractCdpField
  };
};
