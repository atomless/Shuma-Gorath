#!/usr/bin/env python3

from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading
import unittest
import urllib.error
import urllib.request

from scripts.supervisor.trusted_ingress_proxy import (
    TrustedIngressProxyConfig,
    build_proxy_handler,
)


class _RecordingOriginHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):  # noqa: A003
        return

    def do_GET(self):  # noqa: N802
        self.server.last_request = {  # type: ignore[attr-defined]
            "path": self.path,
            "headers": {str(key).lower(): str(value) for key, value in self.headers.items()},
        }
        body = b"origin-ok"
        self.send_response(200)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


@contextmanager
def _serve(handler_factory):
    server = ThreadingHTTPServer(("127.0.0.1", 0), handler_factory)
    thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.1}, daemon=True)
    thread.start()
    try:
        yield server
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5.0)


class TrustedIngressProxyTests(unittest.TestCase):
    def _proxy_opener(self, proxy_url: str):
        return urllib.request.build_opener(
            urllib.request.ProxyHandler({"http": proxy_url, "https": proxy_url})
        )

    def test_proxy_injects_trusted_forwarded_headers_for_same_origin_request(self):
        with _serve(_RecordingOriginHandler) as origin:
            origin_base_url = f"http://127.0.0.1:{origin.server_port}"
            config = TrustedIngressProxyConfig(
                origin_base_url=origin_base_url,
                auth_token="trusted-token",
                forwarded_secret="forwarded-secret",
            )
            with _serve(build_proxy_handler(config)) as proxy:
                proxy_url = (
                    f"http://198.51.100.44:trusted-token@127.0.0.1:{proxy.server_port}"
                )
                opener = self._proxy_opener(proxy_url)
                request = urllib.request.Request(f"{origin_base_url}/research/?page=2", method="GET")
                request.add_header("User-Agent", "trusted-ingress-test")
                request.add_header("X-Shuma-Forwarded-Secret", "attacker-forged")
                with opener.open(request, timeout=5.0) as response:
                    body = response.read().decode("utf-8")

                self.assertEqual(response.status, 200)
                self.assertEqual(body, "origin-ok")
                recorded = origin.last_request  # type: ignore[attr-defined]
                self.assertEqual(recorded["path"], "/research/?page=2")
                self.assertEqual(recorded["headers"]["x-forwarded-for"], "198.51.100.44")
                self.assertEqual(recorded["headers"]["x-forwarded-proto"], "https")
                self.assertEqual(
                    recorded["headers"]["x-shuma-forwarded-secret"],
                    "forwarded-secret",
                )
                self.assertEqual(recorded["headers"]["user-agent"], "trusted-ingress-test")

    def test_proxy_rejects_invalid_proxy_credentials(self):
        with _serve(_RecordingOriginHandler) as origin:
            origin_base_url = f"http://127.0.0.1:{origin.server_port}"
            config = TrustedIngressProxyConfig(
                origin_base_url=origin_base_url,
                auth_token="trusted-token",
                forwarded_secret="forwarded-secret",
            )
            with _serve(build_proxy_handler(config)) as proxy:
                opener = self._proxy_opener(
                    f"http://198.51.100.44:wrong-token@127.0.0.1:{proxy.server_port}"
                )
                request = urllib.request.Request(f"{origin_base_url}/", method="GET")
                with self.assertRaises(urllib.error.HTTPError) as exc:
                    opener.open(request, timeout=5.0)
                self.assertEqual(exc.exception.code, 403)

    def test_proxy_rejects_cross_origin_targets(self):
        with _serve(_RecordingOriginHandler) as origin:
            origin_base_url = f"http://127.0.0.1:{origin.server_port}"
            config = TrustedIngressProxyConfig(
                origin_base_url=origin_base_url,
                auth_token="trusted-token",
                forwarded_secret="forwarded-secret",
            )
            with _serve(build_proxy_handler(config)) as proxy:
                opener = self._proxy_opener(
                    f"http://198.51.100.44:trusted-token@127.0.0.1:{proxy.server_port}"
                )
                request = urllib.request.Request("http://example.com/", method="GET")
                with self.assertRaises(urllib.error.HTTPError) as exc:
                    opener.open(request, timeout=5.0)
                self.assertEqual(exc.exception.code, 403)


if __name__ == "__main__":
    unittest.main()
