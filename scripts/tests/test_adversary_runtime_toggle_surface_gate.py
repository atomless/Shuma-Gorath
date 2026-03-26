import unittest

import scripts.tests.adversary_runtime_toggle_surface_gate as runtime_surface_gate


class _Response:
    status = 200

    def read(self) -> bytes:
        return b"ok"

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False


class _Opener:
    def __init__(self) -> None:
        self.requests = []

    def open(self, req, timeout=5):
        self.requests.append({str(key).lower(): str(value) for key, value in req.header_items()})
        return _Response()


class RuntimeToggleSurfaceGateTests(unittest.TestCase):
    def test_health_probe_includes_health_secret_when_configured(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=10,
        )
        opener = _Opener()
        gate.opener = opener

        gate.ensure_health()

        self.assertEqual(opener.requests[0]["x-shuma-health-secret"], "health-secret")
        self.assertEqual(opener.requests[0]["x-shuma-forwarded-secret"], "forwarded-secret")
        self.assertEqual(opener.requests[0]["x-forwarded-for"], "127.0.0.1")

    def test_recent_scrapling_run_coverage_reads_operator_snapshot_recent_runs(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )

        operator_snapshot_body = {
            "adversary_sim": {
                "recent_runs": [
                    {
                        "run_id": "sim-run-001",
                        "lane": "scrapling_traffic",
                        "profile": "scrapling_traffic.bulk_scraper",
                        "observed_fulfillment_modes": ["bulk_scraper"],
                        "owned_surface_coverage": {
                            "overall_status": "covered",
                            "required_surface_ids": [
                                "challenge_routing",
                                "not_a_bot_submit",
                                "puzzle_submit_or_escalation",
                            ],
                            "blocking_surface_ids": [],
                        },
                    }
                ]
            },
            "objectives": {"profile_id": "human_only_private"},
            "verified_identity": {
                "effective_non_human_policy": {
                    "verified_identity_override_mode": "strict_human_only"
                }
            },
            "budget_distance": {
                "rows": [
                    {
                        "metric": "suspicious_forwarded_request_rate",
                        "target": 0.0,
                    },
                    {
                        "metric": "suspicious_forwarded_byte_rate",
                        "target": 0.0,
                    },
                    {
                        "metric": "suspicious_forwarded_latency_share",
                        "target": 0.0,
                    },
                ]
            },
        }

        def fake_request(method, path, payload=None, extra_headers=None):
            self.assertEqual(method, "GET")
            self.assertIn("/admin/operator-snapshot", path)
            return {"status": 200, "body": operator_snapshot_body, "raw": ""}

        gate.request = fake_request

        coverage = gate.poll_recent_scrapling_run_coverage()

        self.assertEqual(coverage["run_id"], "sim-run-001")
        self.assertEqual(coverage["overall_status"], "covered")
        self.assertEqual(coverage["profile_id"], "human_only_private")
        self.assertEqual(coverage["verified_identity_override_mode"], "strict_human_only")
        self.assertEqual(coverage["suspicious_forwarded_request_target"], 0.0)
        self.assertEqual(coverage["suspicious_forwarded_byte_target"], 0.0)
        self.assertEqual(coverage["suspicious_forwarded_latency_target"], 0.0)
        self.assertEqual(
            coverage["required_surface_ids"],
            [
                "challenge_routing",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
            ],
        )
        self.assertEqual(coverage["observed_fulfillment_modes"], ["bulk_scraper"])

    def test_configure_runtime_surface_profile_preserves_public_pass_headroom_and_restores_core_defenses(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )

        captured = {}

        def fake_request(method, path, payload=None, extra_headers=None):
            captured["method"] = method
            captured["path"] = path
            captured["payload"] = payload
            return {"status": 200, "body": {}, "raw": ""}

        gate.request = fake_request

        gate.configure_runtime_surface_profile()

        self.assertEqual(captured["method"], "POST")
        self.assertEqual(captured["path"], "/admin/config")
        self.assertEqual(captured["payload"]["defence_modes"]["rate"], "both")
        self.assertEqual(captured["payload"]["rate_limit"], 80)
        self.assertTrue(captured["payload"]["pow_enabled"])
        self.assertTrue(captured["payload"]["challenge_puzzle_enabled"])
        self.assertTrue(captured["payload"]["not_a_bot_enabled"])
        self.assertFalse(captured["payload"]["maze_auto_ban"])
        self.assertTrue(captured["payload"]["geo_edge_headers_enabled"])
        self.assertEqual(captured["payload"]["geo_challenge"], ["RU"])
        self.assertEqual(captured["payload"]["ban_durations"]["rate_limit"], 1)
        self.assertEqual(captured["payload"]["ban_durations"]["tarpit_persistence"], 1)

    def test_clear_loopback_bans_posts_unban_for_loopback_and_unknown_identities(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )

        seen: list[tuple[str, str]] = []

        def fake_request(method, path, payload=None, extra_headers=None):
            seen.append((method, path))
            return {"status": 200, "body": {}, "raw": ""}

        gate.request = fake_request

        gate.clear_loopback_bans()

        self.assertEqual(
            seen,
            [
                ("POST", "/admin/unban?ip=127.0.0.1"),
                ("POST", "/admin/unban?ip=::1"),
                ("POST", "/admin/unban?ip=unknown"),
            ],
        )

    def test_poll_recent_scrapling_run_coverage_waits_for_covered_recent_run(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )

        responses = iter(
            [
                {
                    "status": 200,
                    "body": {
                        "adversary_sim": {
                            "recent_runs": [
                                {
                                    "run_id": "sim-run-001",
                                    "lane": "scrapling_traffic",
                                    "observed_fulfillment_modes": ["crawler"],
                                    "owned_surface_coverage": {
                                        "overall_status": "partial",
                                        "required_surface_ids": ["challenge_routing"],
                                        "blocking_surface_ids": ["challenge_routing"],
                                    },
                                }
                            ]
                        },
                        "objectives": {"profile_id": "human_only_private"},
                        "verified_identity": {
                            "effective_non_human_policy": {
                                "verified_identity_override_mode": "strict_human_only"
                            }
                        },
                        "budget_distance": {
                            "rows": [
                                {
                                    "metric": "suspicious_forwarded_request_rate",
                                    "target": 0.0,
                                },
                                {
                                    "metric": "suspicious_forwarded_byte_rate",
                                    "target": 0.0,
                                },
                                {
                                    "metric": "suspicious_forwarded_latency_share",
                                    "target": 0.0,
                                },
                            ]
                        },
                    },
                    "raw": "",
                },
                {
                    "status": 200,
                    "body": {
                        "adversary_sim": {
                            "recent_runs": [
                                {
                                    "run_id": "sim-run-002",
                                    "lane": "scrapling_traffic",
                                    "observed_fulfillment_modes": ["http_agent"],
                                    "owned_surface_coverage": {
                                        "overall_status": "covered",
                                        "required_surface_ids": ["pow_verify_abuse"],
                                        "blocking_surface_ids": [],
                                    },
                                }
                            ]
                        },
                        "objectives": {"profile_id": "human_only_private"},
                        "verified_identity": {
                            "effective_non_human_policy": {
                                "verified_identity_override_mode": "strict_human_only"
                            }
                        },
                        "budget_distance": {
                            "rows": [
                                {
                                    "metric": "suspicious_forwarded_request_rate",
                                    "target": 0.0,
                                },
                                {
                                    "metric": "suspicious_forwarded_byte_rate",
                                    "target": 0.0,
                                },
                                {
                                    "metric": "suspicious_forwarded_latency_share",
                                    "target": 0.0,
                                },
                            ]
                        },
                    },
                    "raw": "",
                },
            ]
        )

        def fake_request(method, path, payload=None, extra_headers=None):
            self.assertEqual(method, "GET")
            self.assertIn("/admin/operator-snapshot", path)
            return next(responses)

        gate.request = fake_request

        coverage = gate.poll_recent_scrapling_run_coverage()

        self.assertEqual(coverage["run_id"], "sim-run-002")
        self.assertEqual(coverage["overall_status"], "covered")
        self.assertEqual(coverage["profile_id"], "human_only_private")
        self.assertEqual(coverage["verified_identity_override_mode"], "strict_human_only")
        self.assertEqual(coverage["observed_fulfillment_modes"], ["http_agent"])

    def test_poll_post_sim_oversight_run_waits_for_matching_completed_sim_run(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )

        responses = iter(
            [
                {
                    "status": 200,
                    "body": {
                        "recent_runs": [
                            {
                                "run_id": "ovragent-other",
                                "trigger_kind": "post_adversary_sim",
                                "sim_run_id": "sim-run-other",
                                "execution": {"apply": {"stage": "refused"}},
                            }
                        ]
                    },
                    "raw": "",
                },
                {
                    "status": 200,
                    "body": {
                        "recent_runs": [
                            {
                                "run_id": "ovragent-post-sim-1",
                                "trigger_kind": "post_adversary_sim",
                                "sim_run_id": "sim-run-002",
                                "execution": {"apply": {"stage": "canary_applied"}},
                            }
                        ]
                    },
                    "raw": "",
                },
            ]
        )

        def fake_request(method, path, payload=None, extra_headers=None):
            self.assertEqual(method, "GET")
            self.assertEqual(path, "/admin/oversight/agent/status")
            return next(responses)

        gate.request = fake_request

        oversight_run = gate.poll_post_sim_oversight_run("sim-run-002")

        self.assertEqual(oversight_run["run_id"], "ovragent-post-sim-1")
        self.assertEqual(oversight_run["trigger_kind"], "post_adversary_sim")
        self.assertEqual(oversight_run["sim_run_id"], "sim-run-002")
        self.assertEqual(oversight_run["apply_stage"], "canary_applied")

    def test_live_summary_counts_read_live_only_summary_paths(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=1,
        )

        counts = gate.live_summary_counts(
            {
                "summary": {
                    "challenge": {"total_failures": 2},
                    "pow": {"total_attempts": 3},
                    "rate": {"total_violations": 4},
                    "geo": {"total_violations": 5},
                },
                "details": {
                    "events": {
                        "recent_events": [
                            {"is_simulation": True, "event": "Challenge", "reason": "sim_event"}
                        ]
                    }
                },
            }
        )

        self.assertEqual(
            counts,
            {
                "challenge_failures": 2,
                "pow_attempts": 3,
                "rate_violations": 4,
                "geo_violations": 5,
            },
        )

    def test_live_summary_leaks_only_report_positive_delta_above_baseline(self) -> None:
        leaked = runtime_surface_gate.live_summary_leaks(
            current={
                "challenge_failures": 2,
                "pow_attempts": 4,
                "rate_violations": 3,
                "geo_violations": 3,
            },
            baseline={
                "challenge_failures": 2,
                "pow_attempts": 4,
                "rate_violations": 3,
                "geo_violations": 3,
            },
        )
        self.assertEqual(leaked, {})

        leaked = runtime_surface_gate.live_summary_leaks(
            current={
                "challenge_failures": 3,
                "pow_attempts": 4,
                "rate_violations": 5,
                "geo_violations": 3,
            },
            baseline={
                "challenge_failures": 2,
                "pow_attempts": 4,
                "rate_violations": 3,
                "geo_violations": 3,
            },
        )
        self.assertEqual(
            leaked,
            {
                "challenge_failures": 1,
                "rate_violations": 2,
            },
        )

    def test_poll_live_summary_matches_baseline_waits_for_baseline_live_only_counts(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )
        baseline = {
            "challenge_failures": 1,
            "pow_attempts": 2,
            "rate_violations": 3,
            "geo_violations": 4,
        }

        responses = iter(
            [
                {
                    "status": 200,
                    "body": {
                        "summary": {
                            "challenge": {"total_failures": 2},
                            "pow": {"total_attempts": 2},
                            "rate": {"total_violations": 3},
                            "geo": {"total_violations": 4},
                        },
                        "details": {
                            "events": {
                                "recent_events": [
                                    {"is_simulation": True, "event": "Challenge", "reason": "sim_event"}
                                ]
                            }
                        },
                    },
                    "raw": "",
                },
                {
                    "status": 200,
                    "body": {
                        "summary": {
                            "challenge": {"total_failures": 1},
                            "pow": {"total_attempts": 2},
                            "rate": {"total_violations": 3},
                            "geo": {"total_violations": 4},
                        }
                    },
                    "raw": "",
                },
            ]
        )

        def fake_request(method, path, payload=None, extra_headers=None):
            self.assertEqual(method, "GET")
            self.assertIn("/admin/monitoring", path)
            return next(responses)

        gate.request = fake_request

        counts = gate.poll_live_summary_matches_baseline(baseline)

        self.assertEqual(counts, baseline)


if __name__ == "__main__":
    unittest.main()
