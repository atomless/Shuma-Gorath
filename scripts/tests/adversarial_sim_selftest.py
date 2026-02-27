#!/usr/bin/env python3
"""Minimal deterministic self-test harness for adversarial simulator mechanics.

This intentionally validates simulator mechanics only (ordering, retry/backoff,
budget stops, and gate math) against a tiny fixed-response stub server.
It does not try to validate product defense efficacy.
"""

from __future__ import annotations

import json
import random
import threading
import time
import unittest
import urllib.error
import urllib.request
from contextlib import contextmanager
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Dict, Iterable, List, Optional, Tuple


@dataclass(frozen=True)
class Scenario:
    id: str
    path: str
    expected_outcome: str
    max_latency_ms: int
    max_retries: int = 0
    initial_backoff_ms: int = 5


@dataclass(frozen=True)
class ScenarioOutcome:
    id: str
    observed_outcome: str
    passed: bool
    latency_ms: int
    retries: int
    retry_classification: str
    terminal_reason: str


class StubState:
    def __init__(self) -> None:
        self.lock = threading.Lock()
        self.flaky_hits = 0

    def next_flaky_hit(self) -> int:
        with self.lock:
            self.flaky_hits += 1
            return self.flaky_hits


class StubHandler(BaseHTTPRequestHandler):
    # Silence stdlib request logging to keep output deterministic and concise.
    def log_message(self, _format: str, *_args: object) -> None:  # noqa: A003
        return

    def do_GET(self) -> None:  # noqa: N802
        state = getattr(self.server, "stub_state")
        assert isinstance(state, StubState)

        if self.path == "/allow":
            self._send(200, "allow", delay_ms=6)
            return
        if self.path == "/challenge":
            self._send(200, "challenge", delay_ms=8)
            return
        if self.path == "/deny":
            self._send(403, "deny", delay_ms=5)
            return
        if self.path == "/flaky":
            hit = state.next_flaky_hit()
            if hit == 1:
                self._send(503, "temporary_unavailable", delay_ms=4)
            else:
                self._send(200, "allow", delay_ms=6)
            return

        self._send(404, "not_found", delay_ms=2)

    def _send(self, status: int, body: str, delay_ms: int) -> None:
        time.sleep(max(0, delay_ms) / 1000.0)
        payload = body.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


@contextmanager
def running_stub_server() -> Iterable[Tuple[str, threading.Thread]]:
    server = ThreadingHTTPServer(("127.0.0.1", 0), StubHandler)
    setattr(server, "stub_state", StubState())
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base_url = f"http://127.0.0.1:{server.server_address[1]}"
    try:
        yield base_url, thread
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)


def classify_outcome(status: int, body: str) -> str:
    lowered = (body or "").lower()
    if status in {429, 500, 502, 503, 504}:
        return "transient_error"
    if status == 403 or "deny" in lowered:
        return "deny_temp"
    if "challenge" in lowered:
        return "challenge"
    if status == 200 and "allow" in lowered:
        return "allow"
    return "unknown"


def percentile(values: List[int], pct: int) -> int:
    if not values:
        return 0
    ordered = sorted(values)
    idx = int(round((pct / 100.0) * (len(ordered) - 1)))
    idx = max(0, min(len(ordered) - 1, idx))
    return ordered[idx]


def compute_gate_snapshot(
    outcomes: List[ScenarioOutcome], *, p95_limit_ms: int, failure_ratio_max: float
) -> Dict[str, object]:
    latencies = [outcome.latency_ms for outcome in outcomes if outcome.terminal_reason == "ok"]
    p95 = percentile(latencies, 95)
    total = max(1, len(outcomes))
    failed = len([outcome for outcome in outcomes if not outcome.passed])
    failure_ratio = failed / total
    checks = [
        {
            "name": "latency_p95",
            "passed": p95 <= p95_limit_ms,
            "observed": p95,
            "limit": p95_limit_ms,
            "threshold_source": "selftest.profile.latency_p95_limit_ms",
        },
        {
            "name": "failure_ratio",
            "passed": failure_ratio <= failure_ratio_max,
            "observed": failure_ratio,
            "limit": failure_ratio_max,
            "threshold_source": "selftest.profile.failure_ratio_max",
        },
    ]
    return {"all_passed": all(check["passed"] for check in checks), "checks": checks}


def request_once(base_url: str, path: str, timeout_s: float = 2.0) -> Tuple[int, str, int]:
    req = urllib.request.Request(url=f"{base_url}{path}", method="GET")
    start = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=timeout_s) as response:
            body = response.read().decode("utf-8", errors="replace")
            status = int(response.getcode() or 0)
    except urllib.error.HTTPError as err:
        body = err.read().decode("utf-8", errors="replace")
        status = int(err.code)
    latency_ms = int((time.monotonic() - start) * 1000)
    return status, body, latency_ms


def run_profile(
    base_url: str,
    scenarios: List[Scenario],
    *,
    seed: int,
    suite_runtime_budget_ms: int,
) -> Tuple[List[str], List[ScenarioOutcome]]:
    ordered = list(scenarios)
    rng = random.Random(seed)
    rng.shuffle(ordered)
    start = time.monotonic()
    sequence = [scenario.id for scenario in ordered]
    outcomes: List[ScenarioOutcome] = []

    for scenario in ordered:
        elapsed_ms = int((time.monotonic() - start) * 1000)
        if elapsed_ms > suite_runtime_budget_ms:
            outcomes.append(
                ScenarioOutcome(
                    id=scenario.id,
                    observed_outcome="not_run",
                    passed=False,
                    latency_ms=0,
                    retries=0,
                    retry_classification="none",
                    terminal_reason="suite_runtime_budget_exceeded",
                )
            )
            break

        retries = 0
        backoff_ms = scenario.initial_backoff_ms
        retry_classification = "none"
        observed_outcome = "unknown"
        terminal_reason = "ok"
        latency_ms = 0

        while True:
            status, body, latency_ms = request_once(base_url, scenario.path)
            observed_outcome = classify_outcome(status, body)
            if observed_outcome != "transient_error":
                if retries > 0:
                    retry_classification = "transient_recovered"
                break
            if retries >= scenario.max_retries:
                retry_classification = "transient_exhausted"
                terminal_reason = "retry_exhausted"
                break
            retries += 1
            retry_classification = "transient_retrying"
            time.sleep(backoff_ms / 1000.0)
            backoff_ms *= 2

        passed = (
            terminal_reason == "ok"
            and observed_outcome == scenario.expected_outcome
            and latency_ms <= scenario.max_latency_ms
        )
        if terminal_reason == "ok" and latency_ms > scenario.max_latency_ms:
            terminal_reason = "scenario_latency_budget_exceeded"

        outcomes.append(
            ScenarioOutcome(
                id=scenario.id,
                observed_outcome=observed_outcome,
                passed=passed,
                latency_ms=latency_ms,
                retries=retries,
                retry_classification=retry_classification,
                terminal_reason=terminal_reason,
            )
        )

    return sequence, outcomes


class SimulatorSelfTest(unittest.TestCase):
    def test_seed_reproducibility_and_ordering(self) -> None:
        scenarios = [
            Scenario("s_allow", "/allow", "allow", 250),
            Scenario("s_challenge", "/challenge", "challenge", 250),
            Scenario("s_deny", "/deny", "deny_temp", 250),
        ]
        with running_stub_server() as (base_url, _thread):
            seq1, out1 = run_profile(base_url, scenarios, seed=17, suite_runtime_budget_ms=1000)
            seq2, out2 = run_profile(base_url, scenarios, seed=17, suite_runtime_budget_ms=1000)
        self.assertEqual(seq1, seq2)
        self.assertEqual([(o.id, o.passed, o.observed_outcome) for o in out1], [(o.id, o.passed, o.observed_outcome) for o in out2])

    def test_runtime_budget_enforcement_produces_hard_stop(self) -> None:
        scenarios = [
            Scenario("s_allow", "/allow", "allow", 250),
            Scenario("s_challenge", "/challenge", "challenge", 250),
        ]
        with running_stub_server() as (base_url, _thread):
            _seq, outcomes = run_profile(base_url, scenarios, seed=1, suite_runtime_budget_ms=1)
        self.assertTrue(any(outcome.terminal_reason == "suite_runtime_budget_exceeded" for outcome in outcomes))

    def test_retry_backoff_classification_is_deterministic(self) -> None:
        scenarios = [Scenario("s_flaky", "/flaky", "allow", 250, max_retries=2, initial_backoff_ms=5)]
        with running_stub_server() as (base_url, _thread):
            _seq, outcomes = run_profile(base_url, scenarios, seed=2, suite_runtime_budget_ms=1000)
        self.assertEqual(len(outcomes), 1)
        self.assertEqual(outcomes[0].retry_classification, "transient_recovered")
        self.assertEqual(outcomes[0].retries, 1)
        self.assertTrue(outcomes[0].passed)

    def test_quantitative_gate_snapshot_reports_observed_values(self) -> None:
        outcomes = [
            ScenarioOutcome("a", "allow", True, 10, 0, "none", "ok"),
            ScenarioOutcome("b", "challenge", True, 20, 0, "none", "ok"),
            ScenarioOutcome("c", "deny_temp", False, 30, 0, "none", "scenario_latency_budget_exceeded"),
        ]
        snapshot = compute_gate_snapshot(outcomes, p95_limit_ms=25, failure_ratio_max=0.40)
        self.assertIn("checks", snapshot)
        checks = {item["name"]: item for item in snapshot["checks"]}
        self.assertEqual(checks["latency_p95"]["observed"], 20)
        self.assertTrue(checks["failure_ratio"]["passed"])

    def test_hard_stop_teardown_stops_server_thread(self) -> None:
        scenarios = [Scenario("s_allow", "/allow", "allow", 250)]
        thread: Optional[threading.Thread] = None
        with running_stub_server() as (base_url, server_thread):
            thread = server_thread
            _seq, outcomes = run_profile(base_url, scenarios, seed=9, suite_runtime_budget_ms=1)
            self.assertTrue(outcomes)
        self.assertIsNotNone(thread)
        assert thread is not None
        self.assertFalse(thread.is_alive())


def main() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(SimulatorSelfTest)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    summary = {
        "schema_version": "sim-selftest.v1",
        "passed": result.wasSuccessful(),
        "tests_run": result.testsRun,
        "failures": len(result.failures),
        "errors": len(result.errors),
    }
    print(json.dumps(summary, separators=(",", ":")))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
