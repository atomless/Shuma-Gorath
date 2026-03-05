import socket
import ssl
import sys
import unittest
from http.client import RemoteDisconnected
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gateway_failure_harness as harness


class GatewayFailureHarnessTests(unittest.TestCase):
    def test_classify_exception_maps_timeout(self):
        self.assertEqual(harness.classify_exception(TimeoutError()), "timeout")
        self.assertEqual(harness.classify_exception(socket.timeout()), "timeout")

    def test_classify_exception_maps_transport(self):
        self.assertEqual(harness.classify_exception(RemoteDisconnected()), "transport")
        self.assertEqual(harness.classify_exception(OSError("connection reset")), "transport")
        self.assertEqual(harness.classify_exception(ssl.SSLError("tls failed")), "transport")
        self.assertEqual(
            harness.classify_exception(ssl.SSLCertVerificationError("hostname mismatch")),
            "transport",
        )


if __name__ == "__main__":
    unittest.main()
