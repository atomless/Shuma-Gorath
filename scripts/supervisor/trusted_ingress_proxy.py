#!/usr/bin/env python3
"""Local trusted-ingress proxy for adversary-sim worker realism.

This proxy is intentionally narrow:
- it only accepts authenticated local proxy traffic,
- it only forwards to the configured Shuma origin,
- and it injects the same trusted forwarding headers the runtime already expects.

Workers never receive the forwarded-secret directly.
"""

from __future__ import annotations

import argparse
import base64
import ipaddress
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import os
from typing import Iterable
import urllib.error
import urllib.parse
import urllib.request


HOP_BY_HOP_HEADERS = {
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
}


class NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class TrustedIngressProxyConfig:
    def __init__(self, *, origin_base_url: str, auth_token: str, forwarded_secret: str):
        parsed = urllib.parse.urlparse(str(origin_base_url or "").strip())
        if not parsed.scheme or parsed.scheme not in {"http", "https"} or not parsed.netloc:
            raise ValueError("origin_base_url must be an absolute http or https URL")
        if not str(auth_token or "").strip():
            raise ValueError("auth_token must not be empty")
        if not str(forwarded_secret or "").strip():
            raise ValueError("forwarded_secret must not be empty")
        self.origin_scheme = parsed.scheme
        self.origin_netloc = parsed.netloc
        self.origin_base_url = urllib.parse.urlunparse(
            (parsed.scheme, parsed.netloc, "", "", "", "")
        ).rstrip("/")
        self.auth_token = str(auth_token).strip()
        self.forwarded_secret = str(forwarded_secret).strip()

    def target_url_for_proxy_path(self, raw_target: str) -> str:
        target = str(raw_target or "").strip()
        if not target:
            raise ValueError("proxy target missing")
        parsed = urllib.parse.urlparse(target)
        if parsed.scheme and parsed.netloc:
            if parsed.scheme != self.origin_scheme or parsed.netloc != self.origin_netloc:
                raise PermissionError("proxy target must stay same-origin")
            path = parsed.path or "/"
            return urllib.parse.urlunparse(
                (self.origin_scheme, self.origin_netloc, path, "", parsed.query, "")
            )
        if not target.startswith("/"):
            raise PermissionError("proxy target must be absolute-url or slash path")
        return f"{self.origin_base_url}{target}"


def _decode_proxy_authorization(raw_header: str) -> tuple[str, str] | None:
    value = str(raw_header or "").strip()
    if not value:
        return None
    scheme, _, encoded = value.partition(" ")
    if scheme.lower() != "basic" or not encoded.strip():
        return None
    try:
        decoded = base64.b64decode(encoded.strip(), validate=True).decode("utf-8")
    except Exception:
        return None
    username, separator, password = decoded.partition(":")
    if not separator:
        return None
    return username, password


def _parse_client_ip(raw_value: str) -> str | None:
    value = str(raw_value or "").strip()
    if not value:
        return None
    try:
        return str(ipaddress.ip_address(value))
    except ValueError:
        return None


def _forward_headers(headers: Iterable[tuple[str, str]]) -> dict[str, str]:
    forwarded: dict[str, str] = {}
    for key, value in headers:
        lowered = str(key or "").strip().lower()
        if not lowered or lowered in HOP_BY_HOP_HEADERS:
            continue
        if lowered in {"host", "x-forwarded-for", "x-forwarded-proto", "x-shuma-forwarded-secret"}:
            continue
        forwarded[str(key)] = str(value)
    return forwarded


def build_proxy_handler(config: TrustedIngressProxyConfig):
    opener = urllib.request.build_opener(NoRedirectHandler)

    class TrustedIngressProxyHandler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, format, *args):  # noqa: A003
            return

        def _reject(self, status: HTTPStatus, detail: str) -> None:
            payload = detail.encode("utf-8", errors="replace")
            self.send_response(status.value)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(payload)

        def _proxy_auth(self) -> tuple[str, str] | None:
            return _decode_proxy_authorization(self.headers.get("Proxy-Authorization", ""))

        def _forward(self) -> None:
            auth = self._proxy_auth()
            if auth is None:
                self.send_response(HTTPStatus.PROXY_AUTHENTICATION_REQUIRED.value)
                self.send_header("Proxy-Authenticate", 'Basic realm="shuma-trusted-ingress"')
                self.send_header("Content-Length", "0")
                self.send_header("Connection", "close")
                self.end_headers()
                return
            username, password = auth
            client_ip = _parse_client_ip(username)
            if client_ip is None or password != config.auth_token:
                self._reject(HTTPStatus.FORBIDDEN, "trusted_ingress_auth_failed")
                return
            try:
                target_url = config.target_url_for_proxy_path(self.path)
            except PermissionError:
                self._reject(HTTPStatus.FORBIDDEN, "trusted_ingress_origin_mismatch")
                return
            except ValueError:
                self._reject(HTTPStatus.BAD_REQUEST, "trusted_ingress_target_invalid")
                return

            request_headers = _forward_headers(self.headers.items())
            request_headers["X-Forwarded-For"] = client_ip
            request_headers["X-Forwarded-Proto"] = "https"
            request_headers["X-Shuma-Forwarded-Secret"] = config.forwarded_secret

            body = None
            content_length = int(self.headers.get("Content-Length", "0") or 0)
            if content_length > 0:
                body = self.rfile.read(content_length)
            request = urllib.request.Request(
                target_url,
                data=body,
                headers=request_headers,
                method=self.command,
            )
            try:
                with opener.open(request, timeout=15.0) as upstream:
                    response_body = upstream.read()
                    self.send_response(upstream.status)
                    for key, value in upstream.headers.items():
                        lowered = str(key or "").strip().lower()
                        if lowered in HOP_BY_HOP_HEADERS or lowered == "content-length":
                            continue
                        self.send_header(str(key), str(value))
                    self.send_header("Content-Length", str(len(response_body)))
                    self.send_header("Connection", "close")
                    self.end_headers()
                    if self.command.upper() != "HEAD":
                        self.wfile.write(response_body)
            except urllib.error.HTTPError as exc:
                response_body = exc.read()
                self.send_response(exc.code)
                for key, value in exc.headers.items():
                    lowered = str(key or "").strip().lower()
                    if lowered in HOP_BY_HOP_HEADERS or lowered == "content-length":
                        continue
                    self.send_header(str(key), str(value))
                self.send_header("Content-Length", str(len(response_body)))
                self.send_header("Connection", "close")
                self.end_headers()
                if self.command.upper() != "HEAD":
                    self.wfile.write(response_body)
            except Exception:
                self._reject(HTTPStatus.BAD_GATEWAY, "trusted_ingress_upstream_failed")

        def do_GET(self):  # noqa: N802
            self._forward()

        def do_POST(self):  # noqa: N802
            self._forward()

        def do_PUT(self):  # noqa: N802
            self._forward()

        def do_PATCH(self):  # noqa: N802
            self._forward()

        def do_DELETE(self):  # noqa: N802
            self._forward()

        def do_HEAD(self):  # noqa: N802
            self._forward()

        def do_OPTIONS(self):  # noqa: N802
            self._forward()

        def do_CONNECT(self):  # noqa: N802
            self._reject(HTTPStatus.METHOD_NOT_ALLOWED, "trusted_ingress_connect_unsupported")

    return TrustedIngressProxyHandler


def run_server(*, listen_host: str, listen_port: int, config: TrustedIngressProxyConfig) -> None:
    httpd = ThreadingHTTPServer((listen_host, listen_port), build_proxy_handler(config))
    try:
        httpd.serve_forever(poll_interval=0.5)
    finally:
        httpd.server_close()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run the local trusted-ingress proxy for adversary-sim worker IP realism"
    )
    parser.add_argument("--listen-host", default=os.environ.get("ADVERSARY_SIM_TRUSTED_INGRESS_LISTEN_HOST", "127.0.0.1"))
    parser.add_argument("--listen-port", type=int, default=int(os.environ.get("ADVERSARY_SIM_TRUSTED_INGRESS_LISTEN_PORT", "3871")))
    parser.add_argument(
        "--origin-base-url",
        default=os.environ.get("ADVERSARY_SIM_TRUSTED_INGRESS_ORIGIN_BASE_URL", "http://127.0.0.1:3000"),
    )
    parser.add_argument(
        "--auth-token",
        default=os.environ.get("ADVERSARY_SIM_TRUSTED_INGRESS_AUTH_TOKEN", ""),
    )
    parser.add_argument(
        "--forwarded-secret",
        default=os.environ.get("SHUMA_FORWARDED_IP_SECRET", ""),
    )
    args = parser.parse_args()
    try:
        config = TrustedIngressProxyConfig(
            origin_base_url=args.origin_base_url,
            auth_token=args.auth_token,
            forwarded_secret=args.forwarded_secret,
        )
    except ValueError as error:
        raise SystemExit(str(error))
    run_server(
        listen_host=str(args.listen_host).strip() or "127.0.0.1",
        listen_port=int(args.listen_port),
        config=config,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
