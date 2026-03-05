import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "deploy"))
import probe_gateway_origin_bypass as probe


class ProbeGatewayOriginBypassUnitTests(unittest.TestCase):
    def test_classify_inconclusive_when_gateway_probe_fails(self) -> None:
        gateway = probe.ProbeResponse(status_code=None, error_class="transport", detail="conn reset")
        direct = probe.ProbeResponse(status_code=200, error_class=None, detail="ok")
        decision, reason = probe.classify_origin_bypass(gateway, direct)
        self.assertEqual(decision, "inconclusive")
        self.assertIn("gateway", reason)

    def test_classify_protected_when_direct_origin_unreachable(self) -> None:
        gateway = probe.ProbeResponse(status_code=200, error_class=None, detail="ok")
        direct = probe.ProbeResponse(status_code=None, error_class="transport", detail="conn refused")
        decision, reason = probe.classify_origin_bypass(gateway, direct)
        self.assertEqual(decision, "protected")
        self.assertIn("unreachable", reason)

    def test_classify_protected_when_direct_origin_denies_request(self) -> None:
        gateway = probe.ProbeResponse(status_code=200, error_class=None, detail="ok")
        direct = probe.ProbeResponse(status_code=403, error_class=None, detail="forbidden")
        decision, reason = probe.classify_origin_bypass(gateway, direct)
        self.assertEqual(decision, "protected")
        self.assertIn("deny", reason)

    def test_classify_exposed_when_direct_origin_is_reachable(self) -> None:
        gateway = probe.ProbeResponse(status_code=200, error_class=None, detail="ok")
        direct = probe.ProbeResponse(status_code=200, error_class=None, detail="ok")
        decision, reason = probe.classify_origin_bypass(gateway, direct)
        self.assertEqual(decision, "exposed")
        self.assertIn("reachable", reason)


if __name__ == "__main__":
    unittest.main()
