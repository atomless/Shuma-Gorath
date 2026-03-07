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


if __name__ == "__main__":
    unittest.main()
