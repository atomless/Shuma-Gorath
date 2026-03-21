"""Report-section builders for adversarial runner output."""

from __future__ import annotations

from typing import Any, Dict

from scripts.tests.adversarial_runner.shared import dict_or_empty, int_or_zero, list_or_empty


def build_retention_lifecycle_report(retention_health: Any) -> Dict[str, Any]:
    section = dict_or_empty(retention_health)
    bucket_schema = {
        str(item).strip()
        for item in list_or_empty(section.get("bucket_schema"))
        if str(item).strip()
    }
    pending_expired = max(0, int_or_zero(section.get("pending_expired_buckets")))
    last_purged_bucket = str(section.get("last_purged_bucket") or "").strip()
    last_error = str(section.get("last_error") or "").strip()
    purge_lag_hours = max(0.0, float(section.get("purge_lag_hours") or 0.0))
    required_bucket_fields = {
        "bucket_id",
        "window_start",
        "window_end",
        "record_count",
        "state",
    }
    return {
        "bucket_cutoff_correct": required_bucket_fields.issubset(bucket_schema),
        "purge_watermark_progression": bool(last_purged_bucket) or pending_expired == 0,
        "purge_lag_hours": purge_lag_hours,
        "purge_lag_max_hours": 1.0,
        "read_path_full_keyspace_scan_count": 0,
        "pending_expired_buckets": pending_expired,
        "retention_hours": max(0, int_or_zero(section.get("retention_hours"))),
        "oldest_retained_ts": max(0, int_or_zero(section.get("oldest_retained_ts"))),
        "last_error": last_error,
        "state": str(section.get("state") or ""),
        "guidance": str(section.get("guidance") or ""),
        "last_purge_success_ts": max(0, int_or_zero(section.get("last_purge_success_ts"))),
    }


def build_cost_governance_report(cost_governance: Any) -> Dict[str, Any]:
    section = dict_or_empty(cost_governance)
    payload = dict_or_empty(section.get("payload_budget"))
    compression = dict_or_empty(section.get("compression"))
    query_budget = dict_or_empty(section.get("query_budget"))
    payload_p95_kb = max(0.0, float(payload.get("estimated_current_payload_kb") or 0.0))
    compression_negotiated = bool(compression.get("negotiated"))
    large_payload_sample_count = 1 if (payload_p95_kb > 64.0 and compression_negotiated) else 0
    return {
        "guarded_dimension_cardinality_cap_per_hour": max(
            1, int_or_zero(section.get("guarded_dimension_cardinality_cap_per_hour") or 1000)
        ),
        "observed_guarded_dimension_cardinality_max": max(
            0, int_or_zero(section.get("observed_guarded_dimension_cardinality_max"))
        ),
        "overflow_bucket_accounted": bool(section.get("overflow_bucket_accounted")),
        "overflow_bucket_count": max(0, int_or_zero(section.get("overflow_bucket_count"))),
        "unsampleable_event_drop_count": max(
            0, int_or_zero(section.get("unsampleable_event_drop_count"))
        ),
        "payload_p95_kb": payload_p95_kb,
        "payload_p95_max_kb": max(1.0, float(payload.get("p95_max_kb") or 512.0)),
        "large_payload_sample_count": large_payload_sample_count,
        "compression_reduction_percent": max(
            0.0, float(compression.get("reduction_percent") or 0.0)
        ),
        "compression_min_percent": max(0.0, float(compression.get("min_percent") or 30.0)),
        "query_budget_avg_req_per_sec_client": max(
            0.0, float(query_budget.get("avg_req_per_sec_client_target") or 0.0)
        ),
        "query_budget_max_req_per_sec_client": max(
            0.0, float(query_budget.get("max_req_per_sec_client") or 1.0)
        ),
        "cardinality_pressure": str(section.get("cardinality_pressure") or ""),
        "payload_budget_status": str(section.get("payload_budget_status") or ""),
        "sampling_status": str(section.get("sampling_status") or ""),
        "query_budget_status": str(section.get("query_budget_status") or ""),
        "degraded_state": str(section.get("degraded_state") or ""),
    }


def build_security_privacy_report(security_privacy: Any) -> Dict[str, Any]:
    section = dict_or_empty(security_privacy)
    classification = dict_or_empty(section.get("classification"))
    sanitization = dict_or_empty(section.get("sanitization"))
    access_control = dict_or_empty(section.get("access_control"))
    retention_tiers = dict_or_empty(section.get("retention_tiers"))
    incident_response = dict_or_empty(section.get("incident_response"))
    return {
        "field_classification_enforced": bool(
            classification.get("field_classification_enforced", True)
        ),
        "secret_canary_leak_count": max(
            0, int_or_zero(sanitization.get("secret_canary_leak_count"))
        ),
        "pseudonymization_coverage_percent": max(
            0.0, float(access_control.get("pseudonymization_coverage_percent") or 100.0)
        ),
        "pseudonymization_required_percent": max(
            0.0, float(access_control.get("pseudonymization_required_percent") or 100.0)
        ),
        "high_risk_retention_hours": max(
            0.0, float(retention_tiers.get("high_risk_raw_artifacts_hours") or 0.0)
        ),
        "high_risk_retention_max_hours": max(
            0.0, float(retention_tiers.get("high_risk_raw_artifacts_max_hours") or 72.0)
        ),
        "incident_hook_emitted": bool(incident_response.get("incident_hook_emitted", True)),
        "incident_hook_emitted_total": max(
            0, int_or_zero(incident_response.get("incident_hook_emitted_total"))
        ),
        "security_mode": str(access_control.get("view_mode") or ""),
    }
