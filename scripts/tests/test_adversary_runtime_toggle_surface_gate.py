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

    def test_poll_categories_reads_js_required_from_taxonomy_signals(self) -> None:
        gate = runtime_surface_gate.RuntimeToggleSurfaceGate(
            base_url="http://127.0.0.1:3000",
            api_key="test-api-key",
            forwarded_secret="forwarded-secret",
            health_secret="health-secret",
            timeout_seconds=2,
        )

        monitoring_body = {
            "summary": {
                "pow": {"total_attempts": 1},
                "rate": {"total_violations": 1},
                "geo": {"total_violations": 1},
            },
            "details": {
                "analytics": {"ban_count": 1},
                "cdp": {
                    "stats": {"total_detections": 1},
                    "fingerprint_stats": {"events": 1},
                },
                "tarpit": {"metrics": {"activations": {"progressive": 1}}},
                "events": {
                    "recent_events": [
                        {
                            "event": "Challenge",
                            "reason": "botness_gate_maze",
                            "outcome": None,
                            "taxonomy": {
                                "level": "L7_DECEPTION_EXPLICIT",
                                "signals": ["S_JS_REQUIRED_MISSING"],
                            },
                            "is_simulation": True,
                        }
                    ]
                },
            },
        }

        def fake_request(method, path, payload=None, extra_headers=None):
            self.assertEqual(method, "GET")
            self.assertIn("/admin/monitoring", path)
            return {"status": 200, "body": monitoring_body, "raw": ""}

        gate.request = fake_request

        seen = gate.poll_categories()

        self.assertTrue(seen["challenge"])
        self.assertTrue(seen["js_required"])
        self.assertTrue(all(seen.values()))

    def test_configure_js_required_profile_disables_geo_and_not_a_bot_preemption(self) -> None:
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

        gate.configure_js_required_profile()

        self.assertEqual(captured["method"], "POST")
        self.assertEqual(captured["path"], "/admin/config")
        self.assertEqual(captured["payload"]["defence_modes"]["rate"], "signal")
        self.assertEqual(captured["payload"]["rate_limit"], 1000)
        self.assertTrue(captured["payload"]["js_required_enforced"])
        self.assertFalse(captured["payload"]["not_a_bot_enabled"])
        self.assertFalse(captured["payload"]["geo_edge_headers_enabled"])
        self.assertEqual(captured["payload"]["geo_challenge"], [])

    def test_configure_runtime_surface_profile_enables_actual_rate_limit_signal(self) -> None:
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
        self.assertEqual(captured["payload"]["rate_limit"], 6)
        self.assertTrue(captured["payload"]["not_a_bot_enabled"])
        self.assertTrue(captured["payload"]["geo_edge_headers_enabled"])
        self.assertEqual(captured["payload"]["geo_challenge"], ["RU"])

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

    def test_poll_live_summary_clean_waits_for_zero_live_only_counts(self) -> None:
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
                        "summary": {"challenge": {"total_failures": 1}},
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
                            "challenge": {"total_failures": 0},
                            "pow": {"total_attempts": 0},
                            "rate": {"total_violations": 0},
                            "geo": {"total_violations": 0},
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

        counts = gate.poll_live_summary_clean()

        self.assertEqual(
            counts,
            {
                "challenge_failures": 0,
                "pow_attempts": 0,
                "rate_violations": 0,
                "geo_violations": 0,
            },
        )


if __name__ == "__main__":
    unittest.main()
