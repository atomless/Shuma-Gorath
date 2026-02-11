# 🐙 Observability & Grafana

## 🐙 Prometheus Metrics Endpoint

Shuma-Gorath exposes Prometheus-compatible metrics at:

```
GET /metrics
```

This endpoint is unauthenticated for Prometheus compatibility. Restrict access at the network edge if required.

### 🐙 Metrics Included

- `bot_defence_requests_total`
- `bot_defence_bans_total{reason="..."}`
- `bot_defence_blocks_total`
- `bot_defence_challenges_total`
- `bot_defence_challenge_served_total`
- `bot_defence_challenge_solved_total`
- `bot_defence_challenge_incorrect_total`
- `bot_defence_challenge_expired_replay_total`
- `bot_defence_whitelisted_total`
- `bot_defence_test_mode_actions_total`
- `bot_defence_maze_hits_total`
- `bot_defence_active_bans`
- `bot_defence_test_mode_enabled`

## 🐙 Prometheus Scrape Example

```yaml
scrape_configs:
  - job_name: shuma-gorath
    static_configs:
      - targets: ["your-domain.example.com"]
    metrics_path: /metrics
```

## 🐙 Grafana Integration

1. Add Prometheus as a data source
2. Build panels for requests total, bans by reason, active bans, challenges/blocks over time, and test mode status

## 🐙 Spin Cloud Monitoring

```bash
spin cloud logs
spin cloud apps info
spin cloud apps metrics
```
