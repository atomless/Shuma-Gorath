# SIM2 Realtime Benchmark Summary

- status: PASS
- verification scope: harness_type=synthetic_benchmark runtime_dev=synthetic_contract_check_only runtime_prod=not_verified_by_this_harness claims_runtime_prod_verification=False
- latency percentiles:
  - cursor_polling_default: p50=24499.0ms p95=46099.0ms p99=48039.0ms
  - cursor_polling_fast: p50=124.0ms p95=237.0ms p99=247.0ms
  - sse: p50=99.0ms p95=189.0ms p99=197.0ms
- overflow/drop counts:
  - cursor_polling_default: 14519995
  - cursor_polling_fast: 0
  - sse: 0
- request budget metrics (avg req/sec/client):
  - cursor_polling_default_avg_req_per_sec_client: 1.008
  - cursor_polling_fast_avg_req_per_sec_client: 4.008
  - sse_avg_req_per_sec_client: 0.008
